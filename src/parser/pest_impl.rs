//! Pest-based parser (optional, enabled with pest-parser feature)
//!
//! This module is only compiled when the `pest-parser` feature is enabled.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::expression::{
    Assign, Ast, BinaryOperation, Expression, ForOfLoop, FunctionCall,
    FunctionDeclaration, If, Literal, Local, Loop, Parameter,
    Return, Statement, TryCatch, TypeAnnotation, Unary,
};
use crate::tokenizer::{Location, Operator};
use crate::value::Value;

#[derive(Parser)]
#[grammar = "chen.pest"]
pub struct ChenLangParser;

pub fn parse(code: &str) -> Result<Ast, Box<pest::error::Error<Rule>>> {
    let pairs = ChenLangParser::parse(Rule::program, code)?;
    let mut ast = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for p in pair.into_inner() {
                    if p.as_rule() == Rule::statement {
                        ast.push(parse_statement(p));
                    }
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    Ok(ast)
}

fn loc_from_pair(pair: &Pair<Rule>) -> Location {
    let (line, col) = pair.as_span().start_pos().line_col();
    let index = pair.as_span().start();
    Location {
        col: col as u32,
        line: line as u32,
        index,
    }
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::declaration => parse_declaration(inner, loc),
        Rule::async_declaration => {
            let inner_decl = inner.into_inner().find(|p| p.as_rule() == Rule::declaration).unwrap();
            let mut stmt = parse_declaration(inner_decl, loc);
            if let Statement::Local(ref mut local) = stmt {
                if let Expression::Function(decl) = local.expression.clone() {
                    local.expression = Expression::AsyncFunction(decl);
                }
            }
            stmt
        }
        Rule::for_loop => parse_for_loop(inner, loc),
        Rule::while_loop => parse_while_loop(inner, loc),
        Rule::function_def => Statement::FunctionDeclaration(build_function_declaration(inner)),
        Rule::async_function_def => {
            let inner_func = inner.into_inner().find(|p| p.as_rule() == Rule::function_def).unwrap();
            Statement::AsyncFunctionDeclaration(build_function_declaration(inner_func))
        }
        Rule::return_stmt => parse_return_stmt(inner, loc),
        Rule::break_stmt => Statement::Break(loc),
        Rule::continue_stmt => Statement::Continue(loc),
        Rule::try_catch => parse_try_catch(inner),
        Rule::throw_stmt => parse_throw_stmt(inner),
        Rule::assignment => parse_assignment(inner, loc),
        Rule::expression => Statement::Expression(parse_expression(inner)),
        _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
    }
}

fn parse_declaration(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut inner = pair.into_inner();
    let _let_kw = inner.next();
    let name = inner.next().unwrap().as_str().to_string();

    let mut type_annotation = None;
    let mut next_opt = inner.next();

    if let Some(next) = next_opt {
        if next.as_rule() == Rule::type_expr {
            type_annotation = Some(parse_type_annotation(next));
            // skip assign
            inner.next();
        } else {
            // it was assign, already consumed
        }
    }

    let expr_pair = inner.next().expect("Expected expression in declaration");
    let expression = parse_expression(expr_pair);
    println!("Pest declaration: {} = {:?}", name, expression);

    Statement::Local(Local {
        name,
        type_annotation,
        expression,
        loc,
    })
}

fn parse_assignment(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut inner = pair.into_inner();
    let target_pair = inner.next().unwrap();
    let _assign_kw = inner.next();
    let value = parse_expression(inner.next().unwrap());

    let mut target_inner = target_pair.into_inner();
    let name = target_inner.next().unwrap().as_str().to_string();
    let mut expr = Expression::Identifier(name.clone(), loc);

    for postfix in target_inner {
        expr = parse_postfix(expr, postfix);
    }

    match expr {
        Expression::Identifier(name, loc) => Statement::Assign(Assign {
            name,
            expr: Box::new(value),
            loc,
        }),
        Expression::GetField { object, field, loc } => Statement::SetField {
            object: *object,
            field,
            value,
            loc,
        },
        Expression::Index { object, index, loc } => Statement::SetIndex {
            object: *object,
            index: *index,
            value,
            loc,
        },
        _ => unreachable!("Invalid assignment target"),
    }
}

fn parse_for_loop(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut var = String::new();
    let mut iterable = Expression::Literal(Literal::Value(Value::Null), loc);
    let mut body = Vec::new();
    let mut is_async = false;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::AWAIT => is_async = true,
            Rule::identifier => var = p.as_str().to_string(),
            Rule::expression => iterable = parse_expression(p),
            Rule::block => body = parse_block(p),
            _ => {}
        }
    }

    Statement::ForOf(ForOfLoop {
        var,
        iterable,
        body,
        is_async,
        loc,
    })
}

fn parse_while_loop(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut test = Expression::Literal(Literal::Value(Value::Bool(true)), loc);
    let mut body = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expression => test = parse_expression(p),
            Rule::block => body = parse_block(p),
            _ => {}
        }
    }

    Statement::Loop(Loop { test, body, loc })
}

fn build_function_declaration(pair: Pair<Rule>) -> FunctionDeclaration {
    let loc = loc_from_pair(&pair);
    let mut name = None;
    let mut name_loc = None;
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::identifier => {
                name = Some(p.as_str().to_string());
                name_loc = Some(loc_from_pair(&p));
            }
            Rule::parameters => {
                for param_pair in p.into_inner() {
                    let mut p_inner = param_pair.clone().into_inner();
                    let p_name = p_inner.next().unwrap().as_str().to_string();
                    let mut p_type = None;
                    if let Some(t_pair) = p_inner.next() {
                        p_type = Some(parse_type_annotation(t_pair));
                    }
                    parameters.push(Parameter {
                        name: p_name,
                        type_annotation: p_type,
                        loc: loc_from_pair(&param_pair),
                    });
                }
            }
            Rule::type_expr => {
                return_type = Some(parse_type_annotation(p));
            }
            Rule::block => body = parse_block(p),
            _ => {}
        }
    }

    FunctionDeclaration {
        name,
        parameters,
        return_type,
        body,
        loc: name_loc.unwrap_or(loc),
    }
}

fn parse_return_stmt(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut inner = pair.into_inner();
    inner.next(); // skip kw
    let expression = inner.next().map(parse_expression).unwrap_or(Expression::Literal(
        Literal::Value(Value::Null),
        loc,
    ));
    Statement::Return(Return { expression, loc })
}

fn parse_block(pair: Pair<Rule>) -> Vec<Statement> {
    let mut statements = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::statement {
            statements.push(parse_statement(p));
        }
    }
    statements
}

fn parse_expression(pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next().unwrap();
    parse_logical_or(inner)
}

fn parse_logical_or(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_logical_and(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let loc = loc_from_pair(&op);
        let right = parse_logical_and(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator: Operator::Or,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_logical_and(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_equality(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let loc = loc_from_pair(&op);
        let right = parse_equality(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator: Operator::And,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_equality(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_comparison(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let loc = loc_from_pair(&op_pair);
        let operator = match op_pair.as_rule() {
            Rule::eq => Operator::Equals,
            Rule::neq => Operator::NotEquals,
            _ => unreachable!(),
        };
        let right = parse_comparison(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_comparison(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_term(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let loc = loc_from_pair(&op_pair);
        let operator = match op_pair.as_rule() {
            Rule::lt => Operator::Lt,
            Rule::lte => Operator::LtE,
            Rule::gt => Operator::Gt,
            Rule::gte => Operator::GtE,
            _ => unreachable!(),
        };
        let right = parse_term(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_term(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_factor(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let loc = loc_from_pair(&op_pair);
        let operator = match op_pair.as_rule() {
            Rule::add => Operator::Add,
            Rule::subtract => Operator::Subtract,
            _ => unreachable!(),
        };
        let right = parse_factor(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_factor(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut left = parse_unary(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let loc = loc_from_pair(&op_pair);
        let operator = match op_pair.as_rule() {
            Rule::multiply => Operator::Multiply,
            Rule::divide => Operator::Divide,
            Rule::modulo => Operator::Mod,
            _ => unreachable!(),
        };
        let right = parse_unary(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_unary(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    match first.as_rule() {
        Rule::not => {
            let expr = parse_unary(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Not,
                expr: Box::new(expr),
                loc,
            })
        }
        Rule::subtract => {
            let expr = parse_unary(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Subtract,
                expr: Box::new(expr),
                loc,
            })
        }
        Rule::AWAIT => {
            let expr = parse_unary(inner.next().unwrap());
            Expression::Await {
                expression: Box::new(expr),
                loc,
            }
        }
        Rule::primary => parse_primary(first),
        _ => unreachable!(),
    }
}

fn parse_primary(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let atom_pair = inner.next().unwrap();
    let mut expr = parse_atom(atom_pair);

    for postfix in inner {
        expr = parse_postfix(expr, postfix);
    }
    expr
}

fn parse_atom(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer => Expression::Literal(Literal::Value(Value::int(inner.as_str().parse().unwrap())), loc),
        Rule::float => Expression::Literal(Literal::Value(Value::float(inner.as_str().parse().unwrap())), loc),
        Rule::bool => Expression::Literal(Literal::Value(Value::bool(inner.as_str() == "true")), loc),
        Rule::string => {
            let s = inner.as_str();
            Expression::Literal(Literal::Value(Value::string(s[1..s.len() - 1].to_string())), loc)
        }
        Rule::identifier => Expression::Identifier(inner.as_str().to_string(), loc),
        Rule::expression => parse_expression(inner),
        Rule::if_expr => parse_if_expr(inner),
        Rule::block => Expression::Block(parse_block(inner), loc),
        Rule::object_literal => parse_object_literal(inner),
        Rule::function_def => Expression::Function(build_function_declaration(inner)),
        Rule::async_function_def => {
            let inner_func = inner.into_inner().find(|p| p.as_rule() == Rule::function_def).unwrap();
            Expression::AsyncFunction(build_function_declaration(inner_func))
        }
        Rule::new_expr => parse_new_expr(inner),
        Rule::array_literal => parse_array_literal(inner),
        _ => unreachable!("Unexpected rule in atom: {:?}", inner.as_rule()),
    }
}

fn parse_new_expr(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let _new_kw = inner.next();
    let primary_pair = inner.next().unwrap();
    let constructor = parse_primary(primary_pair);

    let mut args = Vec::new();
    if let Some(args_pair) = inner.next() {
        for arg in args_pair.into_inner() {
            args.push(parse_expression(arg));
        }
    }

    Expression::New {
        constructor: Box::new(constructor),
        arguments: args,
        loc,
    }
}

fn parse_postfix(base: Expression, pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::call_suffix => {
            let mut args = Vec::new();
            if let Some(args_pair) = inner.into_inner().next() {
                for arg in args_pair.into_inner() {
                    args.push(parse_expression(arg));
                }
            }
            Expression::FunctionCall(FunctionCall {
                callee: Box::new(base),
                arguments: args,
                loc,
            })
        }
        Rule::dot_suffix => {
            let field_pair = inner.into_inner().next().unwrap();
            let field = field_pair.as_str().to_string();
            Expression::GetField {
                object: Box::new(base),
                field,
                loc,
            }
        }
        Rule::index_suffix => {
            let idx_expr = parse_expression(inner.into_inner().next().unwrap());
            Expression::Index {
                object: Box::new(base),
                index: Box::new(idx_expr),
                loc,
            }
        }
        _ => unreachable!("Unexpected rule in postfix"),
    }
}

fn parse_object_literal(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut fields = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::pair {
            let mut inner = p.into_inner();
            let key_pair = inner.next().unwrap();
            let key_loc = loc_from_pair(&key_pair);
            let key_expr = match key_pair.as_rule() {
                Rule::identifier => Expression::Literal(crate::expression::Literal::String(key_pair.as_str().to_string()), key_loc),
                Rule::string => {
                    let s = key_pair.as_str();
                    Expression::Literal(crate::expression::Literal::String(s[1..s.len() - 1].to_string()), key_loc)
                }
                Rule::integer => Expression::Literal(crate::expression::Literal::String(key_pair.as_str().to_string()), key_loc),
                Rule::computed_property => parse_expression(key_pair.into_inner().next().unwrap()),
                _ => unreachable!(),
            };
            let value = parse_expression(inner.next().unwrap());
            fields.push((key_expr, value));
        }
    }
    Expression::ObjectLiteral(fields, loc)
}

fn parse_array_literal(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut elements = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            elements.push(parse_expression(p));
        }
    }
    Expression::ArrayLiteral(elements, loc)
}

fn parse_if_expr(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let _if_kw = inner.next();
    let test = parse_expression(inner.next().unwrap());
    let body = parse_block(inner.next().unwrap());

    let mut else_body = Vec::new();
    if let Some(_) = inner.next() {
        // ELSE followed by (if_expr | block)
        let next = inner.next().unwrap();
        match next.as_rule() {
            Rule::if_expr => {
                else_body.push(Statement::Expression(parse_if_expr(next)));
            }
            Rule::block => {
                else_body = parse_block(next);
            }
            _ => unreachable!(),
        }
    }

    Expression::If(If {
        test: Box::new(test),
        body,
        else_body,
        loc,
    })
}

fn parse_try_catch(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let items: Vec<_> = pair
        .into_inner()
        .filter(|p| !matches!(p.as_rule(), Rule::TRY | Rule::CATCH | Rule::FINALLY))
        .collect();

    let mut iter = items.into_iter();
    let try_body = parse_block(iter.next().unwrap());

    let mut error_name = None;
    let mut catch_body = Vec::new();

    if let Some(next_pair) = iter.next() {
        match next_pair.as_rule() {
            Rule::identifier => {
                error_name = Some(next_pair.as_str().to_string());
                if let Some(catch_block_pair) = iter.next() {
                    catch_body = parse_block(catch_block_pair);
                }
            }
            Rule::block => {
                catch_body = parse_block(next_pair);
            }
            _ => {}
        }
    }

    let finally_body = iter.next().map(parse_block);

    Statement::TryCatch(TryCatch {
        try_body,
        error_name,
        catch_body,
        finally_body,
        loc,
    })
}

fn parse_throw_stmt(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let _throw_kw = inner.next();
    let value = parse_expression(inner.next().unwrap());
    Statement::Throw { value, loc }
}

fn parse_type_annotation(pair: Pair<Rule>) -> TypeAnnotation {
    let inner = pair.into_inner().next().unwrap();
    parse_type_union(inner)
}

fn parse_type_union(pair: Pair<Rule>) -> TypeAnnotation {
    let mut inner = pair.into_inner();
    let mut annotations = Vec::new();

    while let Some(p) = inner.next() {
        annotations.push(parse_type_primary(p));
        // terminals like '|' are skipped in into_inner() for non-atomic rules!
    }

    if annotations.len() == 1 {
        annotations.remove(0)
    } else {
        TypeAnnotation::Union(annotations)
    }
}

fn parse_type_primary(pair: Pair<Rule>) -> TypeAnnotation {
    let s = pair.as_str();
    match s {
        "int" => return TypeAnnotation::Int,
        "float" => return TypeAnnotation::Float,
        "bool" => return TypeAnnotation::Bool,
        "string" => return TypeAnnotation::String,
        "object" => return TypeAnnotation::Object,
        "null" => return TypeAnnotation::Null,
        _ => {}
    }

    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::type_generic => {
            let mut g_inner = inner.into_inner();
            let name = g_inner.next().unwrap().as_str().to_string();
            let mut arguments = Vec::new();
            for arg in g_inner {
                arguments.push(parse_type_annotation(arg));
            }
            TypeAnnotation::Generic { name, arguments }
        }
        Rule::identifier => {
            let s = inner.as_str();
            match s {
                "int" => TypeAnnotation::Int,
                "float" => TypeAnnotation::Float,
                "bool" => TypeAnnotation::Bool,
                "string" => TypeAnnotation::String,
                "object" => TypeAnnotation::Object,
                "null" => TypeAnnotation::Null,
                _ => TypeAnnotation::Named(s.to_string()),
            }
        }
        _ => unreachable!("Unexpected type primary: {:?}", inner.as_rule()),
    }
}
