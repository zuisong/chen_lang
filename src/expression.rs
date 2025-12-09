use std::clone::Clone;
use std::fmt::Debug;

use crate::token::Operator;
use crate::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Value(Value),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub callee: Box<Expression>,
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
    Unary(Unary),
    Identifier(String),
    Block(Vec<Statement>),
    If(If),
    /// 对象字面量: #{ k: v, ... }
    ObjectLiteral(Vec<(String, Expression)>),
    /// 数组字面量
    ArrayLiteral(Vec<Expression>),
    /// 属性访问: obj.field
    GetField {
        object: Box<Expression>,
        field: String,
    },
    /// 索引访问: obj[expr]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    /// 函数定义表达式 (匿名函数/Lambda)
    Function(FunctionDeclaration),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: Option<String>,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub test: Box<Expression>,
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
    Loop(Loop),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Local(Local),
    Assign(Assign),
    /// 设置属性: obj.field = value
    SetField {
        object: Expression,
        field: String,
        value: Expression,
    },
    /// 设置索引: obj[index] = value
    SetIndex {
        object: Expression,
        index: Expression,
        value: Expression,
    },
    Break,
    Continue,
}

pub type Ast = Vec<Statement>;

/// 一元表达式
#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub operator: Operator,
    pub expr: Box<Expression>,
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
