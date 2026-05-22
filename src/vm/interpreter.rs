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
use crate::compiler::compile;
use crate::expression::Literal;
use crate::parser::parse_from_source;
use crate::tokenizer::Location;
use crate::value::{ObjClosure, ObjUpvalue, OpResult, UpvalueState, Value, ValueError, ValueType};
use crate::vm::fiber::{CallFrame, ExceptionHandler};
use crate::vm::{Fiber, FiberState, Instruction, Program, RuntimeErrorWithContext, VM, VMResult, VMRuntimeError};

impl VM {
    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        self.execute_rc(Rc::new(program.clone()))
    }

    /// 核心事件循环 - 处理就绪任务并等待新任务
    async fn run_event_loop(&mut self) -> VMResult {
        // Initial execution
        if self.current_fiber.is_none() {
            let root_fiber = Rc::new(RefCell::new(Fiber::new()));
            root_fiber.borrow_mut().state = FiberState::Running;
            self.current_fiber = Some(root_fiber);
        }
        
        let mut last_res = self.execute_from(self.pc);
        if let Some(f) = &self.current_fiber {
            self.save_state_to_fiber(&mut f.borrow_mut());
            if last_res.is_err() && !matches!(last_res.as_ref().unwrap_err().error, VMRuntimeError::Yield) {
                f.borrow_mut().state = FiberState::Dead;
            }
        }

        loop {
            // Check if we are done
            {
                let pending = *self.async_state.pending_tasks.borrow();
                let queue_empty = self.async_state.ready_queue.borrow().is_empty();

                let current_fiber_done = if let Some(f) = &self.current_fiber {
                    let f_borrow = f.borrow();
                    f_borrow.state == FiberState::Dead || (f_borrow.state == FiberState::Running && f_borrow.call_stack.is_empty())
                } else {
                    true
                };

                if pending == 0 && queue_empty {
                    if current_fiber_done {
                        break;
                    } else {
                        // Deadlock detection: all fibers suspended and no pending tasks
                        return Err(RuntimeErrorWithContext {
                            error: VMRuntimeError::UncaughtException("Deadlock detected: all fibers suspended and no pending tasks".to_string()),
                            loc: crate::tokenizer::Location::default(),
                            pc: self.pc,
                        });
                    }
                }
            }

            let mut did_work = false;
            let queue = self.async_state.ready_queue.clone();

            let mut ready_tasks = Vec::new();
            {
                let mut q = queue.borrow_mut();
                while let Some(task) = q.pop_front() {
                    ready_tasks.push(task);
                }
            }

            if !ready_tasks.is_empty() {
                did_work = true;
                for (fiber, res) in ready_tasks {
                    self.current_fiber = Some(fiber.clone());
                    self.load_state_from_fiber(&fiber.borrow());

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
                                    let error_msg = match &err {
                                        VMRuntimeError::UncaughtException(msg) => msg.clone(),
                                        _ => err.to_string(),
                                    };
                                    self.stack.push(Value::string(error_msg));
                                    match self.execute_instruction(&Instruction::Throw, &program) {
                                        Ok(_) => {
                                            self.pc += 1;
                                        }
                                        Err(e) => {
                                            last_res = Err(RuntimeErrorWithContext {
                                                error: e,
                                                loc: crate::tokenizer::Location::default(),
                                                pc: self.pc,
                                            });
                                            f.state = FiberState::Dead;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        f.state = FiberState::Running;
                    }

                    last_res = self.execute_from(self.pc);
                    
                    // CRITICAL: Save state immediately after execution
                    self.save_state_to_fiber(&mut fiber.borrow_mut());

                    if let Ok(ref result) = last_res {
                        let mut f = fiber.borrow_mut();

                        let is_finished =
                            f.state == FiberState::Dead || (f.state == FiberState::Running && f.call_stack.is_empty());

                        if is_finished {
                            f.result = Some(result.clone());
                            f.state = FiberState::Dead;

                            if let Some(promise_rc) = f.associated_promise.take() {
                                let final_state = if let Some(initial_state) = f.finally_initial_state.take() {
                                    initial_state
                                } else {
                                    crate::promise::PromiseState::Fulfilled(result.clone())
                                };
                                self.settle_promise(promise_rc, final_state);
                            }

                            if f.is_spawned {
                                let mut pt = self.async_state.pending_tasks.borrow_mut();
                                *pt -= 1;
                            }
                            self.async_state.notify.notify_waiters();
                        }
                    }

                    if let Err(e) = &last_res {
                        if matches!(e.error, VMRuntimeError::Yield) {
                        } else {
                            // Error in fiber, handle it
                            let mut f = fiber.borrow_mut();
                            f.state = FiberState::Dead;
                            if let Some(promise_rc) = f.associated_promise.take() {
                                let error_val = Value::string(e.to_string());
                                self.settle_promise(promise_rc, crate::promise::PromiseState::Rejected(error_val));
                            }
                            if let Some(promise_rc) = f.reject_on_error_promise.take() {
                                let error_val = Value::string(e.to_string());
                                self.settle_promise(promise_rc, crate::promise::PromiseState::Rejected(error_val));
                            }
                            if f.is_spawned {
                                let mut pt = self.async_state.pending_tasks.borrow_mut();
                                *pt -= 1;
                            }
                            
                            // If it's a spawned fiber without a promise, or the root fiber, 
                            // we should probably propagate the error.
                            if !f.is_spawned || f.associated_promise.is_none() {
                                return last_res;
                            }
                        }
                    }
                }
            }

            if !did_work {
                tokio::select! {
                    _ = self.async_state.notify.notified() => {},
                    _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {},
                }
            }
        }

        last_res
    }

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
        let saved_this = self.current_this.clone();
        self.program = Some(program.clone());

        // If we are already running (nested call), use synchronous execution
        if self.current_fiber.is_some() && tokio::runtime::Handle::try_current().is_ok() {
            let res = self.execute_from(0);
            self.program = saved_program;
            self.current_this = saved_this;
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
        self.current_this = saved_this;
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

    pub fn execute_from(&mut self, start_pc: usize) -> VMResult {
        self.pc = start_pc;

        loop {
            let (instruction_clone, program_clone) = {
                let program = self.program.as_ref().ok_or_else(|| RuntimeErrorWithContext {
                    error: VMRuntimeError::UndefinedVariable("No program loaded".into()),
                    loc: Location {
                        line: 0,
                        col: 0,
                        index: 0,
                    },
                    pc: self.pc,
                })?;

                if self.pc >= program.instructions.len() {
                    break;
                }

                let instruction = program.instructions[self.pc].clone();
                let program = program.clone();
                (instruction, program)
            };

            debug!("Executing instruction {}: {:?} (FP: {}, Stack: {:?})", self.pc, instruction_clone, self.fp, self.stack);

            match self.execute_instruction(&instruction_clone, &program_clone) {
                Ok(continue_execution) => {
                    if !continue_execution {
                        break;
                    }
                }
                Err(error) => {
                    let loc = *program_clone.lines.get(&self.pc).unwrap_or(&Location {
                        line: 0,
                        col: 0,
                        index: 0,
                    });
                    return Err(RuntimeErrorWithContext {
                        error,
                        loc,
                        pc: self.pc,
                    });
                }
            }

            self.pc += 1;
        }

        let result = self.stack.pop().unwrap_or(Value::null());
        Ok(result)
    }

    pub fn save_state_to_fiber(&self, fiber: &mut Fiber) {
        debug!("Saving state to fiber: stack={:?}, PC={}", self.stack, self.pc);
        fiber.stack = self.stack.clone();
        fiber.pc = self.pc;
        fiber.fp = self.fp;
        fiber.call_stack = self.call_stack.clone();
        fiber.exception_handlers = self.exception_handlers.clone();
        fiber.current_closure = self.current_closure.clone();
        fiber.current_this = self.current_this.clone();
        fiber.program = self.program.clone();
    }

    pub fn load_state_from_fiber(&mut self, fiber: &Fiber) {
        debug!("Loading state from fiber: stack={:?}, PC={}", fiber.stack, fiber.pc);
        self.stack = fiber.stack.clone();
        self.pc = fiber.pc;
        self.fp = fiber.fp;
        self.call_stack = fiber.call_stack.clone();
        self.exception_handlers = fiber.exception_handlers.clone();
        self.current_closure = fiber.current_closure.clone();
        self.current_this = fiber.current_this.clone();
        self.program = fiber.program.clone();
    }

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

                    let ast = match parse_from_source(&code) {
                        Ok(a) => a,
                        Err(e) => {
                            return Err(VMRuntimeError::UncaughtException(format!(
                                "Parse error in {}: {}",
                                path, e
                            )));
                        }
                    };

                    let module_program = compile(&code.chars().collect::<Vec<char>>(), ast);

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
                    self.stack.push(value.clone());
                } else {
                    let func_label = format!("func_{}", var_name);
                    if let Some(prog) = &self.program {
                        if let Some(symbol) = prog.syms.get(&func_label) {
                            let closure = crate::value::ObjClosure {
                                name: var_name.clone(),
                                func_symbol: symbol.clone(),
                                program: prog.clone(),
                                upvalues: Vec::new(),
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

            Instruction::LoadThis => {
                if let Some(this_val) = &self.current_this {
                    self.stack.push(this_val.clone());
                } else {
                    return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "load_this".to_string(),
                        left_type: ValueType::Null,
                        right_type: ValueType::Null,
                    }));
                }
            }

            Instruction::Store(var_name) => {
                if let Some(value) = self.stack.pop() {
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
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }

                        let call_stack_instr = Instruction::CallStack(argc);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Subtract => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.subtract(&right)?;

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }

                        let call_stack_instr = Instruction::CallStack(argc);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Multiply => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.multiply(&right)?;

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }

                        let call_stack_instr = Instruction::CallStack(argc);
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

            Instruction::Neg => {
                let val = self.stack.pop().unwrap_or(Value::null());
                let op_result = val.neg()?;

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }

                        let call_stack_instr = Instruction::CallStack(argc);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
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

                        if let Some(sym) = program.syms.get(&func_label) {
                            if *arg_count != sym.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call".to_string(),
                                    left_type: ValueType::Null,
                                    right_type: ValueType::Null,
                                }));
                            }

                            if sym.is_async {
                                let mut args = Vec::with_capacity(*arg_count);
                                for _ in 0..*arg_count {
                                    args.push(self.stack.pop().unwrap());
                                }
                                args.reverse();

                                let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                                let promise_val = Value::Promise(promise.clone());

                                let mut fiber = Fiber::new();
                                fiber.program = Some(self.program.clone().unwrap());
                                fiber.current_closure = Some(Rc::new(ObjClosure {
                                    name: func_name.clone(),
                                    func_symbol: sym.clone(),
                                    program: self.program.clone().unwrap(),
                                    upvalues: Vec::new(),
                                }));
                                fiber.fp = 0;
                                fiber.pc = sym.location as usize;
                                fiber.stack.extend(args);
                                fiber.state = FiberState::Running;
                                fiber.is_spawned = true;
                                fiber.skip_push_on_resume = true;
                                fiber.associated_promise = Some(promise.clone());

                                let fiber_rc = Rc::new(RefCell::new(fiber));
                                self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                                *self.async_state.pending_tasks.borrow_mut() += 1;
                                self.async_state.notify.notify_one();

                                self.stack.push(promise_val);
                                return Ok(true);
                            }

                            self.call_stack.push(CallFrame {
                                pc: self.pc,
                                fp: self.fp,
                                program: self.program.clone(),
                                closure: self.current_closure.clone(),
                                this_binding: self.current_this.clone(),
                                discard_return: false,
                                push_values_after_return: Vec::new(),
                            });
                            self.fp = self.stack.len() - *arg_count;
                            self.current_this = None;

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

                                    if sym.is_async {
                                        let mut args = Vec::with_capacity(*arg_count);
                                        for _ in 0..*arg_count {
                                            args.push(self.stack.pop().unwrap());
                                        }
                                        args.reverse();

                                        let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                                        let promise_val = Value::Promise(promise.clone());

                                        let mut fiber = Fiber::new();
                                        fiber.program = Some(closure.program.clone());
                                        fiber.current_closure = Some(closure.clone());
                                        fiber.fp = 0;
                                        fiber.pc = sym.location as usize;
                                        fiber.stack.extend(args);
                                        fiber.state = FiberState::Running;
                                        fiber.is_spawned = true;
                                        fiber.skip_push_on_resume = true;
                                        fiber.associated_promise = Some(promise.clone());

                                        let fiber_rc = Rc::new(RefCell::new(fiber));
                                        self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                                        *self.async_state.pending_tasks.borrow_mut() += 1;
                                        self.async_state.notify.notify_one();

                                        self.stack.push(promise_val);
                                        return Ok(true);
                                    }

                                    self.call_stack.push(CallFrame {
                                        pc: self.pc,
                                        fp: self.fp,
                                        program: self.program.clone(),
                                        closure: self.current_closure.clone(),
                                        this_binding: self.current_this.clone(),
                                        discard_return: false,
                                        push_values_after_return: Vec::new(),
                                    });
                                    self.fp = self.stack.len() - *arg_count;
                                    self.program = Some(closure.program.clone());
                                    self.current_closure = Some(closure.clone());
                                    self.current_this = None;

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
                                    let result = native_fn(self, crate::value::NativeContext { this: None, args })?;
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
                    self.current_closure = frame.closure;
                    self.current_this = frame.this_binding;
                    if !frame.discard_return {
                        self.stack.push(return_value);
                    }
                    for v in frame.push_values_after_return {
                        self.stack.push(v);
                    }
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
                if field == "length" {
                    match &obj {
                        Value::String(s) => {
                            self.stack.push(Value::int(s.chars().count() as i32));
                            return Ok(true);
                        }
                        Value::Object(table_ref) => {
                            let is_array = table_ref.borrow().metatable.as_ref().is_some_and(
                                |meta| matches!(&self.array_prototype, Value::Object(proto) if Rc::ptr_eq(meta, proto)),
                            );
                            if is_array {
                                self.stack.push(Value::int(table_ref.borrow().data.len() as i32));
                                return Ok(true);
                            }
                        }
                        _ => {}
                    }
                }

                let mut op_result = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(field)?
                } else {
                    obj.get_field_with_meta(field)?
                };

                if let OpResult::Value(Value::Null) = op_result
                    && let Value::Object(_) = obj
                {
                    op_result = self.object_prototype.get_field_with_meta(field)?;
                }

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::SetField(field) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                let op_result = obj.set_field_with_meta(field.clone(), value)?;
                match op_result {
                    OpResult::Value(_) => {}
                    OpResult::MetamethodCall(call_info) => {
                        let is_native = matches!(call_info.metamethod, Value::NativeFunction(_));
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc);
                        let res = self.execute_instruction(&call_stack_instr, program);

                        if is_native {
                            self.stack.pop();
                        } else if let Some(frame) = self.call_stack.last_mut() {
                            frame.discard_return = true;
                        }
                        return res;
                    }
                }
            }

            Instruction::GetMethod(field) => {
                let obj = self.stack.pop().unwrap_or(Value::null());
                let mut op_result = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(field)?
                } else {
                    obj.get_field_with_meta(field)?
                };

                if let OpResult::Value(Value::Null) = op_result
                    && let Value::Object(_) = obj
                {
                    op_result = self.object_prototype.get_field_with_meta(field)?;
                }

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(obj);
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        let is_native = matches!(call_info.metamethod, Value::NativeFunction(_));
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc);
                        let res = self.execute_instruction(&call_stack_instr, program);

                        if is_native {
                            self.stack.push(obj);
                        } else if let Some(frame) = self.call_stack.last_mut() {
                            frame.push_values_after_return.push(obj);
                        }
                        return res;
                    }
                }
            }

            Instruction::GetIndex => {
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                let key = index.to_string();

                let mut op_result = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(&key)?
                } else {
                    obj.get_field_with_meta(&key)?
                };

                if let OpResult::Value(Value::Null) = op_result
                    && let Value::Object(_) = obj
                {
                    op_result = self.object_prototype.get_field_with_meta(&key)?;
                }

                match op_result {
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::SetIndex => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                match obj {
                    Value::Object(table_ref) => {
                        let op_result =
                            Value::Object(table_ref.clone()).set_field_with_meta(index.to_string(), value)?;
                        match op_result {
                            OpResult::Value(_) => {}
                            OpResult::MetamethodCall(call_info) => {
                                let is_native = matches!(call_info.metamethod, Value::NativeFunction(_));
                                self.stack.push(call_info.metamethod);
                                let argc = call_info.args.len();
                                for arg in call_info.args {
                                    self.stack.push(arg);
                                }
                                let call_stack_instr = Instruction::CallStack(argc);
                                let res = self.execute_instruction(&call_stack_instr, program);

                                if is_native {
                                    self.stack.pop();
                                } else if let Some(frame) = self.call_stack.last_mut() {
                                    frame.discard_return = true;
                                }
                                return res;
                            }
                        }
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

                        if sym.is_async {
                            // Collect arguments
                            let mut args = Vec::with_capacity(*arg_count);
                            for _ in 0..*arg_count {
                                args.push(self.stack.pop().unwrap());
                            }
                            args.reverse();

                            let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                            let promise_val = Value::Promise(promise.clone());

                            let mut fiber = Fiber::new();
                            fiber.program = Some(closure.program.clone());
                            fiber.current_closure = Some(closure.clone());
                            fiber.fp = 0;
                            fiber.pc = sym.location as usize;
                            fiber.stack.extend(args);
                            fiber.state = FiberState::Running;
                            fiber.is_spawned = true;
                            fiber.skip_push_on_resume = true;
                            fiber.associated_promise = Some(promise.clone());

                            let fiber_rc = Rc::new(RefCell::new(fiber));
                            self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                            *self.async_state.pending_tasks.borrow_mut() += 1;
                            self.async_state.notify.notify_one();

                            self.stack.push(promise_val);
                            return Ok(true);
                        }

                        self.call_stack.push(CallFrame {
                            pc: self.pc,
                            fp: self.fp,
                            program: self.program.clone(),
                            closure: self.current_closure.clone(),
                            this_binding: self.current_this.clone(),
                            discard_return: false,
                            push_values_after_return: Vec::new(),
                        });
                        self.fp = self.stack.len() - *arg_count;
                        self.program = Some(closure.program.clone());
                        self.current_closure = Some(closure.clone());
                        self.current_this = None;

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

                        let result = native_fn(self, crate::value::NativeContext { this: None, args });
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

            Instruction::CallMethodStack(arg_count) => {
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(*arg_count + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallMethodStack: missing function".to_string(),
                    ))?;
                let receiver_idx = func_idx.checked_sub(1).ok_or(VMRuntimeError::StackUnderflow(
                    "CallMethodStack: missing receiver".to_string(),
                ))?;

                let receiver = self.stack.remove(receiver_idx);
                let func_val = self.stack.remove(receiver_idx);

                return match func_val {
                    Value::Fn(closure) => {
                        let sym = &closure.func_symbol;

                        if *arg_count != sym.narguments {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "call_method_stack".to_string(),
                                left_type: ValueType::Function,
                                right_type: ValueType::Null,
                            }));
                        }

                        if sym.is_async {
                            let mut args = Vec::with_capacity(*arg_count);
                            for _ in 0..*arg_count {
                                args.push(self.stack.pop().unwrap());
                            }
                            args.reverse();

                            let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                            let promise_val = Value::Promise(promise.clone());

                            let mut fiber = Fiber::new();
                            fiber.program = Some(closure.program.clone());
                            fiber.current_closure = Some(closure.clone());
                            fiber.current_this = Some(receiver);
                            fiber.fp = 0;
                            fiber.pc = sym.location as usize;
                            fiber.stack.extend(args);
                            fiber.state = FiberState::Running;
                            fiber.is_spawned = true;
                            fiber.skip_push_on_resume = true;
                            fiber.associated_promise = Some(promise.clone());

                            let fiber_rc = Rc::new(RefCell::new(fiber));
                            self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                            *self.async_state.pending_tasks.borrow_mut() += 1;
                            self.async_state.notify.notify_one();

                            self.stack.push(promise_val);
                            return Ok(true);
                        }

                        self.call_stack.push(CallFrame {
                            pc: self.pc,
                            fp: self.fp,
                            program: self.program.clone(),
                            closure: self.current_closure.clone(),
                            this_binding: self.current_this.clone(),
                            discard_return: false,
                            push_values_after_return: Vec::new(),
                        });
                        self.fp = self.stack.len() - *arg_count;
                        self.program = Some(closure.program.clone());
                        self.current_closure = Some(closure.clone());
                        self.current_this = Some(receiver);

                        self.stack.resize(self.fp + sym.nlocals, Value::null());
                        self.pc = (sym.location as usize) - 1;
                        Ok(true)
                    }
                    Value::NativeFunction(native_fn) => {
                        let start_index =
                            self.stack
                                .len()
                                .checked_sub(*arg_count)
                                .ok_or(VMRuntimeError::StackUnderflow(
                                    "CallMethodStack native: missing args".into(),
                                ))?;
                        let args: Vec<Value> = self.stack.drain(start_index..).collect();

                        let result = native_fn(self, crate::value::NativeContext { this: Some(receiver), args });
                        let val = result?;

                        self.stack.push(val);
                        Ok(true)
                    }
                    _ => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "call_method_stack".to_string(),
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
                    self.call_stack.truncate(handler.call_stack_len);
                    self.program = handler.program;
                    self.current_closure = handler.closure;
                    self.current_this = handler.this_binding;
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
                    call_stack_len: self.call_stack.len(),
                    program: self.program.clone(),
                    closure: self.current_closure.clone(),
                    this_binding: self.current_this.clone(),
                });
            }

            Instruction::PopExceptionHandler => {
                self.exception_handlers.pop();
            }
            Instruction::AsyncCallStack(narguments) => {
                let mut args = Vec::with_capacity(*narguments);
                for _ in 0..*narguments {
                    args.push(self.stack.pop().expect("Stack underflow for async call arguments"));
                }
                args.reverse();

                let callee = self.stack.pop().expect("Stack underflow for async call callee");
                
                match callee {
                    Value::Fn(closure) => {
                        let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                        let promise_val = Value::Promise(promise.clone());
                        
                        let mut fiber = Fiber::new();
                        fiber.program = Some(closure.program.clone());
                        fiber.current_closure = Some(closure.clone());
                        fiber.fp = 0;
                        fiber.stack.extend(args);
                        fiber.state = FiberState::Running;
                        fiber.is_spawned = true;
                        fiber.associated_promise = Some(promise.clone());
                        
                        let fiber_rc = Rc::new(RefCell::new(fiber));
                        self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                        *self.async_state.pending_tasks.borrow_mut() += 1;
                        
                        self.stack.push(promise_val);
                    }
                    _ => return Err(VMRuntimeError::ValueError(ValueError::CallNonFunction(callee.get_type()))),
                }
            }
            Instruction::Yield => {
                let promise_val = self.stack.pop().expect("Stack underflow for yield");

                if let Value::Promise(promise_rc) = promise_val {
                    let state = promise_rc.borrow().state.clone();
                    match state {
                        crate::promise::PromiseState::Fulfilled(val) => {
                            self.stack.push(val);
                        }
                        crate::promise::PromiseState::Rejected(reason) => {
                            if let Some(handler) = self.exception_handlers.pop() {
                                self.stack.truncate(handler.stack_size);
                                self.fp = handler.fp;
                                self.call_stack.truncate(handler.call_stack_len);
                                self.program = handler.program;
                                self.current_closure = handler.closure;
                                self.current_this = handler.this_binding;
                                self.stack.push(reason.clone());

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
                            return Err(VMRuntimeError::UncaughtException(reason.to_string()));
                        }
                        crate::promise::PromiseState::Pending => {
                            let current_fiber_rc = self.current_fiber.clone().expect("Yield outside of fiber");
                            let mut current_fiber = current_fiber_rc.borrow_mut();

                            // Save state BEFORE yielding
                            // Increment PC so we resume at the NEXT instruction
                            self.pc += 1;
                            self.save_state_to_fiber(&mut current_fiber);
                            current_fiber.state = FiberState::Suspended;

                            promise_rc.borrow_mut().add_reaction(crate::promise::Reaction::ResumeFiber(current_fiber_rc.clone()));
                            return Err(VMRuntimeError::Yield);
                        }
                    }
                } else {
                    // Not a promise, just push it back (like Promise.resolve(val))
                    self.stack.push(promise_val);
                }
            }
        }

        Ok(true)
    }
}
