use std::clone::Clone;
use std::fmt::Debug;

use crate::token::Operator;

#[derive(Debug)]
pub enum Literal {
    Identifier(String),
    Value(Value),
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct BinaryOperation {
    pub operator: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
    NotStatement(NotStatement),
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct If {
    pub test: Expression,
    pub body: Vec<Statement>,
    pub else_body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Local {
    pub name: String,
    pub expression: Expression,
}

#[derive(Debug)]
pub struct Return {
    pub expression: Expression,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct NotStatement {
    /// 要取反的表达式
    pub expr: Box<Expression>,
}

/// 打印
#[derive(Debug)]
pub struct PrintStatement {
    /// 要打印的表达式对象
    pub expression: Box<Expression>,
    /// 是否换行
    pub is_newline: bool,
}

/// 赋值语句
#[derive(Debug)]
pub struct Assign {
    /// 变量名
    pub name: String,
    /// 赋值语句右边的表达式
    pub expr: Box<Expression>,
}

/// 循环语句
#[derive(Debug)]
pub struct Loop {
    /// 循环终止判断条件
    pub test: Expression,
    /// 循环语句里面要执行的语句块
    pub body: Vec<Statement>,
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
