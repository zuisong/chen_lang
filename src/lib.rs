#[allow(unused_imports)]
#[allow(unused_parens)]
#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unreachable_code)]
///
/// 关键字   if for int
/// 函数库   print
/// 操作符  = +-*/  ==
/// 逻辑运算符  && || ！
/// 标识符   纯字母
///
use std::collections::{HashMap, VecDeque};

use crate::expression::*;
use crate::token::*;
use failure::*;
use log::*;


pub mod expression;
pub mod parse;
pub mod token;

pub fn run(code: String) -> Result<(), failure::Error> {
    let tokens = token::tokenlizer(code)?;
    debug!("{:?}",&tokens);
    let ast = parser(tokens)?;
    evlate(ast);
    Ok(())
}

fn parser(tokens: Vec<Token>) -> Result<Cmd, failure::Error> {
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

fn evlate(ast: Cmd) {
    let mut ctx = Context {
        output: vec![],
        variables: Default::default(),
    };
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx);
    }
    debug!("{:?}",&ast);

    for x in ctx.output {
        println!("{}", x.to_string());
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Context {
    output: Vec<Const>,
    variables: HashMap<String, Const>,
}
