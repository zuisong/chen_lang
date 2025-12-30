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
pub mod native_coroutine;
mod native_date;
mod native_fs;
#[cfg(feature = "http")]
mod native_http;
mod native_io;
mod native_json;
mod native_process;
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
    Import(String),               // Import a module
    PushExceptionHandler(String), // Push exception handler (catch label)
    PopExceptionHandler,          // Pop exception handler
}

/// 程序表示
#[derive(Debug, Default, Clone)]
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
pub struct ExceptionHandler {
    pub catch_label: String,
    pub stack_size: usize,
    pub fp: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    Running,
    Suspended,
    Dead,
}

#[derive(Debug, Clone)]
pub struct Fiber {
    pub stack: Vec<Value>,
    pub pc: usize,
    pub fp: usize,
    pub call_stack: Vec<(usize, usize, Option<Rc<Program>>)>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub state: FiberState,
    pub caller: Option<Rc<RefCell<Fiber>>>,
}

impl Default for Fiber {
    fn default() -> Self {
        Self::new()
    }
}

impl Fiber {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            exception_handlers: Vec::new(),
            state: FiberState::Suspended,
            caller: None,
        }
    }
}

/// 虚拟机实现
pub struct VM {
    pub stack: Vec<Value>,                  // 操作数栈
    pub variables: IndexMap<String, Value>, // 全局变量存储
    pub pc: usize,                          // 程序计数器
    pub fp: usize,                          // 帧指针
    // (pc, fp, program)
    pub call_stack: Vec<(usize, usize, Option<Rc<Program>>)>, // 调用栈
    pub module_cache: IndexMap<String, Value>,                // Module Cache
    pub stdout: Box<dyn Write>,                               // 标准输出
    pub array_prototype: Value,                               // 数组原型对象
    pub string_prototype: Value,                              // 字符串原型对象
    // exception_handlers moved to Fiber effectively (current fiber context)
    // But VM still holds current running state.
    // When running, we use VM's vectors. When switching, we swap them.
    pub exception_handlers: Vec<ExceptionHandler>,

    pub current_fiber: Option<Rc<RefCell<Fiber>>>,
    pub program: Option<Rc<Program>>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

use native_array_prototype::create_array_prototype;
use native_coroutine::create_coroutine_object;
use native_date::create_date_object;
use native_fs::create_fs_object;
#[cfg(feature = "http")]
use native_http::create_http_object;
use native_io::create_io_object;
use native_json::create_json_object;
use native_process::create_process_object;

use crate::vm::native_string_prototype::create_string_prototype;

impl VM {
    pub fn new() -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        variables.insert("coroutine".to_string(), create_coroutine_object());

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
            current_fiber: None,
            program: None,
            module_cache: IndexMap::new(),
        }
    }

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        variables.insert("coroutine".to_string(), create_coroutine_object());

        VM {
            stack: Vec::with_capacity(1024),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: writer,
            array_prototype: create_array_prototype(),
            string_prototype: create_string_prototype(),
            exception_handlers: Vec::new(),
            current_fiber: None,
            program: None,
            module_cache: IndexMap::new(),
        }
    }

    /// 注册全局变量
    pub fn register_global_var(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// 注册字符串类型的全局变量
    pub fn add_var_str(&mut self, name: &str, value: &str) {
        self.register_global_var(name, Value::string(value.to_string()));
    }

    /// 注册布尔类型的全局变量
    pub fn add_var_bool(&mut self, name: &str, value: bool) {
        self.register_global_var(name, Value::bool(value));
    }

    /// 注册整数类型的全局变量
    pub fn add_var_int(&mut self, name: &str, value: i32) {
        self.register_global_var(name, Value::int(value));
    }

    /// 注册浮点类型的全局变量
    pub fn add_var_float(&mut self, name: &str, value: f64) {
        self.register_global_var(name, Value::float(Decimal::from_f64_retain(value).unwrap_or_default()));
    }

    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        // We need to clone the program to keep it in VM if we change VM to hold Rc<Program>
        // But execute takes &Program.
        // We probably need to change execute signature or clone/wrap it.
        // Given constraint, we might just not be able to store &Program easily without lifetimes.
        // Using Rc is best.
        // Assuming caller can provide Rc or we clone it (expensive if deep, but Program is Vec and Map).
        // Actually, let's change execute to take Rc<Program> or just wrap it here cheaply if we can?
        // No, we can't wrap &Program into Rc<Program>.

        // TEMPORARY: For this specific task, we'll assume we can't easily change the signature of execute
        // to Rc<Program> without breaking main.rs (which I can't see but user might have).
        // BUT I can change `Program` to be `Rc`'d in `main.rs` if I had access.
        //
        // Let's rely on internal mutability or raw pointers? Unsafe.
        //
        // Let's go with: Modify `execute` to take `Rc<Program>`.
        // This is a breaking change for the API but necessary for this feature.
        // I will change the signature.
        self.execute_rc(Rc::new(program.clone())) // Clone for now to satisfy type checker if we can't change caller.
    }

    pub fn execute_rc(&mut self, program: Rc<Program>) -> VMResult {
        let saved_program = self.program.clone();
        self.program = Some(program.clone());
        let res = self.execute_from(0);
        self.program = saved_program; // Restore previous program
        res
    }

    /// 从指定PC开始执行程序
    /// Note: This now uses self.program which must be set before calling
    pub fn execute_from(&mut self, start_pc: usize) -> VMResult {
        self.pc = start_pc;

        // Execute instructions from self.program, which can change during execution
        loop {
            // Get current program (it may change during CallStack)
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

            // eprintln!(
            //     "PC={}, Instruction={:?}, Stack={:?}",
            //     self.pc, instruction_clone, self.stack
            // );
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

        // 返回栈顶值或null
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
                // 1. Try Standard Library
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
                    // 2. Try User Module (File)
                    // Check Cache
                    if let Some(cached_val) = self.module_cache.get(path) {
                        self.stack.push(cached_val.clone());
                        // Important: Return Ok(true) to continue execution!
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

                    // Compile module
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

                    // Execute recursively
                    // IMPORTANT: Save stack state to prevent pollution
                    let saved_stack_size = self.stack.len();
                    let saved_pc = self.pc;
                    let saved_fp = self.fp;

                    let res = self.execute_rc(Rc::new(module_program));

                    self.pc = saved_pc;
                    self.fp = saved_fp;

                    match res {
                        Ok(val) => {
                            // Clean up stack: restore to saved size, then push result
                            self.stack.truncate(saved_stack_size);
                            self.module_cache.insert(path.clone(), val.clone());
                            self.stack.push(val);
                        }
                        Err(e) => {
                            // On error, also restore stack
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
                    debug!("All variables in VM: {:?}", self.variables);
                    self.stack.push(value.clone());
                } else {
                    // Check if it is a function in the current program
                    let func_label = format!("func_{}", var_name);
                    if program.syms.contains_key(&func_label) {
                        // Create a Closure capturing the current program
                        if let Some(prog) = &self.program {
                            self.stack.push(Value::Closure(var_name.clone(), prog.clone()));
                        } else {
                            // Fallback (should normally have a program)
                            self.stack.push(Value::Function(var_name.clone()));
                        }
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
                    "set_meta" => {
                        // ... existing set_meta logic ...
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
                        // ... existing get_meta logic ...
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

                        // Decision: Should we look up 'program.syms' first?
                        // If it's a value call (e.g. variable holds a closure), we should use variable lookup.
                        // But typically `call foo` is for global functions.
                        // Global functions in current module are in program.syms.
                        // Global functions from imports are in variables (as Closures).
                        // So checking variables is also important.
                        // BUT `call` instruction is specific to simple calls.

                        // Let's check variables first for Closure.
                        // Wait, local variables shadowing?
                        // `Call` usually resolves to checking variables if not strictly a "direct call".
                        // Logic was: syms -> variables -> None.

                        let target_info = if let Some(sym) = program.syms.get(&func_label) {
                            Some((sym.clone(), self.program.clone()))
                        } else if let Some(val) = self.variables.get(func_name) {
                            match val {
                                Value::Closure(name, prog) => {
                                    let label = format!("func_{}", name);
                                    if let Some(sym) = prog.syms.get(&label) {
                                        Some((sym.clone(), Some(prog.clone())))
                                    } else {
                                        None
                                    }
                                }
                                Value::Function(name) => {
                                    // Legacy: assume current program
                                    let label = format!("func_{}", name);
                                    if let Some(sym) = program.syms.get(&label) {
                                        Some((sym.clone(), self.program.clone()))
                                    } else {
                                        None
                                    }
                                }
                                Value::NativeFunction(native_fn) => {
                                    // Handle native below
                                    None
                                }
                                _ => None,
                            }
                        } else {
                            None
                        };

                        // Check native function separately if needed (or integrated above?)
                        // The above matching returns None for NativeFunction, so we fall through.
                        if target_info.is_none() {
                            let native_fn_opt = self.variables.get(func_name).cloned();
                            if let Some(Value::NativeFunction(native_fn)) = native_fn_opt {
                                // Native Call logic
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

                            // 1. 保存返回地址和旧fp和旧program
                            self.call_stack.push((self.pc, self.fp, self.program.clone()));

                            // 2. 设置新fp
                            self.fp = self.stack.len() - *arg_count;

                            // 2.5 Switch Program
                            if let Some(prog) = target_program_opt {
                                self.program = Some(prog);
                            }

                            // 3. 为局部变量分配空间
                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());

                            // 4. 跳转到函数
                            self.pc = (target_symbol.location as usize) - 1;
                            return Ok(true);
                        } else {
                            // debug!("Function label {} not found in {:?}", func_label, program.syms);
                            return Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)));
                        }
                    }
                }
            }
            Instruction::Return => {
                // 1. Pop the return value
                let return_value = self.stack.pop().unwrap_or(Value::null());

                // 2. Pop the call frame (pc, fp, program)
                return if let Some((return_pc, old_fp, old_prog)) = self.call_stack.pop() {
                    // 3. Destroy the current stack frame
                    self.stack.truncate(self.fp);

                    // 4. Restore pc and fp
                    self.pc = return_pc;
                    self.fp = old_fp;

                    // Restore Program if we switched
                    if let Some(prog) = old_prog {
                        self.program = Some(prog);
                    }

                    // 5. Push return value onto the caller's stack
                    self.stack.push(return_value);

                    Ok(true)
                } else {
                    // No more call frames. Check if we are inside a Fiber.
                    if let Some(fiber_rc) = &self.current_fiber {
                        // Mark current fiber as Dead
                        fiber_rc.borrow_mut().state = FiberState::Dead;

                        // Check for caller
                        let caller_opt = fiber_rc.borrow().caller.clone();

                        if let Some(caller_rc) = caller_opt {
                            // Restore caller
                            let caller = caller_rc.borrow();
                            self.load_state_from_fiber(&caller);

                            // Drop borrows
                            drop(caller);

                            // Update current fiber
                            self.current_fiber = Some(caller_rc);

                            // Push return value to caller's stack
                            self.stack.push(return_value);

                            // Continue execution
                            Ok(true)
                        } else {
                            // Fiber has no caller (Main Program finished?)
                            // Or Root fiber finished.
                            debug!("Program end (fiber with no caller)");
                            self.stack.push(return_value);
                            Ok(false)
                        }
                    } else {
                        // Main program end
                        debug!("Program end (no more call stack)");
                        self.stack.push(return_value);
                        Ok(false) // 停止执行
                    }
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
                // DEBUG: Print stack state
                // eprintln!("CallStack: arg_count={}, stack.len()={}", arg_count, self.stack.len());
                // eprintln!("CallStack: Stack state: {:?}", self.stack);

                // 1. Get function from stack (it's below args)
                // Stack: [... func, arg1, ... argN]
                let func_idx = self
                    .stack
                    .len()
                    .checked_sub(*arg_count + 1)
                    .ok_or(VMRuntimeError::StackUnderflow(
                        "CallStack: missing function".to_string(),
                    ))?;

                // eprintln!(
                //     "CallStack: func_idx={}, func_val={:?}",
                //     func_idx,
                //     self.stack.get(func_idx)
                // );
                let func_val = self.stack.remove(func_idx);

                return match func_val {
                    Value::Closure(func_name, prog_rc) => {
                        let func_label = format!("func_{}", func_name);
                        // Use symbol map from the closure's program
                        if let Some(target_symbol) = prog_rc.syms.get(&func_label) {
                            if *arg_count != target_symbol.narguments {
                                return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                                    operator: "call_stack".to_string(),
                                    left_type: ValueType::Function,
                                    right_type: ValueType::Null,
                                }));
                            }

                            // 1. Save return address, old fp, and current program
                            self.call_stack.push((self.pc, self.fp, self.program.clone()));

                            // 2. Set new fp
                            self.fp = self.stack.len() - *arg_count;

                            // 2.5 Switch program
                            self.program = Some(prog_rc.clone());

                            // 3. Allocate space for locals
                            self.stack.resize(self.fp + target_symbol.nlocals, Value::null());

                            // 4. Jump
                            self.pc = (target_symbol.location as usize) - 1;
                            Ok(true)
                        } else {
                            Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)))
                        }
                    }
                    Value::Function(func_name) => {
                        // Reuse logic for user defined functions (same program)
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
                            self.call_stack.push((self.pc, self.fp, self.program.clone()));

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
mod vm_tests;
