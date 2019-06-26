#![feature(box_syntax)]

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

use crate::expression::*;
use crate::token::*;
use failure::*;
use log::*;
use wasm_bindgen::prelude::*;

pub mod expression;
pub mod parse;
pub mod token;

#[wasm_bindgen]
pub fn run(code: String) -> Result<(), failure::Error> {
    let tokens = token::tokenlizer(code)?;
    debug!("tokens => {:?}", &tokens);
    let ast: Command = parser(tokens)?;
    debug!("ast => {:?}", &ast);

    evaluate(ast);
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

fn evaluate(ast: Command) {
    let mut ctx = Context {
        output: vec![],
        variables: Default::default(),
    };
    debug!("{:?}", &ast);
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx);
    }

    for x in ctx.output {
        print!("{}", x);
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Context {
    output: Vec<String>,
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
