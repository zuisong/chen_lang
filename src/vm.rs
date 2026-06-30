use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use indexmap::IndexMap;
use jiff::Timestamp;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

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
use native_object_prototype::create_object_prototype;
use native_string_prototype::create_string_prototype;
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

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("nil".to_string(), Value::null());
        variables.insert("coroutine".to_string(), create_coroutine_object());
        let object_prototype = create_object_prototype();
        variables.insert("Object".to_string(), object_prototype.clone());

        let print_fn = |vm: &mut VM, args: Vec<Value>| {
            for val in args {
                write!(vm.stdout, "{}", val).unwrap();
            }
            vm.stdout.flush().unwrap();
            Ok(Value::null())
        };
        variables.insert("print".to_string(), Value::NativeFunction(Rc::new(Box::new(print_fn))));

        let println_fn = |vm: &mut VM, args: Vec<Value>| {
            for val in args {
                write!(vm.stdout, "{}", val).unwrap();
            }
            writeln!(vm.stdout).unwrap();
            vm.stdout.flush().unwrap();
            Ok(Value::null())
        };
        variables.insert("println".to_string(), Value::NativeFunction(Rc::new(Box::new(println_fn))));

        let require_fn = |vm: &mut VM, args: Vec<Value>| {
            let err = |msg: &str| {
                Err(crate::vm::VMRuntimeError::UncaughtException(msg.to_string()))
            };
            if args.is_empty() {
                return err("require() expects a module path");
            }
            let path = match &args[0] {
                Value::String(s) => s.as_ref().clone(),
                _ => return err("require() expects a string argument"),
            };
            if path.starts_with("stdlib/") {
                let module = match path.as_str() {
                    "stdlib/json" => crate::vm::native_json::create_json_object(),
                    "stdlib/date" => crate::vm::native_date::create_date_object(),
                    "stdlib/fs" => crate::vm::native_fs::create_fs_object(),
                    "stdlib/http" => {
                        #[cfg(feature = "http")]
                        { crate::vm::native_http::create_http_object() }
                        #[cfg(not(feature = "http"))]
                        { Value::Null }
                    }
                    "stdlib/process" => crate::vm::native_process::create_process_object(),
                    "stdlib/io" => crate::vm::native_io::create_io_object(),
                    "stdlib/timer" => crate::vm::native_timer::create_timer_object(),
                    _ => return err(&format!("Stdlib module not found: {}", path)),
                };
                return Ok(module);
            }
            if let Some(cached) = vm.module_cache.get(&path) {
                return Ok(cached.clone());
            }
            let code = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => return err(&format!("Failed to require {}: {}", path, e)),
            };
            let ast = match crate::parser::parse_from_source(&code) {
                Ok(a) => a,
                Err(e) => return err(&format!("Parse error in {}: {}", path, e)),
            };
            let module_program = crate::compiler::compile(&code.chars().collect::<Vec<char>>(), ast);
            let saved_stack = vm.stack.len();
            let saved_pc = vm.pc;
            let saved_fp = vm.fp;
            vm.fp = vm.stack.len();
            vm.pc = 0;
            match vm.execute(&module_program) {
                Ok(return_val) => {
                    vm.stack.truncate(saved_stack);
                    vm.pc = saved_pc;
                    vm.fp = saved_fp;
                    vm.module_cache.insert(path.clone(), return_val.clone());
                    Ok(return_val)
                }
                Err(e) => {
                    vm.stack.truncate(saved_stack);
                    vm.pc = saved_pc;
                    vm.fp = saved_fp;
                    err(&format!("Error loading {}: {}", path, e))
                }
            }
        };
        variables.insert("require".to_string(), Value::NativeFunction(Rc::new(Box::new(require_fn))));

        let error_fn = |_vm: &mut VM, args: Vec<Value>| {
            let msg = if args.is_empty() {
                "error".to_string()
            } else {
                args[0].to_string()
            };
            Err(crate::vm::VMRuntimeError::UncaughtException(msg))
        };
        variables.insert("error".to_string(), Value::NativeFunction(Rc::new(Box::new(error_fn))));

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
