use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{ObjClosure, Value};
use crate::vm::program::Program;

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

/// 希望从被调用函数获得的返回值数量。`usize::MAX` 表示"全部"（对应 Lua 的多返回值语义）。
pub const WANT_ALL: usize = usize::MAX;

/// Call stack frame
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub pc: usize,
    pub fp: usize,
    pub program: Option<Rc<Program>>,
    pub closure: Option<Rc<ObjClosure>>,
    pub discard_return: bool,
    pub push_values_after_return: Vec<Value>,
    /// 调用方期望的返回值数量（`WANT_ALL` = 全部）。用于多返回值截断/补齐。
    pub want_return: usize,
    /// 变长参数数组（从参数中收集的额外实参）。非变长函数为 None。
    pub varargs: Option<Rc<RefCell<crate::value::Table>>>,
}

#[derive(Clone)]
pub struct Fiber {
    pub stack: Vec<Value>,
    pub pc: usize,
    pub fp: usize,
    pub call_stack: Vec<CallFrame>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub state: FiberState,
    pub caller: Option<Rc<RefCell<Fiber>>>,
    pub current_closure: Option<Rc<ObjClosure>>,
    pub program: Option<Rc<Program>>,
    /// 协程完成时的返回值（用于 await_all）
    pub result: Option<Value>,
    /// 标记：恢复执行时是否跳过 push 值（用于 spawn 的新协程）
    pub skip_push_on_resume: bool,
    /// 标记：是否是 spawn 创建的协程（完成时需要减少 pending_tasks）
    pub is_spawned: bool,
    /// 支持原生协程：存储要在协程中运行的原生函数
    pub native_function: Option<Rc<Box<crate::value::NativeFnType>>>,
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
            current_closure: None,
            program: None,
            result: None,
            skip_push_on_resume: false,
            is_spawned: false,
            native_function: None,
        }
    }
}

impl std::fmt::Debug for Fiber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fiber")
            .field("state", &self.state)
            .field("stack_len", &self.stack.len())
            .field("pc", &self.pc)
            .finish()
    }
}
