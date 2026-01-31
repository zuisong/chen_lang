//! Pest-based parser (optional, enabled with pest-parser feature)
//!
//! This module is only compiled when the `pest-parser` feature is enabled.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::expression::*;
use crate::tokenizer::Location;
use crate::tokenizer::Operator;
use crate::value::Value;

#[derive(Parser)]
#[grammar = "chen.pest"]
pub struct ChenLangParser;

pub fn parse(input: &str) -> Result<Ast, Box<pest::error::Error<Rule>>> {
    let pairs = ChenLangParser::parse(Rule::program, input)?;
    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::statement => {
                        statements.push(parse_statement(inner_pair));
                    }
                    Rule::EOI => (),
                    _ => unreachable!("Unexpected rule in program: {:?}", inner_pair.as_rule()),
                }
            }
        }
    }

    Ok(statements)
}

fn loc_from_pair(pair: &Pair<Rule>) -> Location {
    let (line, col) = pair.as_span().start_pos().line_col();
    let index = pair.as_span().start();
    Location {
        line: line as u32,
        col: col as u32,
        index,
    }
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::declaration => parse_declaration(inner, loc),
        Rule::assignment => parse_assignment(inner, loc),
        Rule::for_loop => parse_for_loop(inner, loc),
        Rule::function_def => parse_function_def(inner, loc),
        Rule::return_stmt => parse_return_stmt(inner, loc),
        Rule::break_stmt => Statement::Break(loc),
        Rule::continue_stmt => Statement::Continue(loc),
        Rule::try_catch => parse_try_catch(inner),
        Rule::throw_stmt => parse_throw_stmt(inner),
        Rule::expression => Statement::Expression(parse_expression(inner)),
        _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
    }
}

fn parse_declaration(pair: Pair<Rule>, loc: Location) -> Statement {
    let inner = pair.into_inner();

    let mut name = String::new();
    let mut name_loc = None;
    let mut expr = Expression::Literal(Literal::Value(Value::Null), loc);

    for p in inner {
        match p.as_rule() {
            Rule::identifier => {
                name = p.as_str().to_string();
                name_loc = Some(loc_from_pair(&p));
            }
            Rule::expression => expr = parse_expression(p),
            Rule::LET | Rule::assign => {} // skip keywords/ops
            _ => unreachable!("Unexpected rule in declaration: {:?}", p.as_rule()),
        }
    }

    Statement::Local(Local {
        name,
        expression: expr,
        loc: name_loc.unwrap_or(loc),
    })
}

fn parse_assignment(pair: Pair<Rule>, loc: Location) -> Statement {
    // assignment = { assignment_target ~ assign ~ expression }
    // assignment_target = { identifier ~ postfix* }
    let mut inner = pair.into_inner();
    let target_pair = inner.next().unwrap();
    let mut expr = Expression::Literal(Literal::Value(Value::Null), loc);

    let lvalue = parse_assignment_target(target_pair);

    for p in inner {
        if p.as_rule() == Rule::expression {
            expr = parse_expression(p);
        }
    }

    match lvalue {
        Expression::Identifier(name, loc) => Statement::Assign(Assign {
            name,
            expr: Box::new(expr),
            loc,
        }),
        Expression::GetField { object, field, loc } => Statement::SetField {
            object: *object,
            field,
            value: expr,
            loc,
        },
        Expression::Index { object, index, loc } => Statement::SetIndex {
            object: *object,
            index: *index,
            value: expr,
            loc,
        },
        _ => unreachable!("Invalid l-value in assignment: {:?}", lvalue),
    }
}

fn parse_assignment_target(pair: Pair<Rule>) -> Expression {
    // assignment_target = { identifier ~ postfix* }
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let identifier_pair = inner.next().unwrap();
    let mut expr = Expression::Identifier(identifier_pair.as_str().to_string(), loc);

    for p in inner {
        match p.as_rule() {
            Rule::postfix => expr = parse_postfix(expr, p),
            _ => unreachable!("Unexpected rule in assignment_target"),
        }
    }
    expr
}

fn parse_for_loop(pair: Pair<Rule>, loc: Location) -> Statement {
    let inner = pair.into_inner();
    let mut test = None;
    let mut for_in = None;
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::expression => test = Some(parse_expression(p)),
            Rule::for_in_header => {
                let mut header_inner = p.into_inner();
                let var = header_inner.next().unwrap().as_str().to_string();
                let iterable = parse_expression(header_inner.next().unwrap());
                for_in = Some((var, iterable));
            }
            Rule::block => body = parse_block(p),
            Rule::FOR | Rule::IN => {}
            _ => unreachable!("Unexpected rule in for_loop: {:?}", p.as_rule()),
        }
    }

    if let Some((var, iterable)) = for_in {
        Statement::ForIn(ForInLoop {
            var,
            iterable,
            body,
            loc,
        })
    } else {
        Statement::Loop(Loop {
            test: test.unwrap_or(Expression::Literal(Literal::Value(Value::Bool(true)), loc)),
            body,
            loc,
        })
    }
}

fn build_function_declaration(pair: Pair<Rule>) -> FunctionDeclaration {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner();
    let mut name = None;
    let mut name_loc = None;
    let mut parameters = Vec::new();
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::identifier => {
                name = Some(p.as_str().to_string());
                name_loc = Some(loc_from_pair(&p));
            }
            Rule::parameters => {
                for param in p.into_inner() {
                    parameters.push(param.as_str().to_string());
                }
            }
            Rule::block => body = parse_block(p),
            Rule::DEF => {}
            _ => unreachable!("Unexpected rule in function_def: {:?}", p.as_rule()),
        }
    }

    FunctionDeclaration {
        name,
        parameters,
        body,
        loc: name_loc.unwrap_or(loc),
    }
}

fn parse_function_def(pair: Pair<Rule>, _loc: Location) -> Statement {
    // The line from statement is passed but FunctionDeclaration also has its own line derived inside.
    // We can use the passed line or derive again. Derive is cleaner for helper.
    let decl = build_function_declaration(pair);
    Statement::FunctionDeclaration(decl)
}

fn parse_return_stmt(pair: Pair<Rule>, loc: Location) -> Statement {
    let mut expr = Expression::Literal(Literal::Value(Value::Null), loc);
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            expr = parse_expression(p);
        }
    }
    Statement::Return(Return { expression: expr, loc })
}

fn parse_block(pair: Pair<Rule>) -> Vec<Statement> {
    let mut stmts = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::statement {
            stmts.push(parse_statement(p));
        }
    }
    stmts
}

// Expression parsing logic

fn parse_expression(pair: Pair<Rule>) -> Expression {
    // expression = { logical_or }
    // Check if there's an inner element, if not, treat current pair as logical_or
    let mut inner = pair.clone().into_inner();
    if let Some(first) = inner.next() {
        parse_logical_or(first)
    } else {
        // If no inner element, the pair itself might be the expression content
        parse_logical_or(pair)
    }
}

fn parse_binary_op<F>(pair: Pair<Rule>, parse_sub: F) -> Expression
where
    F: Fn(Pair<Rule>) -> Expression,
{
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();

    let first = match inner.next() {
        Some(p) => p,
        None => {
            // This should not happen with correct grammar, but handle gracefully
            return Expression::Literal(Literal::Value(Value::Null), loc);
        }
    };

    let mut left = parse_sub(first);

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "*" => Operator::Multiply,
            "/" => Operator::Divide,
            "%" => Operator::Mod,
            "==" => Operator::Equals,
            "!=" => Operator::NotEquals,
            "<" => Operator::Lt,
            "<=" => Operator::LtE,
            ">" => Operator::Gt,
            ">=" => Operator::GtE,
            "&&" => Operator::And,
            "||" => Operator::Or,
            _ => unreachable!("Unknown operator: {}", op_pair.as_str()),
        };
        let right_pair = inner.next().unwrap();
        let right = parse_sub(right_pair);

        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
            loc,
        });
    }
    left
}

fn parse_logical_or(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_logical_and)
}

fn parse_logical_and(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_equality)
}

fn parse_equality(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_comparison)
}

fn parse_comparison(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_term)
}

fn parse_term(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_factor)
}

fn parse_factor(pair: Pair<Rule>) -> Expression {
    parse_binary_op(pair, parse_unary)
}

fn parse_unary(pair: Pair<Rule>) -> Expression {
    // unary = { (not | subtract) ~ unary | primary }
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
            // -x is 0 - x
            Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), loc)),
                operator: Operator::Subtract,
                right: Box::new(expr),
                loc,
            })
        }
        Rule::primary => parse_primary(first),
        _ => unreachable!("Unexpected rule in unary: {:?}", first.as_rule()),
    }
}

fn parse_primary(pair: Pair<Rule>) -> Expression {
    // primary = { atom ~ postfix* }
    let mut inner = pair.into_inner();
    let atom_pair = inner.next().unwrap();
    let mut expr = parse_atom(atom_pair);

    for p in inner {
        match p.as_rule() {
            Rule::postfix => expr = parse_postfix(expr, p),
            _ => unreachable!("Unexpected rule in primary"),
        }
    }
    expr
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

fn parse_atom(pair: Pair<Rule>) -> Expression {
    // atom = { float | integer | bool | string | identifier | "(" ~ expression ~ ")" | if_expr | block | object_literal }
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::float => Expression::Literal(Literal::Value(Value::Float(inner.as_str().parse().unwrap())), loc),
        Rule::integer => Expression::Literal(Literal::Value(Value::Int(inner.as_str().parse().unwrap())), loc),
        Rule::bool => Expression::Literal(Literal::Value(Value::Bool(inner.as_str() == "true")), loc),
        Rule::string => {
            let s = inner.as_str();
            // TODO: Better string unescaping
            let content = &s[1..s.len() - 1];
            Expression::Literal(Literal::Value(Value::string(content.to_string())), loc)
        }
        Rule::identifier => Expression::Identifier(inner.as_str().to_string(), loc),
        Rule::expression => parse_expression(inner), // ( expr )
        Rule::if_expr => parse_if_expr(inner),
        Rule::block => Expression::Block(parse_block(inner), loc),
        Rule::object_literal => parse_object_literal(inner),
        Rule::function_def => Expression::Function(build_function_declaration(inner)),
        Rule::array_literal => parse_array_literal(inner),
        Rule::import_expr => parse_import_expr(inner),
        _ => unreachable!("Unexpected rule in atom: {:?}", inner.as_rule()),
    }
}

// parse_await_expr removed

fn parse_postfix(base: Expression, pair: Pair<Rule>) -> Expression {
    // postfix = { call_suffix | dot_suffix | index_suffix }
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::call_suffix => {
            // call_suffix = { "(" ~ arguments? ~ ")" }
            let mut args = Vec::new();
            for p in inner.into_inner() {
                if p.as_rule() == Rule::arguments {
                    for arg in p.into_inner() {
                        args.push(parse_expression(arg));
                    }
                }
            }

            Expression::FunctionCall(FunctionCall {
                callee: Box::new(base),
                arguments: args,
                loc,
            })
        }
        Rule::dot_suffix => {
            // dot_suffix = { "." ~ identifier }
            let field = inner.into_inner().next().unwrap().as_str().to_string();
            Expression::GetField {
                object: Box::new(base),
                field,
                loc,
            }
        }
        Rule::index_suffix => {
            // index_suffix = { "[" ~ expression ~ "]" }
            let idx_expr = parse_expression(inner.into_inner().next().unwrap());
            Expression::Index {
                object: Box::new(base),
                index: Box::new(idx_expr),
                loc,
            }
        }
        Rule::method_suffix => {
            // method_suffix = { ":" ~ identifier ~ call_suffix }
            let mut ms_inner = inner.into_inner();
            let method = ms_inner.next().unwrap().as_str().to_string();
            let call_suffix_pair = ms_inner.next().unwrap();

            let mut args = Vec::new();
            for p in call_suffix_pair.into_inner() {
                if p.as_rule() == Rule::arguments {
                    for arg in p.into_inner() {
                        args.push(parse_expression(arg));
                    }
                }
            }

            Expression::MethodCall(MethodCall {
                object: Box::new(base),
                method,
                arguments: args,
                loc,
            })
        }
        _ => unreachable!("Unexpected rule in postfix"),
    }
}

fn parse_object_literal(pair: Pair<Rule>) -> Expression {
    // object_literal = { "${" ~ (pair ~ ("," ~ pair)*)? ~ "}" }
    let loc = loc_from_pair(&pair);
    let mut fields = Vec::new();

    for p in pair.into_inner() {
        if p.as_rule() == Rule::pair {
            let mut inner = p.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let val = parse_expression(inner.next().unwrap());
            fields.push((key, val));
        }
    }

    Expression::ObjectLiteral(fields, loc)
}

fn parse_if_expr(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner();
    let mut test = Expression::Literal(Literal::Value(Value::Null), loc);
    let mut body = Vec::new();
    let mut else_body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::expression => test = parse_expression(p),
            Rule::block => {
                if body.is_empty() {
                    body = parse_block(p);
                } else {
                    else_body = parse_block(p);
                }
            }
            Rule::if_expr => {
                // Handle "else if"
                let next_if = parse_if_expr(p);
                else_body = vec![Statement::Expression(next_if)];
            }
            Rule::IF | Rule::ELSE => {}
            _ => unreachable!("Unexpected rule in if_expr: {:?}", p.as_rule()),
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
    // try_catch = { TRY ~ block ~ CATCH ~ identifier? ~ block ~ (FINALLY ~ block)? }
    // Note: TRY, CATCH, FINALLY are atomic rules (@{...}) so they appear in into_inner()
    let loc = loc_from_pair(&pair);

    // Collect all items, filtering out keyword rules
    let items: Vec<_> = pair
        .into_inner()
        .filter(|p| !matches!(p.as_rule(), Rule::TRY | Rule::CATCH | Rule::FINALLY))
        .collect();

    let mut iter = items.into_iter();

    // First item should be try block
    let try_body = if let Some(try_block_pair) = iter.next() {
        parse_block(try_block_pair)
    } else {
        Vec::new()
    };

    // Parse optional error variable name and catch block
    let mut error_name = None;
    let mut catch_body = Vec::new();

    if let Some(next_pair) = iter.next() {
        match next_pair.as_rule() {
            Rule::identifier => {
                error_name = Some(next_pair.as_str().to_string());
                // Next must be catch block
                if let Some(catch_block_pair) = iter.next() {
                    catch_body = parse_block(catch_block_pair);
                }
            }
            Rule::block => {
                // No error variable, this is the catch block
                catch_body = parse_block(next_pair);
            }
            _ => {}
        }
    }

    // Parse optional finally block
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
    // throw_stmt = { THROW ~ NEWLINE* ~ expression }
    // Note: THROW is an atomic rule (@{...}) so it appears in into_inner()
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();

    // Skip THROW keyword if present
    let mut expr_pair = inner.next();
    while let Some(ref p) = expr_pair {
        if p.as_rule() == Rule::THROW {
            expr_pair = inner.next();
        } else {
            break;
        }
    }

    let value = if let Some(p) = expr_pair {
        match p.as_rule() {
            Rule::expression => parse_expression(p),
            _ => parse_logical_or(p),
        }
    } else {
        Expression::Literal(Literal::Value(Value::string("Unknown error".to_string())), loc)
    };

    Statement::Throw { value, loc }
}

fn parse_import_expr(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();

    // Skip IMPORT keyword if present
    let mut path_pair = inner.next();
    while let Some(ref p) = path_pair {
        if p.as_rule() == Rule::IMPORT {
            path_pair = inner.next();
        } else {
            break;
        }
    }

    let path_str = path_pair.map(|p| p.as_str()).unwrap_or("");
    // Strip quotes: "path" -> path
    let path = if path_str.len() >= 2 {
        path_str[1..path_str.len() - 1].to_string()
    } else {
        String::new()
    };

    Expression::Import { path, loc }
}
