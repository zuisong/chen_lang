//! Pest-based parser (optional, enabled with pest-parser feature)
//!
//! This module is only compiled when the `pest-parser` feature is enabled.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::expression::*;
use crate::token::Operator;
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

fn parse_statement(pair: Pair<Rule>) -> Statement {
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::declaration => parse_declaration(inner, line),
        Rule::assignment => parse_assignment(inner, line),
        Rule::for_loop => parse_for_loop(inner, line),
        Rule::function_def => parse_function_def(inner, line),
        Rule::return_stmt => parse_return_stmt(inner, line),
        Rule::break_stmt => Statement::Break(line),
        Rule::continue_stmt => Statement::Continue(line),
        Rule::expression => Statement::Expression(parse_expression(inner)),
        _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
    }
}

fn parse_declaration(pair: Pair<Rule>, line: u32) -> Statement {
    let inner = pair.into_inner();

    let mut name = String::new();
    let mut expr = Expression::Literal(Literal::Value(Value::Null), line);

    for p in inner {
        match p.as_rule() {
            Rule::identifier => name = p.as_str().to_string(),
            Rule::expression => expr = parse_expression(p),
            Rule::LET | Rule::assign => {} // skip keywords/ops
            _ => unreachable!("Unexpected rule in declaration: {:?}", p.as_rule()),
        }
    }

    Statement::Local(Local {
        name,
        expression: expr,
        line,
    })
}

fn parse_assignment(pair: Pair<Rule>, line: u32) -> Statement {
    // assignment = { assignment_target ~ assign ~ expression }
    // assignment_target = { identifier ~ postfix* }
    let mut inner = pair.into_inner();
    let target_pair = inner.next().unwrap();
    let mut expr = Expression::Literal(Literal::Value(Value::Null), line);

    let lvalue = parse_assignment_target(target_pair);

    for p in inner {
        if p.as_rule() == Rule::expression {
            expr = parse_expression(p);
        }
    }

    match lvalue {
        Expression::Identifier(name, _) => Statement::Assign(Assign {
            name,
            expr: Box::new(expr),
            line,
        }),
        Expression::GetField { object, field, .. } => Statement::SetField {
            object: *object,
            field,
            value: expr,
            line,
        },
        Expression::Index { object, index, .. } => Statement::SetIndex {
            object: *object,
            index: *index,
            value: expr,
            line,
        },
        _ => unreachable!("Invalid l-value in assignment: {:?}", lvalue),
    }
}

fn parse_assignment_target(pair: Pair<Rule>) -> Expression {
    // assignment_target = { identifier ~ postfix* }
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let mut inner = pair.into_inner();
    let identifier_pair = inner.next().unwrap();
    let mut expr = Expression::Identifier(identifier_pair.as_str().to_string(), line);

    for p in inner {
        match p.as_rule() {
            Rule::postfix => expr = parse_postfix(expr, p),
            _ => unreachable!("Unexpected rule in assignment_target"),
        }
    }
    expr
}

fn parse_for_loop(pair: Pair<Rule>, line: u32) -> Statement {
    let inner = pair.into_inner();
    let mut test = Expression::Literal(Literal::Value(Value::Bool(false)), line);
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::expression => test = parse_expression(p),
            Rule::block => body = parse_block(p),
            Rule::FOR => {}
            _ => unreachable!("Unexpected rule in for_loop: {:?}", p.as_rule()),
        }
    }

    Statement::Loop(Loop { test, body, line })
}

fn build_function_declaration(pair: Pair<Rule>) -> FunctionDeclaration {
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let inner = pair.into_inner();
    let mut name = None;
    let mut parameters = Vec::new();
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::identifier => name = Some(p.as_str().to_string()),
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
        line,
    }
}

fn parse_function_def(pair: Pair<Rule>, _line: u32) -> Statement {
    // The line from statement is passed but FunctionDeclaration also has its own line derived inside.
    // We can use the passed line or derive again. Derive is cleaner for helper.
    let decl = build_function_declaration(pair);
    Statement::FunctionDeclaration(decl)
}

fn parse_return_stmt(pair: Pair<Rule>, line: u32) -> Statement {
    let mut expr = Expression::Literal(Literal::Value(Value::Null), line);
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            expr = parse_expression(p);
        }
    }
    Statement::Return(Return { expression: expr, line })
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
    let inner = pair.into_inner().next().unwrap();
    parse_logical_or(inner)
}

fn parse_binary_op<F>(pair: Pair<Rule>, parse_sub: F) -> Expression
where
    F: Fn(Pair<Rule>) -> Expression,
{
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let mut inner = pair.into_inner();
    let mut left = parse_sub(inner.next().unwrap());

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
            line,
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
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::not => {
            let expr = parse_unary(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Not,
                expr: Box::new(expr),
                line,
            })
        }
        Rule::subtract => {
            let expr = parse_unary(inner.next().unwrap());
            // -x is 0 - x
            Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), line)),
                operator: Operator::Subtract,
                right: Box::new(expr),
                line,
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
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let mut elements = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            elements.push(parse_expression(p));
        }
    }
    Expression::ArrayLiteral(elements, line)
}

fn parse_atom(pair: Pair<Rule>) -> Expression {
    // atom = { float | integer | bool | string | identifier | "(" ~ expression ~ ")" | if_expr | block | object_literal }
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::float => Expression::Literal(Literal::Value(Value::Float(inner.as_str().parse().unwrap())), line),
        Rule::integer => Expression::Literal(Literal::Value(Value::Int(inner.as_str().parse().unwrap())), line),
        Rule::bool => Expression::Literal(Literal::Value(Value::Bool(inner.as_str() == "true")), line),
        Rule::string => {
            let s = inner.as_str();
            // TODO: Better string unescaping
            let content = &s[1..s.len() - 1];
            Expression::Literal(Literal::Value(Value::string(content.to_string())), line)
        }
        Rule::identifier => Expression::Identifier(inner.as_str().to_string(), line),
        Rule::expression => parse_expression(inner), // ( expr )
        Rule::if_expr => parse_if_expr(inner),
        Rule::block => Expression::Block(parse_block(inner), line),
        Rule::object_literal => parse_object_literal(inner),
        Rule::function_def => Expression::Function(build_function_declaration(inner)),
        Rule::array_literal => parse_array_literal(inner),
        _ => unreachable!("Unexpected rule in atom: {:?}", inner.as_rule()),
    }
}

fn parse_postfix(base: Expression, pair: Pair<Rule>) -> Expression {
    // postfix = { call_suffix | dot_suffix | index_suffix }
    let line = pair.as_span().start_pos().line_col().0 as u32;
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
                line,
            })
        }
        Rule::dot_suffix => {
            // dot_suffix = { "." ~ identifier }
            let field = inner.into_inner().next().unwrap().as_str().to_string();
            Expression::GetField {
                object: Box::new(base),
                field,
                line,
            }
        }
        Rule::index_suffix => {
            // index_suffix = { "[" ~ expression ~ "]" }
            let idx_expr = parse_expression(inner.into_inner().next().unwrap());
            Expression::Index {
                object: Box::new(base),
                index: Box::new(idx_expr),
                line,
            }
        }
        _ => unreachable!("Unexpected rule in postfix"),
    }
}

fn parse_object_literal(pair: Pair<Rule>) -> Expression {
    // object_literal = { "#{" ~ (pair ~ ("," ~ pair)*)? ~ "}" }
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let mut fields = Vec::new();

    for p in pair.into_inner() {
        if p.as_rule() == Rule::pair {
            let mut inner = p.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let val = parse_expression(inner.next().unwrap());
            fields.push((key, val));
        }
    }

    Expression::ObjectLiteral(fields, line)
}

fn parse_if_expr(pair: Pair<Rule>) -> Expression {
    let line = pair.as_span().start_pos().line_col().0 as u32;
    let inner = pair.into_inner();
    let mut test = Expression::Literal(Literal::Value(Value::Null), line);
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
            Rule::IF | Rule::ELSE => {}
            _ => unreachable!("Unexpected rule in if_expr: {:?}", p.as_rule()),
        }
    }

    Expression::If(If {
        test: Box::new(test),
        body,
        else_body,
        line,
    })
}
