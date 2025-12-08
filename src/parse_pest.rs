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
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::declaration => parse_declaration(inner),
        Rule::assignment => parse_assignment(inner),
        Rule::for_loop => parse_for_loop(inner),
        Rule::function_def => parse_function_def(inner),
        Rule::return_stmt => parse_return_stmt(inner),
        Rule::break_stmt => Statement::Break,
        Rule::continue_stmt => Statement::Continue,
        Rule::expression => Statement::Expression(parse_expression(inner)),
        _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
    }
}

fn parse_declaration(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner();

    let mut name = String::new();
    let mut expr = Expression::Literal(Literal::Value(Value::Null));

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
    })
}

fn parse_assignment(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner();
    let mut name = String::new();
    let mut expr = Expression::Literal(Literal::Value(Value::Null));

    for p in inner {
        match p.as_rule() {
            Rule::identifier => name = p.as_str().to_string(),
            Rule::expression => expr = parse_expression(p),
            Rule::assign => {}
            _ => unreachable!("Unexpected rule in assignment: {:?}", p.as_rule()),
        }
    }

    Statement::Assign(Assign {
        name,
        expr: Box::new(expr),
    })
}

fn parse_for_loop(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner();
    let mut test = Expression::Literal(Literal::Value(Value::Bool(false)));
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::expression => test = parse_expression(p),
            Rule::block => body = parse_block(p),
            Rule::FOR => {}
            _ => unreachable!("Unexpected rule in for_loop: {:?}", p.as_rule()),
        }
    }

    Statement::Loop(Loop { test, body })
}

fn parse_function_def(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner();
    let mut name = String::new();
    let mut parameters = Vec::new();
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::identifier => name = p.as_str().to_string(),
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

    Statement::FunctionDeclaration(FunctionDeclaration {
        name,
        parameters,
        body,
    })
}

fn parse_return_stmt(pair: Pair<Rule>) -> Statement {
    let mut expr = Expression::Literal(Literal::Value(Value::Null));
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            expr = parse_expression(p);
        }
    }
    Statement::Return(Return { expression: expr })
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

// Generic function to parse binary operations
// This handles rules like: rule = { sub_rule ~ (op ~ sub_rule)* }
fn parse_binary_op<F>(pair: Pair<Rule>, parse_sub: F) -> Expression
where
    F: Fn(Pair<Rule>) -> Expression,
{
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
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::not => {
            let expr = parse_unary(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Not,
                expr: Box::new(expr),
            })
        }
        Rule::subtract => {
            let expr = parse_unary(inner.next().unwrap());
            // -x is 0 - x
            Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)))),
                operator: Operator::Subtract,
                right: Box::new(expr),
            })
        }
        Rule::primary => parse_primary(first),
        _ => unreachable!("Unexpected rule in unary: {:?}", first.as_rule()),
    }
}

fn parse_primary(pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer => {
            Expression::Literal(Literal::Value(Value::Int(inner.as_str().parse().unwrap())))
        }
        Rule::float => Expression::Literal(Literal::Value(Value::Float(
            inner.as_str().parse().unwrap(),
        ))),
        Rule::string => {
            let s = inner.as_str();
            // remove quotes
            let content = &s[1..s.len() - 1];
            Expression::Literal(Literal::Value(Value::string(content.to_string())))
        }
        Rule::bool => {
            let val = inner.as_str() == "true";
            Expression::Literal(Literal::Value(Value::Bool(val)))
        }
        Rule::identifier => Expression::Identifier(inner.as_str().to_string()),
        Rule::function_call => {
            let mut inner_pairs = inner.into_inner();
            let name = inner_pairs.next().unwrap().as_str().to_string();
            let mut args = Vec::new();

            for p in inner_pairs {
                if p.as_rule() == Rule::arguments {
                    for arg in p.into_inner() {
                        args.push(parse_expression(arg));
                    }
                }
            }
            Expression::FunctionCall(FunctionCall {
                name,
                arguments: args,
            })
        }
        Rule::expression => parse_expression(inner), // ( expression )
        Rule::if_expr => parse_if_expr(inner),
        Rule::block => Expression::Block(parse_block(inner)),
        _ => unreachable!("Unexpected rule in primary: {:?}", inner.as_rule()),
    }
}

fn parse_if_expr(pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner();
    let mut test = Expression::Literal(Literal::Value(Value::Null));
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
    })
}
