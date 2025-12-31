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
}
