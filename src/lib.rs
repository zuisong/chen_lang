#![feature(box_syntax)]
//#![deny(missing_docs)]
//#![deny(unused_imports)]
//#![deny(unused_parens)]
//#![deny(dead_code)]
//#![deny(unused_mut)]
//#![deny(unreachable_code)]
//! 一个小的玩具语言
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

use crate::expression::*;
use crate::token::*;

use std::cell::RefCell;
use std::rc::Rc;

/// 表达式模块
pub mod expression;
/// 语法分析模块
pub mod parse;
/// 测试模块
#[cfg(test)]
mod tests;
/// 词法分析模块
pub mod token;

/// 运行代码
#[no_mangle]
pub fn run(code: String) -> Result<(), failure::Error> {
    let tokens = token::tokenlizer(code)?;
    debug!("tokens => {:?}", &tokens);
    let ast: BlockStatement = parser(tokens)?;
    debug!("ast => {:?}", &ast);

    evaluate(ast)?;
    Ok(())
}

/// 词法
fn parser(tokens: Vec<Token>) -> Result<BlockStatement, failure::Error> {
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

    Ok(ast)
}

/// 运行
fn evaluate(ast: BlockStatement) -> Result<Value, failure::Error> {
    let mut ctx = Context::default();
    debug!("{:?}", &ast);
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx)?;
    }

    Ok(Value::Void)
}

/// 程序上下文
/// 保存变量和输出的值
#[derive(Debug)]
pub struct Context<'a> {
    /// 父级上下文
    parent: Option<&'a mut Context<'a>>,

    /// 变量池
    variables: HashMap<String, ValueVar>,
}

trait Var {
    fn get(&self) -> Value;
    fn set(&self, val: Value) -> bool;
}

#[derive(Clone, Debug)]
pub enum VarType {
    Const,
    Let,
}

#[derive(Clone, Debug)]
pub struct ValueVar {
    var_type: VarType,
    value: Option<Rc<RefCell<Value>>>,
}

impl ValueVar {
    pub fn new(var_type: VarType, value: Value) -> Self {
        ValueVar {
            var_type,
            value: Some(Rc::new(RefCell::new(value))),
        }
    }
}

impl Var for ValueVar {
    fn get(&self) -> Value {
        assert!(self.value.is_some(), "get a undefined value");
        (&self.value).as_ref().unwrap().clone().borrow().clone()
    }

    fn set(&self, val: Value) -> bool {
        match self.var_type {
            VarType::Const => false,
            VarType::Let => {
                (&self.value).as_ref().unwrap().clone().replace(val);
                true
            }
        }
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        Context {
            parent: None,
            variables: Default::default(),
        }
    }
}

#[inline]
fn init_with_parent_context<'a>(ctx: &'a mut Context<'a>) -> Context<'a> {
    Context {
        parent: Some(ctx),
        variables: Default::default(),
    }
}

impl Context<'_> {
    fn get_var(&self, name: &str) -> Option<Value> {
        match self.variables.get(name) {
            Some(val) => Some(val.get()),
            None => match &self.parent {
                Some(scoop) => scoop.get_var(name),
                None => None,
            },
        }
    }

    fn insert_var(&mut self, name: &str, val: Value, var_type: VarType) -> bool {
        match self.get_var(name) {
            Some(_) => false,
            None => {
                self.variables
                    .insert(name.to_string(), ValueVar::new(var_type, val));
                true
            }
        }
    }

    fn update_var(&mut self, name: &str, value: Value) -> bool {
        match self.variables.get(name) {
            Some(val) => val.set(value),
            None => match &self.parent {
                Some(ctx) => (*ctx).update_var(name, value),
                None => false,
            },
        }
    }
}
