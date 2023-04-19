//! 一个小的玩具语言
// #![deny(missing_docs)]
#![allow(soft_unstable)]
#![deny(unused_imports)]
#![deny(unused_parens)]
#![deny(dead_code)]
#![deny(unused_mut)]
#![deny(unreachable_code)]

use std::fmt::{Debug, Display};

///
/// 关键字   if for
/// 函数库   print println
/// 操作符  = +-*/  ==
/// 逻辑运算符  && || ！
/// 标识符   纯字母
///
use log::*;

use crate::context::Context;
use crate::expression::*;
use crate::token::*;

/// context模块
pub mod context;
/// 表达式模块
pub mod expression;
/// 语法分析模块
pub mod parse;
/// 测试模块
#[cfg(test)]
mod tests;
/// 词法分析模块
pub mod token;

#[inline]
pub(crate) fn err_msg<M>(msg: M) -> anyhow::Error
where
    M: Display + Debug + Send + Sync + 'static,
{
    anyhow::Error::msg(msg)
}

/// 运行代码
#[no_mangle]
pub fn run(code: String) -> Result<(), anyhow::Error> {
    let tokens = tokenlizer(code)?;
    debug!("tokens => {:?}", &tokens);
    let ast: BlockStatement = parser(tokens)?;
    debug!("ast => {:?}", &ast);
    evaluate(ast)?;
    Ok(())
}

/// 词法
fn parser(tokens: Vec<Token>) -> Result<BlockStatement, anyhow::Error> {
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

    Ok(ast)
}

/// 运行
fn evaluate(ast: BlockStatement) -> Result<Value, anyhow::Error> {
    let mut ctx = Context::default();
    debug!("{:?}", &ast);
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx)?;
    }

    Ok(Value::Void)
}
