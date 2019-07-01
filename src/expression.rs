use std::clone::Clone;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::result::Result::Err;

use failure::{err_msg, Error};

use crate::context::*;
use crate::token::Operator;
use std::rc::Rc;

/// 表达式  核心对象
/// 一切语法都是表达式
pub trait Expression: Debug {
    ///
    /// 表达式执行的方法
    ///
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error>;
}

#[derive(Debug)]
pub struct CallFunctionStatement {
    pub function_name: String,
    pub params: Vec<Element>,
}

impl Expression for CallFunctionStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, Error> {
        let params: Vec<_> = self
            .params
            .iter()
            .map(|it| it.evaluate(ctx).unwrap())
            .collect();
        let func = ctx.get_function(self.function_name.as_str()).unwrap();
        let mut new_ctx = Context::init_with_parent_context(ctx);
        for idx in 0..func.params.len() {
            new_ctx.insert_var(func.params[idx].as_str(), params[idx].clone(), VarType::Let);
        }
        func.body.evaluate(&mut new_ctx)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub params: Vec<String>,
    pub body: Rc<BlockStatement>,
}

impl Expression for FunctionStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, Error> {
        ctx.insert_function(self.name.as_str(), self.clone());
        Ok(Value::Void)
    }
}

///
/// 二元操作符
#[derive(Debug)]
pub struct BinaryStatement {
    /// 操作符左边的表达式
    pub left: Box<dyn Expression>,
    /// 操作符右边的表达式
    pub right: Box<dyn Expression>,
    /// 操作符
    pub operator: Operator,
}

impl Expression for BinaryStatement {
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
pub struct NotStatement {
    /// 要取反的表达式
    pub expr: Box<dyn Expression>,
}

impl Expression for NotStatement {
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
pub struct PrintStatement {
    /// 要打印的表达式对象
    pub expression: Box<dyn Expression>,
    /// 是否换行
    pub is_newline: bool,
}

impl Expression for PrintStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let res = self.expression.evaluate(ctx).unwrap();
        print!("{}", res.to_string());
        if self.is_newline {
            println!();
        }
        Ok(Value::Void)
    }
}

/// 赋值语句
#[derive(Debug)]
pub struct DeclareStatement {
    /// 变量类型
    pub var_type: VarType,
    /// 变量名
    pub left: String,
    /// 赋值语句右边的表达式
    pub right: Box<dyn Expression>,
}

impl Expression for DeclareStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, Error> {
        let res = self.right.evaluate(ctx)?;
        let is_ok = ctx.insert_var(self.left.as_str(), res, (&self.var_type).clone());
        if is_ok {
            Ok(Value::Void)
        } else {
            Err(err_msg(format!("重复定义变量, {}", self.left)))
        }
    }
}

/// 赋值语句
#[derive(Debug)]
pub struct AssignStatement {
    /// 变量名
    pub left: String,
    /// 赋值语句右边的表达式
    pub right: Box<dyn Expression>,
}

impl Expression for AssignStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let e = &self.right;
        //        dbg!(&e);
        let res = e.evaluate(ctx)?.clone();
        let b = ctx.update_var((&self.left).as_str(), res);
        if b {
            Ok(Value::Void)
        } else {
            Err(err_msg(format!("赋值失败,{}", self.left)))
        }
    }
}

/// 一串表达式的集合
pub type BlockStatement = VecDeque<Box<dyn Expression>>;

impl Expression for BlockStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let mut new_ctx: Context = Context::init_with_parent_context(ctx);
        let mut res = Value::Void;
        for expr in self.iter() {
            res = expr.evaluate(&mut new_ctx)?;
        }
        Ok(res)
    }
}

/// 循环语句
#[derive(Debug)]
pub struct LoopStatement {
    /// 循环终止判断条件
    pub predict: Box<dyn Expression>,
    /// 循环语句里面要执行的语句块
    pub loop_block: BlockStatement,
}

impl Expression for LoopStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let mut new_ctx: Context = Context::init_with_parent_context(ctx);

        loop {
            match self.predict.evaluate(&mut new_ctx)? {
                Value::Bool(false) => {
                    break;
                }
                Value::Bool(true) => {
                    self.loop_block.evaluate(&mut new_ctx)?;
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
pub struct IfStatement {
    /// 条件语句 判断条件
    pub predict: Box<dyn Expression>,
    /// 条件语句为真时执行的语句块
    pub if_block: BlockStatement,
    /// 条件语句为假时执行的语句块
    pub else_block: BlockStatement,
}

impl Expression for IfStatement {
    fn evaluate(&self, ctx: &mut Context) -> Result<Value, failure::Error> {
        let mut new_ctx: Context = Context::init_with_parent_context(ctx);
        match self.predict.evaluate(&mut new_ctx)? {
            Value::Bool(false) => {
                self.else_block.evaluate(&mut new_ctx)?;
            }
            Value::Bool(true) => {
                self.if_block.evaluate(&mut new_ctx)?;
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
    Variable(VariableStatement),
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
pub struct VariableStatement {
    /// 变量名
    pub name: String,
}

impl Expression for VariableStatement {
    fn evaluate(&self, context: &mut Context) -> Result<Value, failure::Error> {
        let val = context.get_var(&self.name);
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
