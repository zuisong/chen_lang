use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::rc::Rc;

use indexmap::IndexMap;
use jiff::Timestamp;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use thiserror::Error;
use tracing::debug;

mod native_array_prototype;
mod native_date;
mod native_json;
mod native_string_prototype;

use crate::value::{NativeFnType, Value, ValueError, ValueType};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub location: i32,
    pub narguments: usize,
    pub nlocals: usize,
}

/// 指令集 - 简化后的统一指令
#[derive(Debug, Clone)]
pub enum Instruction {
    // 栈操作
    Push(Value), // 推入常量值
    Pop,         // 弹出栈顶
    Dup,         // 复制栈顶元素

    // 变量操作
    Load(String),  // 加载变量
    Store(String), // 存储到变量

    // 运算操作（统一接口）
    Add,      // 加法
    Subtract, // 减法
    Multiply, // 乘法
    Divide,   // 除法
    Modulo,   // 取模

    // 比较操作
    Equal,              // 等于
    NotEqual,           // 不等于
    LessThan,           // 小于
    LessThanOrEqual,    // 小于等于
    GreaterThan,        // 大于
    GreaterThanOrEqual, // 大于等于

    // 逻辑操作
    And, // 逻辑与
    Or,  // 逻辑或
    Not, // 逻辑非

    // 控制流
    Jump(String),        // 无条件跳转
    JumpIfFalse(String), // 条件跳转（栈顶为假时）
    JumpIfTrue(String),  // 条件跳转（栈顶为真时）

    // 函数调用
    Call(String, usize), // 调用函数（函数名，参数个数）
    Return,              // 返回

    // 标签（用于跳转目标）
    Label(String), // 标签定义

    // New Scope-related Instructions
    DupPlusFP(i32),
    MovePlusFP(usize),

    // Object operations
    NewObject,         // 创建空对象
    SetField(String),  // 设置对象字段: obj[field] = value (弹出 value, obj)
    GetField(String),  // 获取对象字段: obj[field] (弹出 obj, 压入 value)
    GetMethod(String), // 获取方法: obj.method (弹出 obj, 压入 func, obj) - 用于方法调用优化
    SetIndex,          // 设置对象索引: obj[index] = value (弹出 value, index, obj)
    GetIndex,          // 获取对象索引: obj[index] (弹出 index, obj, 压入 value)

    // Call function from stack
    CallStack(usize), // Call function at stack[top-n-1], with n args

    // Array creation (Syntactic sugar for object with numeric keys)
    BuildArray(usize),

    // Exception handling
    Throw,                        // Throw an exception
    PushExceptionHandler(String), // Push exception handler (catch label)
    PopExceptionHandler,          // Pop exception handler
}

/// 程序表示
#[derive(Debug, Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub syms: IndexMap<String, Symbol>, // 符号表
    pub lines: IndexMap<usize, u32>,    // 行号映射 (Instruction Index -> Line Number)
}

/// VM 运行时错误
#[derive(Error, Debug, Clone)]
pub enum VMRuntimeError {
    #[error("Stack underflow: {0}")]
    StackUnderflow(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),
    #[error(transparent)]
    ValueError(#[from] ValueError),
    #[error("Uncaught exception: {0}")]
    UncaughtException(String),
}

/// 包含上下文信息的运行时错误
#[derive(Debug, Error)]
pub struct RuntimeErrorWithContext {
    pub error: VMRuntimeError,
    pub line: u32,
    pub pc: usize,
}
impl Display for RuntimeErrorWithContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

/// VM执行结果
pub type VMResult = Result<Value, RuntimeErrorWithContext>;

/// Exception handler entry
#[derive(Debug, Clone)]
struct ExceptionHandler {
    catch_label: String,
    stack_size: usize,
    fp: usize,
}

/// 虚拟机实现
pub struct VM {
    stack: Vec<Value>,                         // 操作数栈
    variables: IndexMap<String, Value>,        // 全局变量存储
    pc: usize,                                 // 程序计数器
    fp: usize,                                 // 帧指针
    call_stack: Vec<(usize, usize)>,           // 调用栈（保存返回地址, 旧fp）
    stdout: Box<dyn Write>,                    // 标准输出
    array_prototype: Value,                    // 数组原型对象
    string_prototype: Value,                   // 字符串原型对象
    exception_handlers: Vec<ExceptionHandler>, // 异常处理器栈
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

use native_date::create_date_object;
use native_json::create_json_object;

use crate::vm::native_array_prototype::create_array_prototype;
use crate::vm::native_string_prototype::create_string_prototype;

impl VM {
    pub fn new() -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        variables.insert("Date".to_string(), create_date_object());
        variables.insert("JSON".to_string(), create_json_object());
        VM {
            stack: Vec::new(),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: Box::new(std::io::stdout()),
            array_prototype: create_array_prototype(),
            string_prototype: create_string_prototype(),
            exception_handlers: Vec::new(),
        }
    }

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        variables.insert("Date".to_string(), create_date_object());
        variables.insert("JSON".to_string(), create_json_object());
        VM {
            stack: Vec::new(),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: writer,
            array_prototype: create_array_prototype(),
            string_prototype: create_string_prototype(),
            exception_handlers: Vec::new(),
        }
    }

    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        self.execute_from(program, 0)
    }

    /// 从指定PC开始执行程序
    pub fn execute_from(&mut self, program: &Program, start_pc: usize) -> VMResult {
        self.pc = start_pc;

        while self.pc < program.instructions.len() {
            let instruction = &program.instructions[self.pc];
            debug!("Executing instruction {}: {:?}", self.pc, instruction);

            match self.execute_instruction(instruction, program) {
                Ok(continue_execution) => {
                    if !continue_execution {
                        debug!("Execution stopped at PC {}", self.pc);
                        break;
                    }
                }
                Err(error) => {
                    let line = *program.lines.get(&self.pc).unwrap_or(&0);
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

        // 返回栈顶值或null
        let result = self.stack.pop().unwrap_or(Value::null());
        Ok(result)
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
                    metatable: None,
                };

                // Pop count elements
                let start_index = self
                    .stack
                    .len()
                    .checked_sub(*count)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "Stack underflow during array creation".to_string(),
                    ))?;

                for i in 0..*count {
                    // Stack: [..., e0, e1, e2]
                    // start_index points to e0 (e.g. len - 3, i=0 gives index len-3)
                    let val = self.stack[start_index + i].clone();
                    // Use numeric strings keys "0", "1", ...
                    table.data.insert(i.to_string(), val);
                }

                self.stack.truncate(start_index);
                // Set Array Prototype
                let mut table_ref = table;
                // We need to clone the prototype's table reference if it's an object
                if let Value::Object(proto_table) = &self.array_prototype {
                    table_ref.metatable = Some(proto_table.clone());
                }

                self.stack
                    .push(Value::Object(Rc::new(std::cell::RefCell::new(table_ref))));
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
                    debug!("All variables in VM: {:?}", self.variables);
                    self.stack.push(value.clone());
                } else {
                    // Check if it is a function
                    let func_label = format!("func_{}", var_name);
                    if program.syms.contains_key(&func_label) {
                        self.stack.push(Value::Function(var_name.clone()));
                    } else {
                        debug!(
                            "Variable {} not found! Available variables: {:?}",
                            var_name, self.variables
                        );
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
                        self.stack.push(call_info.metamethod); // Push metamethod first
                        self.stack.push(call_info.left); // Then left arg
                        self.stack.push(call_info.right); // Then right arg

                        // CallStack expects func, arg1, arg2, so the metamethod is at func_idx.
                        // The arguments (left, right) are after it, so arg_count is 2.

                        let call_stack_instr = Instruction::CallStack(2);
                        // Do NOT advance PC here. CallStack uses current PC as return address.
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
                        self.stack.push(call_info.metamethod); // Push metamethod first
                        self.stack.push(call_info.left); // Then left arg
                        self.stack.push(call_info.right); // Then right arg

                        // CallStack expects func, arg1, arg2, so the metamethod is at func_idx.
                        // The arguments (left, right) are after it, so arg_count is 2.

                        let call_stack_instr = Instruction::CallStack(2);
                        // Do NOT advance PC here. CallStack uses current PC as return address.
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
                        self.stack.push(call_info.metamethod); // Push metamethod first
                        self.stack.push(call_info.left); // Then left arg
                        self.stack.push(call_info.right); // Then right arg

                        let call_stack_instr = Instruction::CallStack(2);
                        // Do NOT advance PC here.
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
                    Ok(true) // 继续执行，但PC已更新
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
                // 处理内置函数
                match func_name.as_str() {
                    "print" => {
                        // print函数不换行，按参数顺序输出
                        let mut values = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(value) = self.stack.pop() {
                                values.push(value);
                            }
                        }
                        // 反向输出以保持正确顺序
                        for value in values.iter().rev() {
                            write!(self.stdout, "{}", value).unwrap();
                        }
                        self.stdout.flush().unwrap();
                        self.stack.push(Value::null()); // 返回null
                    }
                    "println" => {
                        let mut values = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(value) = self.stack.pop() {
                                values.push(value);
                            }
                        }
                        // 反向输出以保持正确顺序
                        for value in values.iter().rev() {
                            write!(self.stdout, "{}", value).unwrap();
                        }
                        writeln!(self.stdout).unwrap();
                        self.stack.push(Value::null());
                    }
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
                        debug!(
                            "set_meta called: obj_type={:?}, metatable_type={:?}",
                            obj.get_type(),
                            metatable.get_type()
                        );
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
                        // 处理用户定义的函数
                        let func_label = format!("func_{}", func_name);
                        debug!(
                            "Calling function {} (label: {}), arg_count: {}",
                            func_name, func_label, *arg_count
                        );

                        // Try direct symbol lookup, then variable lookup
                        let target_symbol = if let Some(sym) = program.syms.get(&func_label) {
                            Some(sym)
                        } else if let Some(Value::Function(real_name)) = self.variables.get(func_name) {
                            let real_label = format!("func_{}", real_name);
                            program.syms.get(&real_label)
                        } else {
                            None
                        };

                        // Check for NativeFunction in variables
                        if target_symbol.is_none()
                            && let Some(Value::NativeFunction(native_fn)) = self.variables.get(func_name)
                        {
                            // Native Call logic
                            let args_start = self
                                .stack
                                .len()
                                .checked_sub(*arg_count)
                                .ok_or(VMRuntimeError::StackUnderflow("Native call missing args".into()))?;
                            let args: Vec<Value> = self.stack.drain(args_start..).collect();
                            let result = native_fn(args)?;
                            self.stack.push(result);
                            return Ok(true);
                        }

                        return if let Some(target_symbol) = target_symbol {
                            if *arg_count != target_symbol.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call".to_string(),
                                    left_type: ValueType::Null,
                                    right_type: ValueType::Null,
                                }));
                            }

                            // 1. 保存返回地址和旧fp
                            self.call_stack.push((self.pc, self.fp));

                            // 2. 设置新fp
                            self.fp = self.stack.len() - *arg_count;

                            // 3. 为局部变量分配空间
                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());

                            // 4. 跳转到函数
                            self.pc = (target_symbol.location as usize) - 1;
                            Ok(true)
                        } else {
                            debug!("Function label {} not found in {:?}", func_label, program.syms);
                            Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                        };
                    }
                }
            }
            Instruction::Return => {
                // 1. Pop the return value
                let return_value = self.stack.pop().unwrap_or(Value::null());

                // 2. Pop the call frame
                return if let Some((return_pc, old_fp)) = self.call_stack.pop() {
                    // 3. Destroy the current stack frame
                    self.stack.truncate(self.fp);

                    // 4. Restore pc and fp
                    self.pc = return_pc;
                    self.fp = old_fp;

                    // 5. Push return value onto the caller's stack
                    self.stack.push(return_value);

                    Ok(true)
                } else {
                    // No more call frames, program is ending
                    debug!("Program end (no more call stack)");
                    Ok(false) // 停止执行
                };
            }

            Instruction::Label(_) => {
                // 标签只是跳转目标，不执行任何操作
            }

            Instruction::MovePlusFP(offset) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.fp + offset;

                // Ensure the stack is large enough
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

            // Object operations
            Instruction::NewObject => {
                self.stack.push(Value::object());
            }

            Instruction::GetField(field) => {
                let obj = self.stack.pop().unwrap_or(Value::null());
                // Use metatable-aware field access
                let mut value = if let Value::String(_) = obj {
                    self.string_prototype.get_field_with_meta(field)
                } else {
                    obj.get_field_with_meta(field)
                };

                if let Value::Null = value
                    && let Value::Object(_) = obj
                        && field == "keys" {
                            let array_proto = self.array_prototype.clone();
                            value = Value::NativeFunction(Rc::new(Box::new(move |args| {
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
                // Use metatable-aware field setting
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
                        && field == "keys" {
                            let array_proto = self.array_prototype.clone();
                            value = Value::NativeFunction(Rc::new(Box::new(move |args| {
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
                // 1. Get function from stack (it's below args)
                // Stack: [... func, arg1, ... argN]
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(*arg_count + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallStack: missing function".to_string(),
                    ))?;

                let func_val = self.stack.remove(func_idx);

                return match func_val {
                    Value::Function(func_name) => {
                        // Reuse logic for user defined functions
                        // Note: We don't support builtins via CallStack yet (except NativeFunction now)
                        let func_label = format!("func_{}", func_name);
                        debug!(
                            "Calling stack function {} (label: {}), arg_count: {}",
                            func_name, func_label, *arg_count
                        );

                        if let Some(target_symbol) = program.syms.get(&func_label) {
                            if *arg_count != target_symbol.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call_stack".to_string(),
                                    left_type: ValueType::Function,
                                    right_type: ValueType::Null,
                                }));
                            }

                            // 1. Save return address and old fp
                            self.call_stack.push((self.pc, self.fp));

                            // 2. Set new fp
                            // Stack is now [... args], so fp is at start of args
                            self.fp = self.stack.len() - *arg_count;

                            // 3. Allocate space for locals
                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());

                            // 4. Jump
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
                        let result = native_fn(args)?;
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

                // Find the nearest exception handler
                if let Some(handler) = self.exception_handlers.pop() {
                    // Restore stack state
                    self.stack.truncate(handler.stack_size);
                    self.fp = handler.fp;

                    // Push error value onto stack
                    self.stack.push(error_value);

                    // Jump to catch block
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

                // No handler found, convert to runtime error
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

    /// 获取当前栈状态（用于调试）
    pub fn get_stack(&self) -> &[Value] {
        &self.stack
    }

    /// 获取变量状态（用于调试）
    pub fn get_variables(&self) -> &IndexMap<String, Value> {
        &self.variables
    }
}

impl Program {
    /// 添加指令
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

#[cfg(test)]
mod tests;
