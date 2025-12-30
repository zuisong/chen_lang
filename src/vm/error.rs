use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::value::{Value, ValueError};

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
        write!(
            f,
            "Runtime error at line {}: {} (PC: {})",
            self.line, self.error, self.pc
        )
    }
}

/// VM执行结果
pub type VMResult = Result<Value, RuntimeErrorWithContext>;
