use std::clone::Clone;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::result::Result::Err;

use failure::err_msg;

use crate::token::Operator;
use crate::Context;

/// 表达式  核心对象
/// 一切语法都是表达式
pub trait Expression: Debug {
    ///
    /// 表达式执行的方法
    ///
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error>;
}

///
/// 二元操作符
#[derive(Debug)]
pub struct BinaryOperator {
    /// 操作符左边的表达式
    pub left: Box<dyn Expression>,
    /// 操作符右边的表达式
    pub right: Box<dyn Expression>,
    /// 操作符
    pub operator: Operator,
}

impl Expression for BinaryOperator {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let l = self.left.evaluate(ctx)?;
        let r = self.right.evaluate(ctx)?;
        match self.operator {
            Operator::ADD => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int + r_int)),
                (Value::Str(a), b) => Ok(Value::Str(format!("{}{}", a.to_string(), b.to_string()))),
                (a, Value::Str(b)) => Ok(Value::Str(format!("{}{}", a.to_string(), b.to_string()))),
                _ => Err(err_msg("不是 int string 类型不能做加法")),
            },
            Operator::Subtract => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int - r_int)),
                _ => Err(err_msg("不是 int 类型不能做减法")),
            },
            Operator::Multiply => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int * r_int)),
                _ => Err(err_msg("不是 int 类型不能做乘法")),
            },
            Operator::Divide => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int / r_int)),
                _ => Err(err_msg("不是 int 类型不能做除法")),
            },
            Operator::Mod => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int % r_int)),
                _ => Err(err_msg("不是 int 类型不能做余数运算")),
            },

            Operator::And => match (l, r) {
                (Value::Bool(l_b), Value::Bool(r_b)) => Ok(Value::Bool(l_b && r_b)),
                _ => Err(err_msg("不是 bool 类型不能做逻辑运算")),
            },

            Operator::Or => match (l, r) {
                (Value::Bool(l_b), Value::Bool(r_b)) => Ok(Value::Bool(l_b || r_b)),
                _ => Err(err_msg("不是 bool 类型不能做逻辑运算")),
            },

            Operator::GT => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int > r_int)),
                _ => Err(err_msg("不是 int 类型不能做比较运算")),
            },
            Operator::LT => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int < r_int)),
                _ => Err(err_msg("不是 int 类型不能做比较运算")),
            },
            Operator::GTE => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int >= r_int)),
                _ => Err(err_msg("不是 int 类型不能做比较运算")),
            },
            Operator::LTE => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Bool(l_int <= r_int)),
                _ => Err(err_msg("不是 int 类型不能做比较运算")),
            },
            Operator::Equals => Ok(Value::Bool(l == r)),
            Operator::NotEquals => Ok(Value::Bool(l != r)),
            Operator::NOT => unreachable!("到了这里就错了"),
            Operator::Assign => unreachable!("到了这里就错了"),
        }
    }
}

/// 取反
#[derive(Debug)]
pub struct Not {
    /// 要取反的表达式
    pub expr: Box<dyn Expression>,
}

impl Expression for Not {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let res = self.expr.evaluate(ctx).unwrap();
        match res {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(err_msg("逻辑运算符只能用在 bool 类型上")),
        }
    }
}

/// 打印
#[derive(Debug)]
pub struct Print {
    /// 要打印的表达式对象
    pub expression: Box<dyn Expression>,
    /// 是否换行
    pub is_newline: bool,
}

impl Expression for Print {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(res.to_string());
        if self.is_newline {
            ctx.output.push(String::from("\n"));
        }
        Ok(Value::Void)
    }
}

/// 赋值语句
#[derive(Debug)]
pub struct Var {
    /// 变量名
    pub left: String,
    /// 赋值语句右边的表达式
    pub right: Box<dyn Expression>,
}

impl Expression for Var {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let e = &self.right;
        //        dbg!(&e);
        let res = e.evaluate(ctx)?.clone();
        ctx.variables.insert((&self.left).clone(), res);
        Ok(Value::Void)
    }
}

/// 一串表达式的集合
pub type Command = VecDeque<Box<dyn Expression>>;

impl Expression for Command {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let mut res = Ok(Value::Void);
        for expr in self.iter() {
            res = expr.evaluate(ctx);
        }
        res
    }
}

/// 循环语句
#[derive(Debug)]
pub struct Loop {
    /// 循环终止判断条件
    pub predict: Box<dyn Expression>,
    /// 循环语句里面要执行的语句块
    pub cmd: Command,
}

impl Expression for Loop {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        loop {
            match self.predict.evaluate(ctx)? {
                Value::Bool(false) => {
                    break;
                }
                Value::Bool(true) => {
                    self.cmd.evaluate(ctx)?;
                }
                _ => {
                    return Err(err_msg(
                        "for循环语句 判断语句块的返回值只能是 bool 类型",
                    ));
                }
            }
        }
        Ok(Value::Void)
    }
}

/// 条件语句
#[derive(Debug)]
pub struct If {
    /// 条件语句 判断条件
    pub predict: Box<dyn Expression>,
    /// 条件语句为真时执行的语句块
    pub if_cmd: Command,
    /// 条件语句为假时执行的语句块
    pub else_cmd: Command,
}

impl Expression for If {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        match self.predict.evaluate(ctx)? {
            Value::Bool(false) => {
                self.else_cmd.evaluate(ctx)?;
            }
            Value::Bool(true) => {
                self.if_cmd.evaluate(ctx)?;
            }
            _ => {
                return Err(err_msg(
                    "for条件语句 判断语句块的返回值只能是 bool 类型",
                ));
            }
        }
        Ok(Value::Void)
    }
}

/// 变量和常量的总称
pub enum Element {
    /// 变量
    Variable(Variable),
    /// 常量
    Value(Value),
}

impl Debug for Element {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            Element::Value(v) => v.fmt(f),
            Element::Variable(v) => v.fmt(f),
        }
    }
}

impl Expression for Element {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        match &self {
            Element::Value(v) => v.evaluate(ctx),
            Element::Variable(v) => v.evaluate(ctx),
        }
    }
}

/// 变量
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Variable {
    /// 变量名
    pub name: String,
}

impl Expression for Variable {
    fn evaluate(&self, context: &mut Context) -> Result<Value, failure::Error> {
        let val = context.variables.get(&self.name);
        assert!(
            val.is_some(),
            "不能获取一个未定义的变量 {}",
            self.name
        );
        Ok(val.unwrap().clone())
    }
}

/// ----------------------------------------
/// 常数类型
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    /// int 常量
    Int(i32),
    /// bool 常量
    Bool(bool),
    /// void 常量
    Void,
    /// string 常量
    Str(String),
}

impl Expression for Value {
    fn evaluate(&self, _: &mut Context) -> Result<Value, failure::Error> {
        Ok(self.clone())
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int(int) => (*int).to_string(),
            Value::Bool(b) => (*b).to_string(),
            Value::Void => String::new(),
            Value::Str(s) => s.clone(),
            //            Value::Float(f) => f.to_string(),
        }
    }
}
//-----------------------------------------
