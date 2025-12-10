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
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation {
    pub operator: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal, u32),
    Unary(Unary),
    Identifier(String, u32),
    Block(Vec<Statement>, u32),
    If(If),
    /// 对象字面量: #{ k: v, ... }
    ObjectLiteral(Vec<(String, Expression)>, u32),
    /// 数组字面量
    ArrayLiteral(Vec<Expression>, u32),
    /// 属性访问: obj.field
    GetField {
        object: Box<Expression>,
        field: String,
        line: u32,
    },
    /// 索引访问: obj[expr]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        line: u32,
    },
    /// 函数定义表达式 (匿名函数/Lambda)
    Function(FunctionDeclaration),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: Option<String>,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub test: Box<Expression>,
    pub body: Vec<Statement>,
    pub else_body: Vec<Statement>,
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Local {
    pub name: String,
    pub expression: Expression,
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expression: Expression,
    pub line: u32,
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
        line: u32,
    },
    /// 设置索引: obj[index] = value
    SetIndex {
        object: Expression,
        index: Expression,
        value: Expression,
        line: u32,
    },
    Break(u32),
    Continue(u32),
}

pub type Ast = Vec<Statement>;

/// 一元表达式
#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub operator: Operator,
    pub expr: Box<Expression>,
    pub line: u32,
}

/// 赋值语句
#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    /// 变量名
    pub name: String,
    /// 赋值语句右边的表达式
    pub expr: Box<Expression>,
    pub line: u32,
}

/// 循环语句
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    /// 循环终止判断条件
    pub test: Expression,
    /// 循环语句里面要执行的语句块
    pub body: Vec<Statement>,
    pub line: u32,
}
