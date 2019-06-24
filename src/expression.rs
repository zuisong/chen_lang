use crate::Context;
use std::clone::Clone;
use std::collections::VecDeque;
use std::fmt::Debug;
use core::fmt::Write;

pub trait Expression: Debug {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const>;
}

#[derive(Debug)]
pub struct Subtract {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
}

impl Expression for Subtract {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int - r_int)),
            (_, _) => panic!("不是数字不能做求余数运算"),
        }
    }
}

#[derive(Debug)]
pub struct Multiply {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
}

impl Expression for Multiply {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int * r_int)),
            (_, _) => panic!("不是数字不能做求余数运算"),
        }
    }
}

#[derive(Debug)]
pub struct Divide {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
}

impl Expression for Divide {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int / r_int)),
            (_, _) => panic!("不是数字不能做求余数运算"),
        }
    }
}

#[derive(Debug)]
pub struct Mod {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
}

impl Expression for Mod {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int % r_int)),
            (_, _) => panic!("不是数字不能做求余数运算"),
        }
    }
}

#[derive(Debug)]
pub struct Add {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
}

impl Expression for Add {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int + r_int)),
            (_, _) => panic!("不是数字不能做加运算"),
        }
    }
}

#[derive(Debug)]
pub struct Println {
    pub expression: Box<dyn Expression>,
}

impl Expression for Println {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(format!("{}\n", res.to_string()));
        None
    }
}

#[derive(Debug)]
pub struct Print {
    pub expression: Box<dyn Expression>,
}

impl Expression for Print {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(res.to_string());
        None
    }
}

// 赋值语句
#[derive(Debug)]
pub struct Var {
    pub left: String,
    pub right: Box<dyn Expression>,
}

impl Expression for Var {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let e = &self.right;
        let res = e.evaluate(ctx).unwrap().clone();
        ctx.variables.insert((&self.left).clone(), res);
        None
    }
}


pub type Cmd = Box<VecDeque<Box<dyn Expression>>>;

impl Expression for Cmd {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        for expre in self.iter() {
            expre.evaluate(ctx);
        }
        None
    }
}

#[derive(Debug)]
pub struct Loop {
    pub predict: Box<dyn Expression>,
    pub cmd: Cmd,
}

impl Expression for Loop {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        loop {
            match self.predict.evaluate(ctx) {
                Some(Const::Int(0)) => {
                    break;
                }
                Some(Const::Int(_)) => {
                    self.cmd.evaluate(ctx);
                }
                _ => {
                    panic!("循环判断条件 返回值 只能是 bool 类型");
                }
            }
        }
        None
    }
}


#[derive(Debug)]
pub struct If {
    pub predict: Box<dyn Expression>,
    pub cmd: Cmd,
}

impl Expression for If {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        // if 语句返回 1 为 真
        if let Some(Const::Int(1)) = self.predict.evaluate(ctx) {
            self.cmd.evaluate(ctx);
        }
        None
    }
}

pub enum Element {
    /// 变量
    Variable(Variable),
    /// 常量
    Const(Const),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Variable {
    pub name: String,
}

impl Expression for Variable {
    fn evaluate(&self, context: &mut Context) -> Option<Const> {
        let val = context.variables.get(&self.name);
        assert!(val.is_some(), "不能获取一个未定义的变量 {}", self.name);
        return Some(val.unwrap().clone());
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Const {
    // 仅支持 int  bool类型
    Int(i32),
    Bool(bool),
    Void,
    String(String),
}

impl Expression for Const {
    fn evaluate(&self, _: &mut Context) -> Option<Const> {
        Some(self.clone())
    }
}

impl ToString for Const {
    fn to_string(&self) -> String {
        match self {
            Const::Int(int) => (*int).to_string(),
            Const::Bool(b) => (*b).to_string(),
            Const::Void => String::new(),
            Const::String(s) => s.clone(),
        }
    }
}
