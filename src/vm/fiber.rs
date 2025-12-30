use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;
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
