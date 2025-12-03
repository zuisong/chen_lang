use std::clone::Clone;
use std::fmt::Debug;

use crate::token::Operator;
use crate::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Identifier(String),
    Value(Value),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation {
    pub operator: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
    NotStatement(NotStatement),
    Block(Vec<Statement>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub test: Expression,
    pub body: Vec<Statement>,
    pub else_body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Local {
    pub name: String,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    If(If),
    Loop(Loop),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Local(Local),
    Assign(Assign),
}

pub type Ast = Vec<Statement>;

/// 取反
#[derive(Debug, PartialEq, Clone)]
pub struct NotStatement {
    /// 要取反的表达式
    pub expr: Box<Expression>,
}

/// 打印
#[derive(Debug, Clone)]
pub struct PrintStatement {
    /// 要打印的表达式对象
    pub expression: Box<Expression>,
    /// 是否换行
    pub is_newline: bool,
}

/// 赋值语句
#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    /// 变量名
    pub name: String,
    /// 赋值语句右边的表达式
    pub expr: Box<Expression>,
}

/// 循环语句
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    /// 循环终止判断条件
    pub test: Expression,
    /// 循环语句里面要执行的语句块
    pub body: Vec<Statement>,
}
