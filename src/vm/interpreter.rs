use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;
use tracing::debug;

use super::native_date::create_date_object;
use super::native_fs::create_fs_object;
#[cfg(feature = "http")]
use super::native_http::create_http_object;
use super::native_io::create_io_object;
use super::native_json::create_json_object;
use super::native_process::create_process_object;
use crate::value::{ObjClosure, ObjUpvalue, UpvalueState, Value, ValueError, ValueType};
use crate::vm::fiber::{CallFrame, ExceptionHandler};
use crate::vm::{Fiber, FiberState, Instruction, Program, RuntimeErrorWithContext, VM, VMResult, VMRuntimeError};

impl VM {
    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        self.execute_rc(Rc::new(program.clone()))
    }

    /// 核心事件循环 - 处理就绪任务并等待新任务
    /// 统一的 async 实现，被 Native 和 WASM 版本共用
    async fn run_event_loop(&mut self) -> VMResult {
        // Initial Execution
        let mut last_res = self.execute_from(0);

        loop {
            // Process Ready Queue (Async Tasks Completion)
            let mut did_work = false;
            let queue = self.async_state.ready_queue.clone();

            // We must detach borrow to allow mutation during resume
            let mut ready_tasks = Vec::new();
            {
                let mut q = queue.borrow_mut();
                while let Some(task) = q.pop_front() {
                    ready_tasks.push(task);
                }
            }

            if !ready_tasks.is_empty() {
                did_work = true;
                // Resume all ready tasks
                for (fiber, res) in ready_tasks {
                    // 1. Set current fiber
                    self.current_fiber = Some(fiber.clone());
                    self.load_state_from_fiber(&fiber.borrow());

                    // 2. Push result to stack (only if not a new spawned coroutine)
                    {
                        let mut f = fiber.borrow_mut();
                        if f.skip_push_on_resume {
                            f.skip_push_on_resume = false;
                        } else {
                            match res {
                                Ok(val) => {
                                    self.stack.push(val);
                                }
                                Err(err) => {
                                    let program = self.program.clone().expect("program should be set");
                                    self.stack.push(Value::string(err.to_string()));
                                    if let Err(e) = self.execute_instruction(&Instruction::Throw, &program) {
                                        return Err(RuntimeErrorWithContext {
                                            error: e,
                                            line: 0,
                                            pc: self.pc,
                                        });
                                    }
                                    // Align with main loop semantics: Throw sets PC to target-1,
                                    // so we need to advance once before re-entering execute_from.
                                    self.pc = self.pc.saturating_add(1);
                                }
                            }
                        }
                    }

                    // 3. Continue execution
                    fiber.borrow_mut().state = FiberState::Running;
                    last_res = self.execute_from(self.pc);

                    // 4. Check fiber completion and save result
                    // Only if execution finished successfully (not yielded)
                    if let Ok(ref result) = last_res {
                        let mut f = fiber.borrow_mut();

                        // We consider the fiber finished if:
                        // 1. It is explicitly marked Dead (by Return instruction)
                        // 2. It is still Running but call stack is empty (ran off end of script)
                        // IMPORTANT: If it is Suspended, it yielded (e.g. async I/O), so we must NOT mark it dead.
                        let is_finished =
                            f.state == FiberState::Dead || (f.state == FiberState::Running && f.call_stack.is_empty());

                        if is_finished {
                            f.result = Some(result.clone());
                            f.state = FiberState::Dead;
                            if f.is_spawned {
                                let mut pt = self.async_state.pending_tasks.borrow_mut();
                                // println!("DEBUG: Fiber finished. Decrementing pending: {} -> {}", *pt, *pt - 1);
                                *pt -= 1;
                            }
                            self.async_state.notify.notify_waiters();
                        }
                    }

                    // 5. Check if we need to propagate error
                    if let Err(e) = &last_res {
                        // If it's just a Yield, we don't propagate it as a VM error
                        // The fiber is already suspended.
                        if matches!(e.error, VMRuntimeError::Yield) {
                            // Continue loop
                        } else {
                            return last_res;
                        }
                    }
                }
            }

            if !did_work {
                let pending = *self.async_state.pending_tasks.borrow();
                if pending == 0 {
                    break;
                }
                // Wait for notification from async tasks
                self.async_state.notify.notified().await;
            }
        }

        last_res
    }

    /// Execute program asynchronously (for WASM).
    /// This keeps the VM alive to handle callbacks.
    #[cfg(target_arch = "wasm32")]
    pub async fn execute_async(&mut self, program: Rc<Program>) -> VMResult {
        let saved_program = self.program.clone();
        self.program = Some(program.clone());

        let res = self.run_event_loop().await;

        self.program = saved_program;
        res
    }

    pub fn execute_rc(&mut self, program: Rc<Program>) -> VMResult {
        let saved_program = self.program.clone();
        self.program = Some(program.clone());

        // Check if we are already in a runtime (e.g. recursive import or nested call)
        if tokio::runtime::Handle::try_current().is_ok() {
            // Already in a runtime - just run synchronously
            let res = self.execute_from(0);
            self.program = saved_program;
            return res;
        }

        #[cfg(not(target_arch = "wasm32"))]
        let res = {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local = tokio::task::LocalSet::new();

            local.block_on(&rt, self.run_event_loop())
        };

        #[cfg(target_arch = "wasm32")]
        let res = self.execute_from(0);

        self.program = saved_program;
        res
    }

    fn capture_upvalue(&mut self, location: usize) -> Rc<RefCell<UpvalueState>> {
        for upvalue in &self.open_upvalues {
            let state = upvalue.borrow();
            if let UpvalueState::Open(idx) = *state
                && idx == location
            {
                return upvalue.clone();
            }
        }
        let created = Rc::new(RefCell::new(UpvalueState::Open(location)));
        self.open_upvalues.push(created.clone());
        created
    }

    fn close_upvalues(&mut self, last: usize) {
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let upvalue_rc = &self.open_upvalues[i];
            let should_close = {
                let state = upvalue_rc.borrow();
                if let UpvalueState::Open(location) = *state {
                    location >= last
                } else {
                    true
                }
            };

            if should_close {
                let upvalue_rc = self.open_upvalues.remove(i);
                let location = if let UpvalueState::Open(loc) = *upvalue_rc.borrow() {
                    loc
                } else {
                    0
                };
                let value = self.stack[location].clone();
                *upvalue_rc.borrow_mut() = UpvalueState::Closed(value);
            } else {
                i += 1;
            }
        }
    }

    /// 从指定PC开始执行程序
    pub fn execute_from(&mut self, start_pc: usize) -> VMResult {
        self.pc = start_pc;

        loop {
            let (instruction_clone, program_clone) = {
                let program = self.program.as_ref().ok_or_else(|| RuntimeErrorWithContext {
                    error: VMRuntimeError::UndefinedVariable("No program loaded".into()),
                    line: 0,
                    pc: self.pc,
                })?;

                if self.pc >= program.instructions.len() {
                    break;
                }

                let instruction = program.instructions[self.pc].clone();
                let program = program.clone();
                (instruction, program)
            };

            debug!("Executing instruction {}: {:?}", self.pc, instruction_clone);

            match self.execute_instruction(&instruction_clone, &program_clone) {
                Ok(continue_execution) => {
                    if !continue_execution {
                        debug!("Execution stopped at PC {}", self.pc);
                        break;
                    }
                }
                Err(error) => {
                    if let VMRuntimeError::Yield = error {
                        // PC has been handled by the native function if necessary
                        break;
                    }

                    let line = *program_clone.lines.get(&self.pc).unwrap_or(&0);
                    debug!("Execution error at PC {} (Line {}): {}", self.pc, line, error);
                    return Err(RuntimeErrorWithContext {
                        error,
                        line,
                        pc: self.pc,
                    });
                }
            }

            self.pc += 1;
        }

        debug!("Execution completed. PC: {}, Stack: {:?}", self.pc, self.stack);

        let result = self.stack.pop().unwrap_or(Value::null());
        Ok(result)
    }

    pub fn save_state_to_fiber(&self, fiber: &mut Fiber) {
        fiber.stack = self.stack.clone();
        fiber.pc = self.pc;
        fiber.fp = self.fp;
        fiber.call_stack = self.call_stack.clone();
        fiber.exception_handlers = self.exception_handlers.clone();
        fiber.current_closure = self.current_closure.clone();
        fiber.program = self.program.clone();
    }

    pub fn load_state_from_fiber(&mut self, fiber: &Fiber) {
        self.stack = fiber.stack.clone();
        self.pc = fiber.pc;
        self.fp = fiber.fp;
        self.call_stack = fiber.call_stack.clone();
        self.exception_handlers = fiber.exception_handlers.clone();
        self.current_closure = fiber.current_closure.clone();
        self.program = fiber.program.clone();
    }

    /// 执行单条指令
    fn execute_instruction(&mut self, instruction: &Instruction, program: &Program) -> Result<bool, VMRuntimeError> {
        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value.clone());
            }

            Instruction::Import(path) => {
                if path.starts_with("stdlib/") {
                    match path.as_str() {
                        "stdlib/json" => {
                            let module = create_json_object();
                            self.stack.push(module);
                        }
                        "stdlib/date" => {
                            let module = create_date_object();
                            self.stack.push(module);
                        }
                        "stdlib/fs" => {
                            let module = create_fs_object();
                            self.stack.push(module);
                        }
                        "stdlib/http" => {
                            #[cfg(feature = "http")]
                            {
                                let module = create_http_object();
                                self.stack.push(module);
                            }
                            #[cfg(not(feature = "http"))]
                            self.stack.push(Value::Null);
                        }
                        "stdlib/process" => {
                            let module = create_process_object();
                            self.stack.push(module);
                        }
                        "stdlib/io" => {
                            let module = create_io_object();
                            self.stack.push(module);
                        }
                        "stdlib/timer" => {
                            let module = super::native_timer::create_timer_object();
                            self.stack.push(module);
                        }
                        _ => {
                            return Err(VMRuntimeError::UndefinedVariable(format!(
                                "Stdlib module not found: {}",
                                path
                            )));
                        }
                    }
                } else {
                    if let Some(cached_val) = self.module_cache.get(path) {
                        self.stack.push(cached_val.clone());
                        return Ok(true);
                    }

                    let code = match std::fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            return Err(VMRuntimeError::UncaughtException(format!(
                                "Failed to import {}: {}",
                                path, e
                            )));
                        }
                    };

                    let ast = match crate::parser::parse_from_source(&code) {
                        Ok(a) => a,
                        Err(e) => {
                            return Err(VMRuntimeError::UncaughtException(format!(
                                "Parse error in {}: {}",
                                path, e
                            )));
                        }
                    };

                    let module_program = crate::compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

                    let saved_stack_size = self.stack.len();
                    let saved_pc = self.pc;
                    let saved_fp = self.fp;

                    let res = self.execute_rc(Rc::new(module_program));

                    self.pc = saved_pc;
                    self.fp = saved_fp;

                    match res {
                        Ok(val) => {
                            self.stack.truncate(saved_stack_size);
                            self.module_cache.insert(path.clone(), val.clone());
                            self.stack.push(val);
                        }
                        Err(e) => {
                            self.stack.truncate(saved_stack_size);
                            return Err(e.error);
                        }
                    }
                }
            }

            Instruction::BuildArray(count) => {
                let mut table = crate::value::Table {
                    data: IndexMap::new(),
                    metatable: if let Value::Object(proto_rc) = &self.array_prototype {
                        Some(proto_rc.clone())
                    } else {
                        None
                    },
                };

                let start_index = self
                    .stack
                    .len()
                    .checked_sub(*count)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "Stack underflow during array creation".to_string(),
                    ))?;

                for i in 0..*count {
                    let val = self.stack[start_index + i].clone();
                    table.data.insert(i.to_string(), val);
                }

                self.stack.truncate(start_index);
                let mut table_ref = table;
                if let Value::Object(proto_table) = &self.array_prototype {
                    table_ref.metatable = Some(proto_table.clone());
                }

                self.stack.push(Value::Object(Rc::new(RefCell::new(table_ref))));
            }

            Instruction::Pop => {
                self.stack.pop();
            }

            Instruction::Dup => {
                if let Some(top) = self.stack.last() {
                    self.stack.push(top.clone());
                } else {
                    return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "dup".to_string(),
                        left_type: ValueType::Null,
                        right_type: ValueType::Null,
                    }));
                }
            }

            Instruction::Load(var_name) => {
                if let Some(value) = self.variables.get(var_name) {
                    debug!("Loading variable {} = {:?}", var_name, value);
                    self.stack.push(value.clone());
                } else {
                    let func_label = format!("func_{}", var_name);
                    if let Some(prog) = &self.program {
                        if let Some(symbol) = prog.syms.get(&func_label) {
                            // Create a closure with empty upvalues for legacy function references
                            let closure = crate::value::ObjClosure {
                                name: var_name.clone(),
                                func_symbol: symbol.clone(),
                                program: prog.clone(),
                                upvalues: Vec::new(), // No upvalues for top-level functions
                            };
                            self.stack.push(Value::Fn(Rc::new(closure)));
                        } else {
                            return Err(VMRuntimeError::UndefinedVariable(var_name.clone()));
                        }
                    } else {
                        return Err(VMRuntimeError::UndefinedVariable(var_name.clone()));
                    }
                }
            }

            Instruction::Store(var_name) => {
                if let Some(value) = self.stack.pop() {
                    debug!("Storing value {:?} to variable {}", value, var_name);
                    self.variables.insert(var_name.clone(), value);
                } else {
                    return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "store".to_string(),
                        left_type: ValueType::Null,
                        right_type: ValueType::Null,
                    }));
                }
            }

            Instruction::Add => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.add(&right)?;

                match op_result {
                    crate::value::OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    crate::value::OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        self.stack.push(call_info.left);
                        self.stack.push(call_info.right);

                        let call_stack_instr = Instruction::CallStack(2);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Subtract => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.subtract(&right)?;

                match op_result {
                    crate::value::OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    crate::value::OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        self.stack.push(call_info.left);
                        self.stack.push(call_info.right);

                        let call_stack_instr = Instruction::CallStack(2);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Multiply => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.multiply(&right)?;

                match op_result {
                    crate::value::OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    crate::value::OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        self.stack.push(call_info.left);
                        self.stack.push(call_info.right);

                        let call_stack_instr = Instruction::CallStack(2);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Divide => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.divide(&right)?;
                self.stack.push(result);
            }

            Instruction::Modulo => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.modulo(&right)?;
                self.stack.push(result);
            }

            Instruction::Equal => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.equal(&right);
                self.stack.push(result);
            }

            Instruction::NotEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.not_equal(&right);
                self.stack.push(result);
            }

            Instruction::LessThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.less_than(&right)?;
                self.stack.push(result);
            }

            Instruction::LessThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.less_equal(&right)?;
                self.stack.push(result);
            }

            Instruction::GreaterThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.greater_than(&right)?;
                self.stack.push(result);
            }

            Instruction::GreaterThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.greater_equal(&right)?;
                self.stack.push(result);
            }

            Instruction::And => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.and(&right);
                self.stack.push(result);
            }

            Instruction::Or => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.or(&right);
                self.stack.push(result);
            }

            Instruction::Not => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let result = value.not();
                self.stack.push(result);
            }

            Instruction::Jump(label) => {
                return if let Some(target) = program.syms.get(label) {
                    self.pc = (target.location as usize) - 1;
                    Ok(true)
                } else {
                    Err(VMRuntimeError::UndefinedLabel(format!("label: {}", label)))
                };
            }

            Instruction::JumpIfFalse(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if !condition.is_truthy() {
                    return if let Some(target) = program.syms.get(label) {
                        self.pc = (target.location as usize) - 1;
                        Ok(true)
                    } else {
                        Err(VMRuntimeError::UndefinedLabel(format!("label: {}", label)))
                    };
                }
            }

            Instruction::JumpIfTrue(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if condition.is_truthy() {
                    return if let Some(target) = program.syms.get(label) {
                        self.pc = (target.location as usize) - 1;
                        Ok(true)
                    } else {
                        Err(VMRuntimeError::UndefinedLabel(format!("label: {}", label)))
                    };
                }
            }

            Instruction::Call(func_name, arg_count) => {
                return match func_name.as_str() {
                    "set_meta" => {
                        if *arg_count != 2 {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "set_meta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }));
                        }
                        let metatable = self.stack.pop().unwrap_or(Value::null());
                        let obj = self.stack.pop().unwrap_or(Value::null());
                        obj.set_metatable(metatable)?;
                        self.stack.push(Value::null());
                        Ok(true)
                    }
                    "get_meta" => {
                        if *arg_count != 1 {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "get_meta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }));
                        }
                        let obj = self.stack.pop().unwrap_or(Value::null());
                        let metatable = obj.get_metatable();
                        self.stack.push(metatable);
                        Ok(true)
                    }
                    _ => {
                        let func_label = format!("func_{}", func_name);

                        // Try to find the function: either as a direct symbol or as a variable holding a closure
                        if let Some(sym) = program.syms.get(&func_label) {
                            // Direct symbol call (e.g. top-level function)
                            if *arg_count != sym.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call".to_string(),
                                    left_type: ValueType::Null,
                                    right_type: ValueType::Null,
                                }));
                            }

                            self.call_stack.push(CallFrame {
                                pc: self.pc,
                                fp: self.fp,
                                program: self.program.clone(),
                                closure: self.current_closure.clone(),
                            });
                            self.fp = self.stack.len() - *arg_count;

                            // For direct symbol calls, we should create a "base" closure if we want current_closure to be set,
                            // but usually these are top-level and don't need it.
                            // However, to be consistent with unified types, we should probably set it.
                            let closure = ObjClosure {
                                name: func_name.clone(),
                                func_symbol: sym.clone(),
                                program: self.program.clone().unwrap(),
                                upvalues: Vec::new(),
                            };
                            self.current_closure = Some(Rc::new(closure));

                            self.stack.resize(self.fp + sym.nlocals, Value::null());
                            self.pc = (sym.location as usize) - 1;
                            Ok(true)
                        } else if let Some(val) = self.variables.get(func_name).cloned() {
                            // Variable lookup
                            match val {
                                Value::Fn(closure) => {
                                    let sym = &closure.func_symbol;
                                    if *arg_count != sym.narguments {
                                        return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                            operator: "call".to_string(),
                                            left_type: ValueType::Function,
                                            right_type: ValueType::Null,
                                        }));
                                    }

                                    self.call_stack.push(CallFrame {
                                        pc: self.pc,
                                        fp: self.fp,
                                        program: self.program.clone(),
                                        closure: self.current_closure.clone(),
                                    });
                                    self.fp = self.stack.len() - *arg_count;
                                    self.program = Some(closure.program.clone());
                                    self.current_closure = Some(closure.clone());

                                    self.stack.resize(self.fp + sym.nlocals, Value::null());
                                    self.pc = (sym.location as usize) - 1;
                                    Ok(true)
                                }
                                Value::NativeFunction(native_fn) => {
                                    let start_index = self
                                        .stack
                                        .len()
                                        .checked_sub(*arg_count)
                                        .ok_or(VMRuntimeError::StackUnderflow("Native call missing args".into()))?;
                                    let args: Vec<Value> = self.stack.drain(start_index..).collect();
                                    let result = native_fn(self, args)?;
                                    self.stack.push(result);
                                    Ok(true)
                                }
                                _ => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call".to_string(),
                                    left_type: val.get_type(),
                                    right_type: ValueType::Null,
                                })),
                            }
                        } else {
                            Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                        }
                    }
                };
            }
            Instruction::Return => {
                let return_value = self.stack.pop().unwrap_or(Value::null());
                self.close_upvalues(self.fp);

                return if let Some(frame) = self.call_stack.pop() {
                    self.stack.truncate(self.fp);
                    self.pc = frame.pc;
                    self.fp = frame.fp;
                    if let Some(prog) = frame.program {
                        self.program = Some(prog);
                    }
                    debug!(
                        "[VM DEBUG] Return: restoring closure to {:?}",
                        frame.closure.as_ref().map(|c| &c.name)
                    );
                    self.current_closure = frame.closure;
                    self.stack.push(return_value);
                    Ok(true)
                } else if let Some(fiber_rc) = &self.current_fiber {
                    fiber_rc.borrow_mut().state = FiberState::Dead;
                    let caller_opt = fiber_rc.borrow().caller.clone();

                    if let Some(caller_rc) = caller_opt {
                        let caller = caller_rc.borrow();
                        self.load_state_from_fiber(&caller);
                        drop(caller);
                        self.current_fiber = Some(caller_rc);
                        self.stack.push(return_value);
                        Ok(false)
                    } else {
                        self.stack.push(return_value);
                        Ok(false)
                    }
                } else {
                    self.stack.push(return_value);
                    Ok(false)
                };
            }

            Instruction::Label(_) => {}

            Instruction::MovePlusFP(offset) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.fp + offset;

                if index >= self.stack.len() {
                    self.stack.resize(index + 1, Value::null());
                }

                self.stack[index] = value;
            }

            Instruction::DupPlusFP(offset) => {
                let index = self.fp + (*offset as usize);
                let value = self.stack.get(index).cloned().unwrap_or(Value::null());
                self.stack.push(value);
            }

            Instruction::GetUpvalue(index) => {
                let closure = self.current_closure.as_ref().ok_or_else(|| {
                    debug!(
                        "[VM DEBUG] GetUpvalue failed: current_closure is None! PC: {}, FP: {}",
                        self.pc, self.fp
                    );
                    VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "get_upvalue".into(),
                        left_type: ValueType::Null,
                        right_type: ValueType::Null,
                    })
                })?;
                let upvalue = &closure.upvalues[*index];
                let val = match &*upvalue.state.borrow() {
                    UpvalueState::Open(location) => self.stack[*location].clone(),
                    UpvalueState::Closed(value) => value.clone(),
                };
                self.stack.push(val);
            }

            Instruction::SetUpvalue(index) => {
                let val = self.stack.pop().unwrap();
                let closure = self.current_closure.as_ref().ok_or_else(|| {
                    VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "set_upvalue".into(),
                        left_type: ValueType::Null,
                        right_type: ValueType::Null,
                    })
                })?;
                let upvalue = &closure.upvalues[*index];
                match &mut *upvalue.state.borrow_mut() {
                    UpvalueState::Open(location) => self.stack[*location] = val,
                    UpvalueState::Closed(closed_val) => *closed_val = val,
                }
            }

            Instruction::CloseUpvalue => {
                self.close_upvalues(self.stack.len() - 1);
                self.stack.pop();
            }
            Instruction::CloseUpvaluesAbove(offset) => {
                self.close_upvalues(self.fp + offset);
            }

            Instruction::Closure(name) => {
                let symbol = program
                    .syms
                    .get(name)
                    .ok_or_else(|| VMRuntimeError::UndefinedVariable(format!("Function symbol not found: {}", name)))?;

                let mut upvalues = Vec::new();
                for (is_local, index) in &symbol.upvalues {
                    if *is_local {
                        upvalues.push(ObjUpvalue {
                            state: self.capture_upvalue(self.fp + index),
                        });
                    } else {
                        let current = self.current_closure.as_ref().ok_or_else(|| {
                            VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "closure".into(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            })
                        })?;
                        upvalues.push(current.upvalues[*index].clone());
                    }
                }

                let closure = ObjClosure {
                    name: name.clone(),
                    func_symbol: symbol.clone(),
                    program: self.program.clone().unwrap(),
                    upvalues,
                };

                self.stack.push(Value::Fn(Rc::new(closure)));
            }

            Instruction::NewObject => {
                self.stack.push(Value::object());
            }

            Instruction::GetField(field) => {
                let obj = self.stack.pop().unwrap_or(Value::null());
                let mut value = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(field)
                } else {
                    obj.get_field_with_meta(field)
                };

                if let Value::Null = value
                    && let Value::Object(_) = obj
                    && field == "keys"
                {
                    let array_proto = self.array_prototype.clone();
                    value = Value::NativeFunction(Rc::new(Box::new(move |_vm, args| {
                        if args.is_empty() {
                            return Err(ValueError::TypeMismatch {
                                expected: ValueType::Object,
                                found: ValueType::Null,
                                operation: "keys".into(),
                            }
                            .into());
                        }
                        let obj = &args[0];
                        if let Value::Object(table_rc) = obj {
                            let table = table_rc.borrow();
                            let mut data = IndexMap::new();
                            for (i, k) in table.data.keys().enumerate() {
                                data.insert(i.to_string(), Value::string(k.clone()));
                            }

                            let mut res_table = crate::value::Table { data, metatable: None };
                            if let Value::Object(proto_rc) = &array_proto {
                                res_table.metatable = Some(proto_rc.clone());
                            }

                            return Ok(Value::Object(Rc::new(RefCell::new(res_table))));
                        }
                        Err(ValueError::TypeMismatch {
                            expected: ValueType::Object,
                            found: obj.get_type(),
                            operation: "keys".into(),
                        }
                        .into())
                    })));
                }

                self.stack.push(value);
            }

            Instruction::SetField(field) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                obj.set_field_with_meta(field.clone(), value)?;
            }

            Instruction::GetMethod(field) => {
                let obj = self.stack.pop().unwrap_or(Value::null());
                let mut value = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(field)
                } else {
                    obj.get_field_with_meta(field)
                };

                if let Value::Null = value
                    && let Value::Object(_) = obj
                    && field == "keys"
                {
                    let array_proto = self.array_prototype.clone();
                    value = Value::NativeFunction(Rc::new(Box::new(move |_vm, args| {
                        if args.is_empty() {
                            return Err(ValueError::TypeMismatch {
                                expected: ValueType::Object,
                                found: ValueType::Null,
                                operation: "keys".into(),
                            }
                            .into());
                        }
                        let obj = &args[0];
                        if let Value::Object(table_rc) = obj {
                            let table = table_rc.borrow();
                            let mut data = IndexMap::new();
                            for (i, k) in table.data.keys().enumerate() {
                                data.insert(i.to_string(), Value::string(k.clone()));
                            }

                            let mut res_table = crate::value::Table { data, metatable: None };
                            if let Value::Object(proto_rc) = &array_proto {
                                res_table.metatable = Some(proto_rc.clone());
                            }

                            return Ok(Value::Object(Rc::new(RefCell::new(res_table))));
                        }
                        Err(ValueError::TypeMismatch {
                            expected: ValueType::Object,
                            found: obj.get_type(),
                            operation: "keys".into(),
                        }
                        .into())
                    })));
                }

                self.stack.push(value);
                self.stack.push(obj);
            }

            Instruction::GetIndex => {
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                match obj {
                    Value::Object(table_ref) => {
                        let key = index.to_string();
                        let table = table_ref.borrow();
                        let value = table.data.get(&key).cloned().unwrap_or(Value::null());
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                            operator: "get_index".to_string(),
                            left_type: obj.get_type(),
                            right_type: ValueType::Null,
                        }));
                    }
                }
            }

            Instruction::SetIndex => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                match obj {
                    Value::Object(table_ref) => {
                        let key = index.to_string();
                        table_ref.borrow_mut().data.insert(key, value);
                    }
                    _ => {
                        return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                            operator: "set_index".to_string(),
                            left_type: obj.get_type(),
                            right_type: ValueType::Null,
                        }));
                    }
                }
            }

            Instruction::CallStack(arg_count) => {
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(*arg_count + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallStack: missing function".to_string(),
                    ))?;

                let func_val = self.stack.remove(func_idx);

                return match func_val {
                    Value::Fn(closure) => {
                        let sym = &closure.func_symbol;
                        if *arg_count != sym.narguments {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "call_stack".to_string(),
                                left_type: ValueType::Function,
                                right_type: ValueType::Null,
                            }));
                        }

                        self.call_stack.push(CallFrame {
                            pc: self.pc,
                            fp: self.fp,
                            program: self.program.clone(),
                            closure: self.current_closure.clone(),
                        });
                        self.fp = self.stack.len() - *arg_count;
                        self.program = Some(closure.program.clone());
                        self.current_closure = Some(closure.clone());
                        debug!(
                            "[VM DEBUG] Call Fn: {}, new current_closure: Some({})",
                            closure.name, closure.name
                        );

                        self.stack.resize(self.fp + sym.nlocals, Value::null());
                        self.pc = (sym.location as usize) - 1;
                        Ok(true)
                    }
                    Value::NativeFunction(native_fn) => {
                        let start_index = self
                            .stack
                            .len()
                            .checked_sub(*arg_count)
                            .ok_or(VMRuntimeError::StackUnderflow("CallStack native: missing args".into()))?;
                        let args: Vec<Value> = self.stack.drain(start_index..).collect();

                        let result = native_fn(self, args);
                        let val = result?;

                        self.stack.push(val);
                        Ok(true)
                    }
                    _ => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "call_stack".to_string(),
                        left_type: func_val.get_type(),
                        right_type: ValueType::Null,
                    })),
                };
            }

            Instruction::Throw => {
                let error_value = self.stack.pop().unwrap_or(Value::string("Unknown error".to_string()));

                if let Some(handler) = self.exception_handlers.pop() {
                    self.stack.truncate(handler.stack_size);
                    self.fp = handler.fp;
                    self.stack.push(error_value);

                    return if let Some(target) = program.syms.get(&handler.catch_label) {
                        self.pc = (target.location as usize) - 1;
                        Ok(true)
                    } else {
                        Err(VMRuntimeError::UndefinedLabel(format!(
                            "catch label: {}",
                            handler.catch_label
                        )))
                    };
                }

                return Err(VMRuntimeError::UncaughtException(error_value.to_string()));
            }

            Instruction::PushExceptionHandler(catch_label) => {
                self.exception_handlers.push(ExceptionHandler {
                    catch_label: catch_label.clone(),
                    stack_size: self.stack.len(),
                    fp: self.fp,
                });
            }

            Instruction::PopExceptionHandler => {
                self.exception_handlers.pop();
            }
        }

        Ok(true)
    }
}
