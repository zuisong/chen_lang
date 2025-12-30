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
use crate::value::{Value, ValueError, ValueType};
use crate::vm::fiber::ExceptionHandler;
use crate::vm::{Fiber, FiberState, Instruction, Program, RuntimeErrorWithContext, VM, VMResult, VMRuntimeError};

impl VM {
    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        self.execute_rc(Rc::new(program.clone()))
    }

    pub fn execute_rc(&mut self, program: Rc<Program>) -> VMResult {
        let saved_program = self.program.clone();
        self.program = Some(program.clone());
        let res = self.execute_from(0);
        self.program = saved_program; // Restore previous program
        res
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
    }

    pub fn load_state_from_fiber(&mut self, fiber: &Fiber) {
        self.stack = fiber.stack.clone();
        self.pc = fiber.pc;
        self.fp = fiber.fp;
        self.call_stack = fiber.call_stack.clone();
        self.exception_handlers = fiber.exception_handlers.clone();
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
                    metatable: None,
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
                    if program.syms.contains_key(&func_label) {
                        if let Some(prog) = &self.program {
                            self.stack.push(Value::Closure(var_name.clone(), prog.clone()));
                        } else {
                            self.stack.push(Value::Function(var_name.clone()));
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

            Instruction::Call(func_name, arg_count) => match func_name.as_str() {
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
                }
                _ => {
                    let func_label = format!("func_{}", func_name);

                    let target_info = if let Some(sym) = program.syms.get(&func_label) {
                        Some((sym.clone(), self.program.clone()))
                    } else if let Some(val) = self.variables.get(func_name) {
                        match val {
                            Value::Closure(name, prog) => {
                                let label = format!("func_{}", name);
                                prog.syms.get(&label).map(|sym| (sym.clone(), Some(prog.clone())))
                            }
                            Value::Function(name) => {
                                let label = format!("func_{}", name);
                                if let Some(sym) = program.syms.get(&label) {
                                    Some((sym.clone(), self.program.clone()))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };

                    if target_info.is_none() {
                        let native_fn_opt = self.variables.get(func_name).cloned();
                        if let Some(Value::NativeFunction(native_fn)) = native_fn_opt {
                            let args_start = self
                                .stack
                                .len()
                                .checked_sub(*arg_count)
                                .ok_or(VMRuntimeError::StackUnderflow("Native call missing args".into()))?;
                            let args: Vec<Value> = self.stack.drain(args_start..).collect();
                            let result = native_fn(self, args)?;
                            self.stack.push(result);
                            return Ok(true);
                        }
                    }

                    if let Some((target_symbol, target_program_opt)) = target_info {
                        if *arg_count != target_symbol.narguments {
                            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                operator: "call".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }));
                        }

                        self.call_stack.push((self.pc, self.fp, self.program.clone()));
                        self.fp = self.stack.len() - *arg_count;

                        if let Some(prog) = target_program_opt {
                            self.program = Some(prog);
                        }

                        self.stack.resize(self.fp + target_symbol.nlocals, Value::null());
                        self.pc = (target_symbol.location as usize) - 1;
                        return Ok(true);
                    } else {
                        return Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)));
                    }
                }
            },
            Instruction::Return => {
                let return_value = self.stack.pop().unwrap_or(Value::null());

                return if let Some((return_pc, old_fp, old_prog)) = self.call_stack.pop() {
                    self.stack.truncate(self.fp);
                    self.pc = return_pc;
                    self.fp = old_fp;
                    if let Some(prog) = old_prog {
                        self.program = Some(prog);
                    }
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
                        Ok(true)
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
                    Value::Closure(func_name, prog_rc) => {
                        let func_label = format!("func_{}", func_name);
                        if let Some(target_symbol) = prog_rc.syms.get(&func_label) {
                            if *arg_count != target_symbol.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call_stack".to_string(),
                                    left_type: ValueType::Function,
                                    right_type: ValueType::Null,
                                }));
                            }

                            self.call_stack.push((self.pc, self.fp, self.program.clone()));
                            self.fp = self.stack.len() - *arg_count;
                            self.program = Some(prog_rc.clone());

                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());
                            self.pc = (target_symbol.location as usize) - 1;
                            Ok(true)
                        } else {
                            Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                        }
                    }
                    Value::Function(func_name) => {
                        let func_label = format!("func_{}", func_name);
                        if let Some(target_symbol) = program.syms.get(&func_label) {
                            if *arg_count != target_symbol.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call_stack".to_string(),
                                    left_type: ValueType::Function,
                                    right_type: ValueType::Null,
                                }));
                            }

                            self.call_stack.push((self.pc, self.fp, self.program.clone()));
                            self.fp = self.stack.len() - *arg_count;
                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());
                            self.pc = (target_symbol.location as usize) - 1;
                            Ok(true)
                        } else {
                            Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                        }
                    }
                    Value::NativeFunction(native_fn) => {
                        let start_index = self
                            .stack
                            .len()
                            .checked_sub(*arg_count)
                            .ok_or(VMRuntimeError::StackUnderflow("CallStack native: missing args".into()))?;
                        let args: Vec<Value> = self.stack.drain(start_index..).collect();
                        let result = native_fn(self, args)?;
                        self.stack.push(result);
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
