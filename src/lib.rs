#![feature(box_syntax)]
#![deny(missing_docs)]
//! A simple key/value store.
#[deny(unused_imports)]
#[deny(unused_parens)]
#[deny(dead_code)]
#[deny(unused_mut)]
#[deny(unreachable_code)]
extern crate wasm_bindgen;

///
/// 关键字   if for
/// 函数库   print println
/// 操作符  = +-*/  ==
/// 逻辑运算符  && || ！
/// 标识符   纯字母
///
use std::collections::HashMap;

use failure::*;
use log::*;
use wasm_bindgen::prelude::*;

use crate::expression::*;
use crate::token::*;

/// 表达式模块
pub mod expression;
/// 语法分析模块
pub mod parse;
/// 词法分析模块
pub mod token;
/// 测试模块
#[cfg(test)]
mod tests;

/// 运行代码
#[wasm_bindgen]
pub fn run(code: String) -> Result<(), failure::Error> {
    let tokens = token::tokenlizer(code)?;
    debug!("tokens => {:?}", &tokens);
    let ast: Command = parser(tokens)?;
    debug!("ast => {:?}", &ast);

    evaluate(ast)?;
    Ok(())
}

fn parser(tokens: Vec<Token>) -> Result<Command, failure::Error> {
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
    let (_, ast) = parse::parse_sequence(lines.as_slice(), 0)?;

    return Ok(ast);
}

fn evaluate(ast: Command) -> Result<Value, failure::Error> {
    let mut ctx = Context {
        output: vec![],
        variables: Default::default(),
    };
    debug!("{:?}", &ast);
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx)?;
    }

    for x in ctx.output {
        print!("{}", x);
    }
    Ok(Value::Void)
}

/// 程序上下文
/// 保存变量和输出的值
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Context {
    /// 输出
    output: Vec<String>,
    /// 变量池
    variables: HashMap<String, Value>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            output: vec![],
            variables: Default::default(),
        }
    }
}
