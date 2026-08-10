use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;
use tracing::debug;

use crate::tokenizer::Location;
use crate::value::{ObjClosure, ObjUpvalue, OpResult, UpvalueState, Value, ValueError, ValueType};
use crate::vm::fiber::{CallFrame, ExceptionHandler, WANT_ALL};
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
                                    let error_value = Value::string(err.to_string());
                                    if let Some(handler) = self.exception_handlers.pop() {
                                        self.stack.truncate(handler.stack_size);
                                        self.fp = handler.fp;
                                        self.stack.push(error_value);
                                        if let Some(target) =
                                            self.program.as_ref().and_then(|p| p.syms.get(&handler.catch_label))
                                        {
                                            self.pc = (target.location as usize) - 1;
                                            self.pc = self.pc.saturating_add(1);
                                        } else {
                                            // No handler, propagate error
                                        }
                                    }
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

    /// 收集变长函数的额外实参为一个数组 Table（必须在 resize 帧之前调用）
    fn capture_varargs(&mut self, argc: usize, narguments: usize) -> Rc<RefCell<crate::value::Table>> {
        let base = self.fp + narguments;
        let count = argc.saturating_sub(narguments);
        let mut data = IndexMap::new();
        for i in 0..count {
            let v = self.stack.get(base + i).cloned().unwrap_or(Value::Null);
            data.insert(i.to_string(), v);
        }
        Rc::new(RefCell::new(crate::value::Table { data, metatable: None }))
    }

    /// 调整 `base` 之上的返回值数量到期望个数 `want`（`WANT_ALL` 表示保持原样）。
    /// 过多则截断，过少则用 nil 补齐。
    fn adjust_return_values(&mut self, base: usize, want: usize) {
        if want == WANT_ALL {
            return;
        }
        let count = self.stack.len().saturating_sub(base);
        if count > want {
            self.stack.truncate(base + want);
        } else if count < want {
            self.stack.resize(base + want, Value::null());
        }
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

    /// 从栈调用一个值（闭包 / 原生函数 / 带 __call 的对象）
    fn call_value(
        &mut self,
        func_val: Value,
        arg_count: usize,
        want: usize,
        program: &Program,
    ) -> Result<bool, VMRuntimeError> {
        return match func_val {
            Value::Fn(closure) => {
                let sym = &closure.func_symbol;
                if !sym.is_vararg && arg_count != sym.narguments {
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
                    discard_return: false,
                    push_values_after_return: Vec::new(),
                    want_return: want,
                    varargs: None,
                });
                self.fp = self.stack.len() - arg_count;
                self.program = Some(closure.program.clone());
                self.current_closure = Some(closure.clone());
                debug!(
                    "[VM DEBUG] Call Fn: {}, new current_closure: Some({})",
                    closure.name, closure.name
                );

                if sym.is_vararg && arg_count > sym.narguments {
                    let varargs = self.capture_varargs(arg_count, sym.narguments);
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.varargs = Some(varargs);
                    }
                }
                self.stack.resize(self.fp + sym.nlocals, Value::null());
                self.pc = (sym.location as usize) - 1;
                Ok(true)
            }
            Value::NativeFunction(native_fn) => {
                let base = self
                    .stack
                    .len()
                    .checked_sub(arg_count)
                    .ok_or(VMRuntimeError::StackUnderflow("CallStack native: missing args".into()))?;
                let args: Vec<Value> = self.stack.drain(base..).collect();

                let result = native_fn(self, args);
                let val = result?;

                self.stack.push(val);
                self.adjust_return_values(base, want);
                Ok(true)
            }
            Value::Object(obj_rc) => {
                // __call 元方法：obj(...) -> __call(obj, ...)
                let obj_val = Value::Object(obj_rc.clone());
                let call_mm = obj_val.get_metamethod_from_object("__call").ok_or_else(|| {
                    VMRuntimeError::ValueError(ValueError::InvalidOperation {
                        operator: "call_stack".to_string(),
                        left_type: ValueType::Object,
                        right_type: ValueType::Null,
                    })
                })?;
                self.stack.push(call_mm);
                self.stack.push(obj_val);
                // 剩余参数已在栈上
                let call_stack_instr = Instruction::CallStack(arg_count + 1, want);
                return self.execute_instruction(&call_stack_instr, program);
            }
            _ => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "call_stack".to_string(),
                left_type: func_val.get_type(),
                right_type: ValueType::Null,
            })),
        };
    }

    /// 从指定PC开始执行程序
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
                        break;
                    }

                    if let Some(handler) = self.exception_handlers.pop() {
                        self.stack.truncate(handler.stack_size);
                        self.fp = handler.fp;
                        // 异常传播必须清理高于 handler.fp 的调用帧（错误可能发生在嵌套函数调用内），
                        // 否则这些残留帧会在函数末尾被 Return/ReturnAll 恢复，导致后续代码重复执行。
                        // 恢复后的 current_closure 取最后一个被弹出的帧（最浅被跳过的调用）的 closure，
                        // 即 try 块所在函数的闭包；若没有弹出任何帧（错误直接发生在 try 体内），
                        // current_closure 保持当前值即可。
                        let mut restored_closure = None;
                        while let Some(frame) = self.call_stack.last() {
                            if frame.fp >= handler.fp {
                                let frame = self.call_stack.pop().unwrap();
                                restored_closure = frame.closure.clone();
                            } else {
                                break;
                            }
                        }
                        if let Some(c) = restored_closure {
                            self.current_closure = Some(c);
                        }
                        let error_msg = match &error {
                            VMRuntimeError::UncaughtException(msg) => msg.clone(),
                            _ => error.to_string(),
                        };
                        self.stack.push(Value::string(error_msg));
                        if let Some(target) = program_clone.syms.get(&handler.catch_label) {
                            self.pc = target.location as usize;
                            continue;
                        }
                    }

                    let loc = *program_clone.lines.get(&self.pc).unwrap_or(&Location {
                        line: 0,
                        col: 0,
                        index: 0,
                    });
                    debug!(
                        "Execution error at PC {} (Line {}:{}): {}",
                        self.pc, loc.line, loc.col, error
                    );
                    return Err(RuntimeErrorWithContext {
                        error,
                        loc,
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

            Instruction::BuildArrayVariadic(fixed) => {
                let vararg_table = self.call_stack.last().and_then(|f| f.varargs.clone());
                let vcount = vararg_table.as_ref().map(|t| t.borrow().data.len()).unwrap_or(0);
                let start = self.stack.len().saturating_sub(*fixed);
                let mut data = IndexMap::new();
                for i in 0..*fixed {
                    let val = self.stack.get(start + i).cloned().unwrap_or(Value::Null);
                    data.insert(i.to_string(), val);
                }
                for i in 0..vcount {
                    let val = vararg_table
                        .as_ref()
                        .and_then(|t| t.borrow().data.get(&i.to_string()).cloned())
                        .unwrap_or(Value::Null);
                    data.insert((fixed + i).to_string(), val);
                }
                self.stack.truncate(start);
                let mut table = crate::value::Table { data, metatable: None };
                if let Value::Object(proto_table) = &self.array_prototype {
                    table.metatable = Some(proto_table.clone());
                }
                self.stack.push(Value::Object(Rc::new(RefCell::new(table))));
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
                    OpResult::Value(value) => {
                        self.stack.push(value);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }

                        let call_stack_instr = Instruction::CallStack(argc, 1);
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

                        let call_stack_instr = Instruction::CallStack(argc, 1);
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

                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Divide => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.divide(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Modulo => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.modulo(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Concat => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                // 检查 __concat 元方法
                if !matches!(left, Value::String(_)) || !matches!(right, Value::String(_)) {
                    let metamethod = left
                        .get_metamethod_from_object("__concat")
                        .or_else(|| right.get_metamethod_from_object("__concat"));
                    if let Some(mm) = metamethod {
                        self.stack.push(mm);
                        self.stack.push(left.clone());
                        self.stack.push(right.clone());
                        let call_stack_instr = Instruction::CallStack(2, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
                let result = Value::string(format!("{}{}", left, right));
                self.stack.push(result);
            }

            Instruction::FloorDiv => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.floor_div(&right)?;
                self.stack.push(result);
            }

            Instruction::Pow => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.pow(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::Equal => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.equal(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::NotEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.not_equal(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::LessThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.less_than(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::LessThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.less_equal(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::GreaterThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.greater_than(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
            }

            Instruction::GreaterThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let op_result = left.greater_equal(&right)?;
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
                        return self.execute_instruction(&call_stack_instr, program);
                    }
                }
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

            Instruction::Length => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let op_result = value.len()?;
                match op_result {
                    OpResult::Value(v) => {
                        self.stack.push(v);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc, 1);
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

            Instruction::Call(func_name, arg_count, want) => {
                return match func_name.as_str() {
                    "set_meta" | "setmetatable" => {
                        if *arg_count != 2 {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: func_name.to_string(),
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
                    "get_meta" | "getmetatable" => {
                        if *arg_count != 1 {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: func_name.to_string(),
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
                            if !sym.is_vararg && *arg_count != sym.narguments {
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
                                discard_return: false,
                                push_values_after_return: Vec::new(),
                                want_return: *want,
                                varargs: None,
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

                            if sym.is_vararg && *arg_count > sym.narguments {
                                let varargs = self.capture_varargs(*arg_count, sym.narguments);
                                if let Some(frame) = self.call_stack.last_mut() {
                                    frame.varargs = Some(varargs);
                                }
                            }
                            self.stack.resize(self.fp + sym.nlocals, Value::null());
                            self.pc = (sym.location as usize) - 1;
                            Ok(true)
                        } else if let Some(val) = self.variables.get(func_name).cloned() {
                            // Variable lookup
                            match val {
                                Value::Fn(closure) => {
                                    let sym = &closure.func_symbol;
                                    if !sym.is_vararg && *arg_count != sym.narguments {
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
                                        discard_return: false,
                                        push_values_after_return: Vec::new(),
                                        want_return: *want,
                                        varargs: None,
                                    });
                                    self.fp = self.stack.len() - *arg_count;
                                    self.program = Some(closure.program.clone());
                                    self.current_closure = Some(closure.clone());

                                    if sym.is_vararg && *arg_count > sym.narguments {
                                        let varargs = self.capture_varargs(*arg_count, sym.narguments);
                                        if let Some(frame) = self.call_stack.last_mut() {
                                            frame.varargs = Some(varargs);
                                        }
                                    }
                                    self.stack.resize(self.fp + sym.nlocals, Value::null());
                                    self.pc = (sym.location as usize) - 1;
                                    Ok(true)
                                }
                                Value::NativeFunction(native_fn) => {
                                    let base = self
                                        .stack
                                        .len()
                                        .checked_sub(*arg_count)
                                        .ok_or(VMRuntimeError::StackUnderflow("Native call missing args".into()))?;
                                    let args: Vec<Value> = self.stack.drain(base..).collect();
                                    let result = native_fn(self, args)?;
                                    self.stack.push(result);
                                    self.adjust_return_values(base, *want);
                                    Ok(true)
                                }
                                other => {
                                    // __call 元方法
                                    let obj_val = other.clone();
                                    if let Value::Object(_) = obj_val {
                                        let call_mm = obj_val.get_metamethod_from_object("__call");
                                        if let Some(call_mm) = call_mm {
                                            // 弹出参数，重新以 CallStack 方式调用 __call(obj, ...)
                                            let base =
                                                self.stack.len().checked_sub(*arg_count).ok_or(
                                                    VMRuntimeError::StackUnderflow("__call missing args".into()),
                                                )?;
                                            let args: Vec<Value> = self.stack.drain(base..).collect();
                                            self.stack.push(call_mm);
                                            self.stack.push(obj_val);
                                            for a in args {
                                                self.stack.push(a);
                                            }
                                            let call_stack_instr = Instruction::CallStack(*arg_count + 1, *want);
                                            return self.execute_instruction(&call_stack_instr, program);
                                        }
                                    }
                                    Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                        operator: "call".to_string(),
                                        left_type: other.get_type(),
                                        right_type: ValueType::Null,
                                    }))
                                }
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
                    let caller_base = self.fp;
                    let want = frame.want_return;
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
                    if !frame.discard_return {
                        self.stack.push(return_value);
                        self.adjust_return_values(caller_base, want);
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

            Instruction::ReturnAll => {
                let boundary = match &self.current_closure {
                    Some(closure) => self.fp + closure.func_symbol.nlocals,
                    None => self.fp,
                };
                let values: Vec<Value> = self.stack[boundary..].to_vec();
                self.close_upvalues(self.fp);

                return if let Some(frame) = self.call_stack.pop() {
                    let caller_base = self.fp;
                    let want = frame.want_return;
                    self.stack.truncate(self.fp);
                    self.pc = frame.pc;
                    self.fp = frame.fp;
                    if let Some(prog) = frame.program {
                        self.program = Some(prog);
                    }
                    self.current_closure = frame.closure;
                    if !frame.discard_return {
                        for v in values {
                            self.stack.push(v);
                        }
                        self.adjust_return_values(caller_base, want);
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
                        for v in values {
                            self.stack.push(v);
                        }
                        Ok(false)
                    } else {
                        for v in values {
                            self.stack.push(v);
                        }
                        Ok(false)
                    }
                } else {
                    for v in values {
                        self.stack.push(v);
                    }
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

            Instruction::Vararg(want) => {
                // 读取当前调用帧的变长参数数组
                let table = self.call_stack.last().and_then(|f| f.varargs.clone());
                let count = table.as_ref().map(|t| t.borrow().data.len()).unwrap_or(0);
                let want = *want;
                if want == WANT_ALL {
                    for i in 0..count {
                        let v = table
                            .as_ref()
                            .and_then(|t| t.borrow().data.get(&i.to_string()).cloned())
                            .unwrap_or(Value::Null);
                        self.stack.push(v);
                    }
                } else {
                    for i in 0..want.min(count) {
                        let v = table
                            .as_ref()
                            .and_then(|t| t.borrow().data.get(&i.to_string()).cloned())
                            .unwrap_or(Value::Null);
                        self.stack.push(v);
                    }
                    for _ in 0..want.saturating_sub(count) {
                        self.stack.push(Value::null());
                    }
                }
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
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
                } else if let Value::Coroutine(_) = obj {
                    if let Some(co_obj) = self.variables.get("coroutine") {
                        co_obj.get_field_with_meta(field)?
                    } else {
                        OpResult::Value(Value::Null)
                    }
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
                        self.stack.push(obj);
                    }
                    OpResult::MetamethodCall(call_info) => {
                        let is_native = matches!(call_info.metamethod, Value::NativeFunction(_));
                        self.stack.push(call_info.metamethod);
                        let argc = call_info.args.len();
                        for arg in call_info.args {
                            self.stack.push(arg);
                        }
                        let call_stack_instr = Instruction::CallStack(argc, 1);
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
                        let call_stack_instr = Instruction::CallStack(argc, 1);
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
                                let call_stack_instr = Instruction::CallStack(argc, 1);
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

            Instruction::CallStack(arg_count, want) => {
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(*arg_count + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallStack: missing function".to_string(),
                    ))?;

                let func_val = self.stack.remove(func_idx);
                return self.call_value(func_val, *arg_count, *want, program);
            }

            Instruction::CallStackVararg(fixed, want) => {
                let vcount = self
                    .call_stack
                    .last()
                    .and_then(|f| f.varargs.clone())
                    .map(|t| t.borrow().data.len())
                    .unwrap_or(0);
                let argc = fixed + vcount;
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(argc + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallStackVararg: missing function".to_string(),
                    ))?;
                let func_val = self.stack.remove(func_idx);
                return self.call_value(func_val, argc, *want, program);
            }

            Instruction::CallVararg(func_name, fixed, want) => {
                let vcount = self
                    .call_stack
                    .last()
                    .and_then(|f| f.varargs.clone())
                    .map(|t| t.borrow().data.len())
                    .unwrap_or(0);
                let argc = fixed + vcount;
                let func_label = format!("func_{}", func_name);
                return if let Some(sym) = program.syms.get(&func_label) {
                    if !sym.is_vararg && argc != sym.narguments {
                        return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                            operator: "call_vararg".to_string(),
                            left_type: ValueType::Null,
                            right_type: ValueType::Null,
                        }));
                    }
                    self.call_stack.push(CallFrame {
                        pc: self.pc,
                        fp: self.fp,
                        program: self.program.clone(),
                        closure: self.current_closure.clone(),
                        discard_return: false,
                        push_values_after_return: Vec::new(),
                        want_return: *want,
                        varargs: None,
                    });
                    self.fp = self.stack.len() - argc;
                    let closure = ObjClosure {
                        name: func_name.clone(),
                        func_symbol: sym.clone(),
                        program: self.program.clone().unwrap(),
                        upvalues: Vec::new(),
                    };
                    self.current_closure = Some(Rc::new(closure));
                    if sym.is_vararg && argc > sym.narguments {
                        let v = self.capture_varargs(argc, sym.narguments);
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.varargs = Some(v);
                        }
                    }
                    self.stack.resize(self.fp + sym.nlocals, Value::null());
                    self.pc = (sym.location as usize) - 1;
                    Ok(true)
                } else if let Some(val) = self.variables.get(func_name.as_str()).cloned() {
                    return self.call_value(val, argc, *want, program);
                } else {
                    Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                };
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
