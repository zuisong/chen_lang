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
/// 编译器模块
pub mod compiler;

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
    let mut lines: Vec<Box<[Token]>> = vec![];
    let mut temp = vec![];
    for x in tokens {
        if let Token::NewLine = x {
            if !temp.is_empty() {
                lines.push(temp.into_boxed_slice());
                temp = vec![];
            }
        } else {
            temp.push(x)
        }
    }
    let (_, ast) = parse::parse_block(lines.as_slice(), 0)?;
    debug!("ast => {:?}", &ast);

    // 编译为字节码并执行
    let raw_chars: Vec<char> = code.chars().collect();
    let program = compiler::compile(&raw_chars, ast);
    debug!("Generated instructions: {:?}", program.instructions);
    debug!("Generated syms: {:?}", program.syms);

    let mut vm = vm::VM::new();
    match vm.execute(&program) {
        vm::VMResult::Ok(result) => {
            debug!("Execution result: {:?}", result);
        }
        vm::VMResult::Error(error) => {
            return Err(anyhow::Error::msg(format!("Runtime error: {}", error)));
        }
    }

    Ok(())
}
