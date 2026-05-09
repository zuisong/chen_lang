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
pub struct StructExpression {
    pub name: String,
    pub fields: Vec<(String, Expression)>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expression: Expression,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchExpression {
    pub value: Box<Expression>,
    pub arms: Vec<MatchArm>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Wildcard(Location),
    Binding(String, Location),
    Literal(Literal, Location),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
        loc: Location,
    },
    EnumVariant {
        enum_name: Option<String>,
        variant: String,
        inner: Option<Box<Pattern>>,
        loc: Location,
    },
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
    /// 结构体构造: Point { x: 1, y: 2 }
    StructLiteral(StructExpression),
    /// match 表达式
    Match(MatchExpression),
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
    /// Import 表达式: import("path")
    Import {
        path: String,
        loc: Location,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeAnnotation {
    Int,
    Float,
    Bool,
    String,
    Object,
    Null,
    Named(String),
    Generic {
        name: String,
        arguments: Vec<TypeAnnotation>,
    },
    Union(Vec<TypeAnnotation>),
    TypeAlias(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: Option<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
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
    pub type_annotation: Option<TypeAnnotation>,
    pub expression: Expression,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeAliasDeclaration {
    pub name: String,
    pub target: TypeAnnotation,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDeclaration {
    pub name: String,
    pub fields: Vec<(String, TypeAnnotation)>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariantDeclaration {
    pub name: String,
    pub payload: Option<TypeAnnotation>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDeclaration {
    pub name: String,
    pub type_parameters: Vec<String>,
    pub variants: Vec<EnumVariantDeclaration>,
    pub loc: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImplDeclaration {
    pub target: String,
    pub methods: Vec<FunctionDeclaration>,
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
    TypeAliasDeclaration(TypeAliasDeclaration),
    StructDeclaration(StructDeclaration),
    EnumDeclaration(EnumDeclaration),
    ImplDeclaration(ImplDeclaration),
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
