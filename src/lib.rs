//! 一个小的玩具语言
#![allow(soft_unstable)]
// #![deny(missing_docs)]
// #![deny(unused_imports)]
// #![deny(unused_parens)]
// #![deny(dead_code)]
// #![deny(unused_mut)]
// #![deny(unreachable_code)]

use std::fmt::{Debug, Display};

use anyhow::Result;
use tracing::debug;

use crate::expression::*;
use crate::token::*;

/// 编译器模块
pub mod compiler;
/// 表达式模块
pub mod expression;
/// 语法分析模块
pub mod parse;
/// 词法分析模块
pub mod token;
/// 值系统模块
pub mod value;
/// 虚拟机模块
pub mod vm;

/// 测试模块
#[cfg(test)]
mod tests;

#[inline]
pub(crate) fn err_msg<M>(msg: M) -> anyhow::Error
where
    M: Display + Debug + Send + Sync + 'static,
{
    anyhow::Error::msg(msg)
}

/// 运行代码
#[unsafe(no_mangle)]
pub fn run(code: String) -> Result<()> {
    let tokens = tokenlizer(code.clone())?;
    let ast = parse::parse(tokens)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let mut vm = vm::VM::new();
    let result = vm.execute(&program);
    match result {
        vm::VMResult::Ok(value) => {
            debug!("Execution result: {:?}", value);
        }
        vm::VMResult::Error(error) => {
            eprintln!("Runtime error: {:?}", error);
        }
    }
    Ok(())
}
