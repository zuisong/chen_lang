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
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error>;
}

#[derive(Debug)]
pub struct BinaryOperator {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: Operator,
}

impl Expression for BinaryOperator {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let l = self.left.evaluate(ctx)?;
        let r = self.right.evaluate(ctx)?;
        match self.operator {
            Operator::ADD => match (l, r) {
                (Value::Int(l_int), Value::Int(r_int)) => Ok(Value::Int(l_int + r_int)),
                (Value::String(_), _) | (_, Value::String(_)) => {
                    //                    Ok(Value::String(format!("{}{}", l.to_string(), r.to_string())))
                    unimplemented!("字符串加法还在做")
                }
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

/// 小于
#[derive(Debug)]
pub struct Not {
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

#[derive(Debug)]
pub struct Println {
    pub expression: Box<dyn Expression>,
}

impl Expression for Println {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(format!("{}\n", res.to_string()));
        Ok(Value::Void)
    }
}

#[derive(Debug)]
pub struct Print {
    pub expression: Box<dyn Expression>,
}

impl Expression for Print {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(res.to_string());
        Ok(Value::Void)
    }
}

// 赋值语句
#[derive(Debug)]
pub struct Var {
    pub left: String,
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

pub type Command = Box<VecDeque<Box<dyn Expression>>>;

impl Expression for Command {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let mut res = Ok(Value::Void);
        for expr in self.iter() {
            res = expr.evaluate(ctx);
        }
        res
    }
}

#[derive(Debug)]
pub struct Loop {
    pub predict: Box<dyn Expression>,
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

#[derive(Debug)]
pub struct If {
    pub predict: Box<dyn Expression>,
    pub if_cmd: Command,
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Variable {
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
        return Ok(val.unwrap().clone());
    }
}

/// ----------------------------------------
/// 常数类型
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    // 仅支持 int  bool类型
    Int(i32),
    Bool(bool),
    Void,
    String(String),
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
            Value::String(s) => s.clone(),
            //            Value::Float(f) => f.to_string(),
        }
    }
}
//-----------------------------------------

#[cfg(test)]
mod tests {
    use crate::expression::Element::Value;
    use crate::expression::Value::{Bool, Int};
    use crate::expression::{BinaryOperator, Expression};
    use crate::token::Operator;
    use crate::Context;

    #[test]
    fn test_sub_int_int() {
        let mut ctx = Context::default();
        let opt = box BinaryOperator {
            operator: Operator::Subtract,
            left: box Value(Int(1)),
            right: box Value(Int(1)),
        };
        assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(0));
    }

    #[should_panic]
    #[test]
    fn test_sub_bool_int() {
        let mut ctx = Context::default();
        let opt: Box<dyn Expression> = box BinaryOperator {
            operator: Operator::ADD,
            left: box Value(Bool(false)),
            right: box Value(Int(1)),
        };
        opt.evaluate(&mut ctx).unwrap();
    }

    #[test]
    fn test_add_int_int() {
        let mut ctx = Context::default();
        let opt = BinaryOperator {
            operator: Operator::ADD,
            left: box Value(Int(1)),
            right: box Value(Int(1)),
        };
        assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(2));
    }

    #[should_panic]
    #[test]
    fn test_add_bool_int() {
        let mut ctx = Context::default();
        let opt = BinaryOperator {
            operator: Operator::ADD,
            left: box Value(Bool(false)),
            right: box Value(Int(1)),
        };
        opt.evaluate(&mut ctx).unwrap();
    }
}
