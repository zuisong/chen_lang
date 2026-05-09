use std::collections::HashMap;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use thiserror::Error;

use crate::expression::{
    Ast, BinaryOperation, Expression, FunctionDeclaration, Literal, Local, Return, Statement, TypeAnnotation, Unary,
};
use crate::tokenizer::{Location, Operator};
use crate::value::Value;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Object,
    Null,
    Unknown,
    TypeVariable(String),
    Generic {
        name: String,
        arguments: Vec<Type>,
    },
    Union(Vec<Type>),
    TypeAlias(String),
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
}

impl Type {
    fn from_annotation(annotation: &TypeAnnotation) -> Self {
        match annotation {
            TypeAnnotation::Int => Type::Int,
            TypeAnnotation::Float => Type::Float,
            TypeAnnotation::Bool => Type::Bool,
            TypeAnnotation::String => Type::String,
            TypeAnnotation::Object => Type::Object,
            TypeAnnotation::Null => Type::Null,
            TypeAnnotation::Named(name) if is_type_variable_name(name) => Type::TypeVariable(name.clone()),
            TypeAnnotation::Named(name) => Type::TypeAlias(name.clone()),
            TypeAnnotation::Generic { name, arguments } => Type::Generic {
                name: name.clone(),
                arguments: arguments.iter().map(Type::from_annotation).collect(),
            },
            TypeAnnotation::Union(types) => Type::Union(types.iter().map(Type::from_annotation).collect()),
            TypeAnnotation::TypeAlias(name) => Type::TypeAlias(name.clone()),
        }
    }

    fn can_assign_to(&self, expected: &Type) -> bool {
        if matches!(self, Type::Unknown) || matches!(expected, Type::Unknown) || self == expected {
            return true;
        }

        match (self, expected) {
            (found, Type::Union(types)) => types.iter().any(|candidate| found.can_assign_to(candidate)),
            (Type::Union(types), expected) => types.iter().all(|candidate| candidate.can_assign_to(expected)),
            (
                Type::Generic {
                    name: found_name,
                    arguments: found_args,
                },
                Type::Generic {
                    name: expected_name,
                    arguments: expected_args,
                },
            ) => {
                found_name == expected_name
                    && found_args.len() == expected_args.len()
                    && found_args
                        .iter()
                        .zip(expected_args)
                        .all(|(found, expected)| found.can_assign_to(expected))
            }
            (_, Type::TypeVariable(_)) | (Type::TypeVariable(_), _) => true,
            _ => false,
        }
    }
}

fn is_type_variable_name(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase) && name.len() == 1
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Object => write!(f, "object"),
            Type::Null => write!(f, "null"),
            Type::Unknown => write!(f, "unknown"),
            Type::TypeVariable(name) | Type::TypeAlias(name) => write!(f, "{name}"),
            Type::Generic { name, arguments } => {
                let args = arguments.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                write!(f, "{name}<{args}>")
            }
            Type::Union(types) => {
                let variants = types.iter().map(ToString::to_string).collect::<Vec<_>>().join(" | ");
                write!(f, "{variants}")
            }
            Type::Function {
                parameters,
                return_type,
            } => {
                let params = parameters
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({params}) -> {return_type}")
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: Type, found: Type, loc: Location },
    #[error("Return type mismatch: expected {expected}, found {found}")]
    ReturnTypeMismatch { expected: Type, found: Type, loc: Location },
    #[error("Argument type mismatch for parameter {index}: expected {expected}, found {found}")]
    ArgumentTypeMismatch {
        index: usize,
        expected: Type,
        found: Type,
        loc: Location,
    },
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    type_aliases: HashMap<String, Type>,
    current_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            type_aliases: HashMap::new(),
            current_return_type: None,
        }
    }

    pub fn check(&mut self, ast: &Ast) -> Result<(), TypeError> {
        for statement in ast {
            if let Statement::TypeAliasDeclaration(alias) = statement {
                let target = self.resolve_aliases(&Type::from_annotation(&alias.target));
                self.type_aliases.insert(alias.name.clone(), target);
            }
        }

        for statement in ast {
            if let Statement::FunctionDeclaration(function) = statement
                && let Some(name) = &function.name
            {
                self.define(name.clone(), self.function_signature(function));
            }
        }

        self.check_statements(ast)
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Result<(), TypeError> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), TypeError> {
        match statement {
            Statement::Local(local) => self.check_local(local),
            Statement::Assign(assign) => {
                let found = self.check_expression(&assign.expr)?;
                if let Some(expected) = self.resolve(&assign.name)
                    && !found.can_assign_to(&expected)
                {
                    return Err(TypeError::TypeMismatch {
                        expected,
                        found,
                        loc: assign.loc,
                    });
                }
                Ok(())
            }
            Statement::FunctionDeclaration(function) => self.check_function(function),
            Statement::TypeAliasDeclaration(_) => Ok(()),
            Statement::Return(ret) => self.check_return(ret),
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Statement::Loop(loop_) => {
                self.check_expression(&loop_.test)?;
                self.with_scope(|checker| checker.check_statements(&loop_.body))
            }
            Statement::ForIn(for_in) => {
                self.check_expression(&for_in.iterable)?;
                self.with_scope(|checker| {
                    checker.define(for_in.var.clone(), Type::Unknown);
                    checker.check_statements(&for_in.body)
                })
            }
            Statement::SetField { object, value, .. } => {
                self.check_expression(object)?;
                self.check_expression(value)?;
                Ok(())
            }
            Statement::SetIndex {
                object, index, value, ..
            } => {
                self.check_expression(object)?;
                self.check_expression(index)?;
                self.check_expression(value)?;
                Ok(())
            }
            Statement::Break(_) | Statement::Continue(_) => Ok(()),
            Statement::TryCatch(try_catch) => {
                self.with_scope(|checker| checker.check_statements(&try_catch.try_body))?;
                self.with_scope(|checker| {
                    if let Some(error_name) = &try_catch.error_name {
                        checker.define(error_name.clone(), Type::Unknown);
                    }
                    checker.check_statements(&try_catch.catch_body)
                })?;
                if let Some(finally_body) = &try_catch.finally_body {
                    self.with_scope(|checker| checker.check_statements(finally_body))?;
                }
                Ok(())
            }
            Statement::Throw { value, .. } => {
                self.check_expression(value)?;
                Ok(())
            }
        }
    }

    fn check_local(&mut self, local: &Local) -> Result<(), TypeError> {
        let found = self.check_expression(&local.expression)?;
        let declared = local
            .type_annotation
            .as_ref()
            .map(Type::from_annotation)
            .map(|ty| self.resolve_aliases(&ty));
        let variable_type = declared.clone().unwrap_or_else(|| found.clone());

        if let Some(expected) = declared
            && !found.can_assign_to(&expected)
        {
            return Err(TypeError::TypeMismatch {
                expected,
                found,
                loc: local.loc,
            });
        }

        self.define(local.name.clone(), variable_type);
        Ok(())
    }

    fn check_function(&mut self, function: &FunctionDeclaration) -> Result<(), TypeError> {
        self.with_scope(|checker| {
            for parameter in &function.parameters {
                let parameter_type = parameter
                    .type_annotation
                    .as_ref()
                    .map(Type::from_annotation)
                    .map(|ty| checker.resolve_aliases(&ty))
                    .unwrap_or(Type::Unknown);
                checker.define(parameter.name.clone(), parameter_type);
            }

            let previous_return_type = checker.current_return_type.clone();
            checker.current_return_type = function
                .return_type
                .as_ref()
                .map(Type::from_annotation)
                .map(|ty| checker.resolve_aliases(&ty));
            let result = checker.check_statements(&function.body);
            checker.current_return_type = previous_return_type;
            result
        })
    }

    fn check_return(&mut self, ret: &Return) -> Result<(), TypeError> {
        let found = self.check_expression(&ret.expression)?;
        if let Some(expected) = &self.current_return_type
            && !found.can_assign_to(expected)
        {
            return Err(TypeError::ReturnTypeMismatch {
                expected: expected.clone(),
                found,
                loc: ret.loc,
            });
        }
        Ok(())
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, TypeError> {
        match expression {
            Expression::Literal(Literal::Value(value), _) => Ok(self.value_type(value)),
            Expression::Identifier(name, _) => Ok(self.resolve(name).unwrap_or(Type::Unknown)),
            Expression::BinaryOperation(operation) => self.check_binary_operation(operation),
            Expression::Unary(unary) => self.check_unary(unary),
            Expression::FunctionCall(function_call) => {
                let callee_type = self.check_expression(&function_call.callee)?;
                let argument_types = function_call
                    .arguments
                    .iter()
                    .map(|argument| self.check_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;

                if let Type::Function {
                    parameters,
                    return_type,
                } = callee_type
                {
                    let mut substitutions = HashMap::new();
                    for (index, (found, expected)) in argument_types.iter().zip(parameters.iter()).enumerate() {
                        let expected = self.apply_substitutions(expected, &substitutions);
                        if !found.can_assign_to(&expected) {
                            return Err(TypeError::ArgumentTypeMismatch {
                                index,
                                expected,
                                found: found.clone(),
                                loc: function_call.loc,
                            });
                        }
                        self.bind_type_variables(&expected, found, &mut substitutions);
                    }
                    Ok(self.apply_substitutions(&return_type, &substitutions))
                } else {
                    Ok(Type::Unknown)
                }
            }
            Expression::MethodCall(method_call) => {
                self.check_expression(&method_call.object)?;
                for argument in &method_call.arguments {
                    self.check_expression(argument)?;
                }
                Ok(Type::Unknown)
            }
            Expression::Block(statements, _) => self.with_scope(|checker| {
                checker.check_statements(statements)?;
                Ok(Type::Unknown)
            }),
            Expression::If(if_expr) => {
                self.check_expression(&if_expr.test)?;
                self.with_scope(|checker| checker.check_statements(&if_expr.body))?;
                self.with_scope(|checker| checker.check_statements(&if_expr.else_body))?;
                Ok(Type::Unknown)
            }
            Expression::ObjectLiteral(fields, _) => {
                for (_, value) in fields {
                    self.check_expression(value)?;
                }
                Ok(Type::Object)
            }
            Expression::ArrayLiteral(elements, _) => {
                let mut element_type = Type::Unknown;
                for element in elements {
                    let current = self.check_expression(element)?;
                    element_type = self.merge_types(&element_type, &current);
                }
                Ok(Type::Generic {
                    name: "Array".to_string(),
                    arguments: vec![element_type],
                })
            }
            Expression::GetField { object, .. } => {
                self.check_expression(object)?;
                Ok(Type::Unknown)
            }
            Expression::Index { object, index, .. } => {
                self.check_expression(object)?;
                self.check_expression(index)?;
                Ok(Type::Unknown)
            }
            Expression::Function(function) => {
                self.check_function(function)?;
                Ok(self.function_signature(function))
            }
            Expression::Import { .. } => Ok(Type::Unknown),
        }
    }

    fn check_binary_operation(&mut self, operation: &BinaryOperation) -> Result<Type, TypeError> {
        let left = self.check_expression(&operation.left)?;
        let right = self.check_expression(&operation.right)?;
        let ty = match operation.operator {
            Operator::Add => {
                if left == Type::String || right == Type::String {
                    Type::String
                } else {
                    self.numeric_result(&left, &right)
                }
            }
            Operator::Subtract | Operator::Multiply | Operator::Divide | Operator::Mod => {
                self.numeric_result(&left, &right)
            }
            Operator::Equals
            | Operator::NotEquals
            | Operator::Lt
            | Operator::LtE
            | Operator::Gt
            | Operator::GtE
            | Operator::And
            | Operator::Or => Type::Bool,
            Operator::Assign | Operator::Not => Type::Unknown,
        };
        Ok(ty)
    }

    fn check_unary(&mut self, unary: &Unary) -> Result<Type, TypeError> {
        self.check_expression(&unary.expr)?;
        match unary.operator {
            Operator::Not => Ok(Type::Bool),
            _ => Ok(Type::Unknown),
        }
    }

    fn numeric_result(&self, left: &Type, right: &Type) -> Type {
        if matches!(left, Type::Unknown) || matches!(right, Type::Unknown) {
            Type::Unknown
        } else if matches!(left, Type::Float) || matches!(right, Type::Float) {
            Type::Float
        } else if matches!(left, Type::Int) && matches!(right, Type::Int) {
            Type::Int
        } else {
            Type::Unknown
        }
    }

    fn function_signature(&self, function: &FunctionDeclaration) -> Type {
        Type::Function {
            parameters: function
                .parameters
                .iter()
                .map(|parameter| {
                    let ty = parameter
                        .type_annotation
                        .as_ref()
                        .map(Type::from_annotation)
                        .unwrap_or_else(|| Type::TypeVariable(parameter.name.clone()));
                    self.resolve_aliases(&ty)
                })
                .collect(),
            return_type: Box::new(
                self.resolve_aliases(&function
                    .return_type
                    .as_ref()
                    .map(Type::from_annotation)
                    .unwrap_or_else(|| self.infer_function_return_type(function))),
            ),
        }
    }

    fn infer_function_return_type(&self, function: &FunctionDeclaration) -> Type {
        if let Some(Statement::Return(ret)) = function.body.iter().find(|statement| matches!(statement, Statement::Return(_)))
        {
            return self.infer_expression_shallow(&ret.expression, function);
        }

        if let Some(Statement::Expression(expression)) = function.body.last() {
            return self.infer_expression_shallow(expression, function);
        }

        Type::Unknown
    }

    fn infer_expression_shallow(&self, expression: &Expression, function: &FunctionDeclaration) -> Type {
        match expression {
            Expression::Identifier(name, _) => function
                .parameters
                .iter()
                .find(|parameter| &parameter.name == name)
                .map(|parameter| {
                    parameter
                        .type_annotation
                        .as_ref()
                        .map(Type::from_annotation)
                        .unwrap_or_else(|| Type::TypeVariable(parameter.name.clone()))
                })
                .unwrap_or(Type::Unknown),
            Expression::Literal(Literal::Value(value), _) => self.value_type(value),
            Expression::ArrayLiteral(elements, _) => {
                let element_type = elements
                    .iter()
                    .map(|element| self.infer_expression_shallow(element, function))
                    .fold(Type::Unknown, |acc, ty| self.merge_types(&acc, &ty));
                Type::Generic {
                    name: "Array".to_string(),
                    arguments: vec![element_type],
                }
            }
            _ => Type::Unknown,
        }
    }

    fn resolve_aliases(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeAlias(name) => self
                .type_aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| Type::TypeAlias(name.clone())),
            Type::Generic { name, arguments } => Type::Generic {
                name: name.clone(),
                arguments: arguments.iter().map(|arg| self.resolve_aliases(arg)).collect(),
            },
            Type::Union(types) => Type::Union(types.iter().map(|ty| self.resolve_aliases(ty)).collect()),
            Type::Function {
                parameters,
                return_type,
            } => Type::Function {
                parameters: parameters.iter().map(|ty| self.resolve_aliases(ty)).collect(),
                return_type: Box::new(self.resolve_aliases(return_type)),
            },
            _ => ty.clone(),
        }
    }

    fn merge_types(&self, left: &Type, right: &Type) -> Type {
        if matches!(left, Type::Unknown) {
            return right.clone();
        }
        if matches!(right, Type::Unknown) || left == right {
            return left.clone();
        }
        Type::Union(vec![left.clone(), right.clone()])
    }

    fn bind_type_variables(&self, expected: &Type, found: &Type, substitutions: &mut HashMap<String, Type>) {
        match (expected, found) {
            (Type::TypeVariable(name), found) => {
                substitutions.entry(name.clone()).or_insert_with(|| found.clone());
            }
            (
                Type::Generic {
                    arguments: expected_args,
                    ..
                },
                Type::Generic {
                    arguments: found_args, ..
                },
            ) => {
                for (expected, found) in expected_args.iter().zip(found_args) {
                    self.bind_type_variables(expected, found, substitutions);
                }
            }
            (Type::Union(expected), Type::Union(found)) => {
                for (expected, found) in expected.iter().zip(found) {
                    self.bind_type_variables(expected, found, substitutions);
                }
            }
            _ => {}
        }
    }

    fn apply_substitutions(&self, ty: &Type, substitutions: &HashMap<String, Type>) -> Type {
        match ty {
            Type::TypeVariable(name) => substitutions.get(name).cloned().unwrap_or_else(|| ty.clone()),
            Type::Generic { name, arguments } => Type::Generic {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.apply_substitutions(argument, substitutions))
                    .collect(),
            },
            Type::Union(types) => Type::Union(
                types
                    .iter()
                    .map(|ty| self.apply_substitutions(ty, substitutions))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    fn value_type(&self, value: &Value) -> Type {
        match value {
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::String(_) => Type::String,
            Value::Object(_) => Type::Object,
            Value::Null => Type::Null,
            Value::Fn(_) | Value::NativeFunction(_) | Value::Coroutine(_) => Type::Unknown,
        }
    }

    fn define(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn resolve(&self, name: &str) -> Option<Type> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name).cloned())
    }

    fn with_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, TypeError>) -> Result<T, TypeError> {
        self.scopes.push(HashMap::new());
        let result = f(self);
        self.scopes.pop();
        result
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check(ast: &Ast) -> Result<(), TypeError> {
    TypeChecker::new().check(ast)
}

pub fn diagnostic(code: &str, file_id: usize, error: &TypeError) -> Diagnostic<usize> {
    let loc = match error {
        TypeError::TypeMismatch { loc, .. }
        | TypeError::ReturnTypeMismatch { loc, .. }
        | TypeError::ArgumentTypeMismatch { loc, .. } => *loc,
    };
    let range = get_line_range(code, loc.line);
    Diagnostic::error()
        .with_message(error.to_string())
        .with_labels(vec![Label::primary(file_id, range).with_message("Type error here")])
}

fn get_line_range(code: &str, line: u32) -> std::ops::Range<usize> {
    if line == 0 {
        return 0..code.len();
    }

    let mut current_line = 1;
    let mut start_byte = 0;
    for line_str in code.split_inclusive('\n') {
        if current_line == line {
            let len = line_str.trim_end().len();
            let end_byte = if len == 0 { start_byte + 1 } else { start_byte + len };
            return start_byte..std::cmp::min(end_byte, code.len());
        }
        start_byte += line_str.len();
        current_line += 1;
    }

    let len = code.len();
    if len > 0 { len - 1..len } else { 0..0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn accepts_matching_local_annotation() {
        let ast = parser::parse_from_source("let x: int = 10").unwrap();
        assert!(check(&ast).is_ok());
    }

    #[test]
    fn rejects_mismatched_local_annotation() {
        let ast = parser::parse_from_source("let x: string = 10").unwrap();
        assert!(matches!(
            check(&ast),
            Err(TypeError::TypeMismatch {
                expected: Type::String,
                found: Type::Int,
                ..
            })
        ));
    }

    #[test]
    fn checks_function_return_annotation() {
        let ast = parser::parse_from_source("def answer() -> int { return \"no\" }").unwrap();
        assert!(matches!(
            check(&ast),
            Err(TypeError::ReturnTypeMismatch {
                expected: Type::Int,
                found: Type::String,
                ..
            })
        ));
    }
}
