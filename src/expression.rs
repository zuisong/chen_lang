use std::clone::Clone;
use std::fmt::Debug;

use crate::tokenizer::Location;
use crate::tokenizer::Operator;
use crate::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Value(Value),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodCall {
    pub object: Box<Expression>,
    pub method: String,
    pub arguments: Vec<Expression>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation {
    pub operator: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    FunctionCall(FunctionCall),
    MethodCall(MethodCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal, Location),
    Unary(Unary),
    Identifier(String, Location),
    Block(Vec<Statement>, Location),
    If(If),
    /// 对象字面量: ${ k: v, ... }
    ObjectLiteral(Vec<(String, Expression)>, Location),
    /// 数组字面量
    ArrayLiteral(Vec<Expression>, Location),
    /// 属性访问: obj.field
    GetField {
        object: Box<Expression>,
        field: String,
        loc: Location,
    },
    /// 索引访问: obj[expr]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        loc: Location,
    },
    /// 函数定义表达式 (匿名函数/Lambda)
    Function(FunctionDeclaration),
    /// Import 表达式: import "path"
    Import {
        path: String,
        loc: Location,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: Option<String>,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub test: Box<Expression>,
    pub body: Vec<Statement>,
    pub else_body: Vec<Statement>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Local {
    pub name: String,
    pub expression: Expression,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expression: Expression,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Loop(Loop),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Local(Local),
    Assign(Assign),
    /// For-In 循环: for var in iterable { body }
    ForIn(ForInLoop),
    /// 设置属性: obj.field = value
    SetField {
        object: Expression,
        field: String,
        value: Expression,
        loc: Location,
    },
    /// 设置索引: obj[index] = value
    SetIndex {
        object: Expression,
        index: Expression,
        value: Expression,
        loc: Location,
    },
    Break(Location),
    Continue(Location),
    /// Try-Catch-Finally 异常处理
    TryCatch(TryCatch),
    /// Throw 抛出异常
    Throw {
        value: Expression,
        loc: Location,
    },
}

pub type Ast = Vec<Statement>;

/// 一元表达式
#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub operator: Operator,
    pub expr: Box<Expression>,
    pub loc: Location,
}

/// 赋值语句
#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    /// 变量名
    pub name: String,
    /// 赋值语句右边的表达式
    pub expr: Box<Expression>,
    pub loc: Location,
}

/// 循环语句
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    /// 循环终止判断条件
    pub test: Expression,
    /// 循环语句里面要执行的语句块
    pub body: Vec<Statement>,
    pub loc: Location,
}

/// For-In 循环语句
#[derive(Debug, PartialEq, Clone)]
pub struct ForInLoop {
    /// 循环变量名
    pub var: String,
    /// 可迭代对象表达式 (例如一个协程或数组)
    pub iterable: Expression,
    /// 循环体
    pub body: Vec<Statement>,
    pub loc: Location,
}

/// Try-Catch-Finally 异常处理
#[derive(Debug, PartialEq, Clone)]
pub struct TryCatch {
    /// try 块中的语句
    pub try_body: Vec<Statement>,
    /// catch 块中的错误变量名
    pub error_name: Option<String>,
    /// catch 块中的语句
    pub catch_body: Vec<Statement>,
    /// finally 块中的语句(可选)
    pub finally_body: Option<Vec<Statement>>,
    pub loc: Location,
}
