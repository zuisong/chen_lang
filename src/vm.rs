use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use indexmap::IndexMap;
use jiff::Timestamp;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::{compiler, parser};

pub mod error;
pub mod fiber;
pub mod interpreter;
pub mod program;

mod native_array_prototype;
pub mod native_coroutine;
mod native_date;
mod native_fs;
#[cfg(feature = "http")]
mod native_http;
mod native_io;
mod native_json;
mod native_object_prototype;
mod native_process;
mod native_string_prototype;
mod native_timer;

pub mod rt;
use rt::AsyncState;

#[cfg(test)]
mod vm_tests;

pub use error::{RuntimeErrorWithContext, VMResult, VMRuntimeError};
pub use fiber::{ExceptionHandler, Fiber, FiberState};
use native_array_prototype::create_array_prototype;
use native_coroutine::create_coroutine_object;
use native_date::create_date_object;
use native_fs::create_fs_object;
#[cfg(feature = "http")]
use native_http::create_http_object;
use native_io::create_io_object;
use native_json::create_json_object;
use native_object_prototype::create_object_prototype;
use native_process::create_process_object;
use native_string_prototype::create_string_prototype;
use native_timer::create_timer_object;
pub use program::{Instruction, Program, Symbol};

pub(crate) use crate::value::{NativeFnType, ObjClosure, Value, ValueError, ValueType};

/// 虚拟机实现
pub struct VM {
    pub stack: Vec<Value>,                  // 操作数栈
    pub variables: IndexMap<String, Value>, // 全局变量存储
    pub pc: usize,                          // 程序计数器
    pub fp: usize,                          // 帧指针
    // (pc, fp, program, closure)
    pub call_stack: Vec<fiber::CallFrame>,     // 调用栈
    pub module_cache: IndexMap<String, Value>, // Module Cache
    pub stdout: Box<dyn Write>,                // 标准输出
    pub array_prototype: Value,                // 数组原型对象
    pub string_prototype: Value,               // 字符串原型对象
    pub object_prototype: Value,               // 对象原型对象
    pub exception_handlers: Vec<ExceptionHandler>,
    pub open_upvalues: Vec<Rc<RefCell<crate::value::UpvalueState>>>,

    pub current_fiber: Option<Rc<RefCell<Fiber>>>,
    pub program: Option<Rc<Program>>,
    pub current_closure: Option<Rc<ObjClosure>>,
    pub current_this: Option<Value>,

    // Async Runtime State
    pub async_state: AsyncState,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self::with_writer(Box::new(std::io::stdout()))
    }

    fn load_user_module(&mut self, path: &str) -> Result<Value, VMRuntimeError> {
        if let Some(cached_val) = self.module_cache.get(path) {
            return Ok(cached_val.clone());
        }

        let code = std::fs::read_to_string(path)
            .map_err(|e| VMRuntimeError::UncaughtException(format!("Failed to load {}: {}", path, e)))?;
        let ast = parser::parse_from_source(&code)
            .map_err(|e| VMRuntimeError::UncaughtException(format!("Parse error in {}: {}", path, e)))?;
        let module_program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

        let saved_stack_size = self.stack.len();
        let saved_pc = self.pc;
        let saved_fp = self.fp;
        let res = self.execute_rc(Rc::new(module_program));

        self.pc = saved_pc;
        self.fp = saved_fp;

        match res {
            Ok(val) => {
                self.stack.truncate(saved_stack_size);
                self.module_cache.insert(path.to_string(), val.clone());
                Ok(val)
            }
            Err(e) => {
                self.stack.truncate(saved_stack_size);
                Err(e.error)
            }
        }
    }

    fn create_console_object() -> Value {
        let console = Value::object();
        if let Value::Object(console_obj) = &console {
            let mut console_obj = console_obj.borrow_mut();
            console_obj.data.insert(
                "print".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        for (i, val) in ctx.args.iter().enumerate() {
                            if i > 0 {
                                write!(vm.stdout, " ").unwrap();
                            }
                            write!(vm.stdout, "{}", val).unwrap();
                        }
                        vm.stdout.flush().unwrap();
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            console_obj.data.insert(
                "log".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        for (i, val) in ctx.args.iter().enumerate() {
                            if i > 0 {
                                write!(vm.stdout, " ").unwrap();
                            }
                            write!(vm.stdout, "{}", val).unwrap();
                        }
                        writeln!(vm.stdout).unwrap();
                        vm.stdout.flush().unwrap();
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            console_obj.data.insert(
                "readLine".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, _ctx: crate::value::NativeContext| {
                        use std::io::BufRead;

                        let stdin = std::io::stdin();
                        let mut line = String::new();
                        stdin
                            .lock()
                            .read_line(&mut line)
                            .map_err(|e| VMRuntimeError::UncaughtException(e.to_string()))?;
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        Ok(Value::string(line))
                    }) as Box<NativeFnType>,
                )),
            );
        }
        console
    }

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        let coroutine_obj = create_coroutine_object();
        variables.insert("coroutine".to_string(), coroutine_obj.clone());
        let object_prototype = create_object_prototype();
        variables.insert("Object".to_string(), object_prototype.clone());
        variables.insert("JSON".to_string(), create_json_object());
        variables.insert("console".to_string(), Self::create_console_object());

        let chen = Value::object();
        if let Value::Object(chen_obj) = &chen {
            let mut chen_obj = chen_obj.borrow_mut();
            chen_obj.data.insert("fs".to_string(), create_fs_object());
            #[cfg(feature = "http")]
            chen_obj.data.insert("http".to_string(), create_http_object());
            chen_obj.data.insert("process".to_string(), create_process_object());
            chen_obj.data.insert("timer".to_string(), create_timer_object());
            chen_obj.data.insert("date".to_string(), create_date_object());
            chen_obj.data.insert("coroutine".to_string(), coroutine_obj);
            chen_obj.data.insert("io".to_string(), create_io_object());
            chen_obj.data.insert(
                "setMeta".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        if args.len() < 2 {
                            return Err(ValueError::InvalidOperation {
                                operator: "Chen.setMeta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }
                            .into());
                        }
                        let obj = &args[0];
                        let meta = args[1].clone();
                        obj.set_metatable(meta)?;
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            chen_obj.data.insert(
                "getMeta".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        let Some(obj) = args.first() else {
                            return Err(ValueError::InvalidOperation {
                                operator: "Chen.getMeta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }
                            .into());
                        };
                        Ok(obj.get_metatable())
                    }) as Box<NativeFnType>,
                )),
            );
            chen_obj.data.insert(
                "load".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        let Some(path_arg) = args.first() else {
                            return Err(ValueError::TypeMismatch {
                                expected: ValueType::String,
                                found: ValueType::Null,
                                operation: "Chen.load".to_string(),
                            }
                            .into());
                        };
                        let path = path_arg.as_string().ok_or_else(|| {
                            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                                expected: ValueType::String,
                                found: path_arg.get_type(),
                                operation: "Chen.load".to_string(),
                            })
                        })?;
                        vm.load_user_module(path)
                    }) as Box<NativeFnType>,
                )),
            );
        }
        variables.insert("Chen".to_string(), chen);

        VM {
            stack: Vec::with_capacity(1024),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: writer,
            array_prototype: create_array_prototype(),
            string_prototype: create_string_prototype(),
            object_prototype,
            exception_handlers: Vec::new(),
            open_upvalues: Vec::new(),
            current_fiber: None,
            program: None,
            current_closure: None,
            current_this: None,
            module_cache: IndexMap::new(),
            async_state: AsyncState::new(),
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

    /// 获取当前栈状态（用于调试）
    pub fn get_stack(&self) -> &[Value] {
        &self.stack
    }

    /// 获取变量状态（用于调试）
    pub fn get_variables(&self) -> &IndexMap<String, Value> {
        &self.variables
    }

    /// 快速抛出运行时异常（供 native function 使用）
    pub fn throw_str(&mut self, msg: impl Into<String>) -> Result<Value, VMRuntimeError> {
        Err(VMRuntimeError::UncaughtException(msg.into()))
    }
}
