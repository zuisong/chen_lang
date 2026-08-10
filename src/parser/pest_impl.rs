//! Pest-based parser (optional, enabled with pest-parser feature)
//!
//! This module is only compiled when the `pest-parser` feature is enabled.
//! It implements the Luau-style grammar defined in `src/chen.pest` and
//! converts the resulting parse tree into the shared AST (`expression.rs`),
//! mirroring the behavior of the handwritten parser.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use rust_decimal::Decimal;

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
                        statements.extend(parse_statement(inner_pair));
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

// --- Statement parsing ---

fn parse_block(pair: Pair<Rule>) -> Vec<Statement> {
    let mut stmts = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::statement {
            stmts.extend(parse_statement(p));
        }
    }
    stmts
}

fn parse_statement(pair: Pair<Rule>) -> Vec<Statement> {
    let loc = loc_from_pair(&pair);
    let first = pair.into_inner().next().unwrap();
    let stmt = match first.as_rule() {
        Rule::local_stmt => parse_local(first),
        Rule::function_stmt => parse_function_stmt(first),
        Rule::while_stmt => parse_while(first),
        Rule::repeat_stmt => parse_repeat(first),
        Rule::for_stmt => return parse_for(first),
        Rule::do_stmt => {
            let block = first.into_inner().next().unwrap();
            Statement::Expression(Expression::Block(parse_block(block), loc))
        }
        Rule::return_stmt => parse_return(first),
        Rule::break_stmt => Statement::Break(loc),
        Rule::continue_stmt => Statement::Continue(loc),
        Rule::try_stmt => parse_try(first),
        Rule::if_stmt => Statement::Expression(parse_if(first)),
        Rule::assign_multi => parse_assign_multi(first),
        Rule::assign_stmt => parse_assign(first),
        Rule::expr_stmt => {
            let e = first.into_inner().next().unwrap();
            Statement::Expression(parse_expr(e))
        }
        _ => unreachable!("Unexpected statement rule: {:?}", first.as_rule()),
    };
    vec![stmt]
}

/// `local name [= expr]` / `local a, b = e1, e2` / `local function f() ... end`
fn parse_local(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // LOCAL
    let first = inner.next().unwrap();

    // local function f() ... end
    if first.as_rule() == Rule::FUNCTION {
        let name_pair = inner.next().unwrap();
        let name = name_pair.as_str().to_string();
        let name_loc = loc_from_pair(&name_pair);
        let func_body = inner.next().unwrap();
        let (parameters, vararg, body, _) = parse_func_body(func_body);
        return Statement::FunctionDeclaration(FunctionDeclaration {
            name: Some(name),
            parameters,
            vararg,
            body,
            loc: name_loc,
        });
    }

    // namelist
    let mut names = Vec::new();
    for n in first.into_inner() {
        names.push(n.as_str().to_string());
    }

    let mut values = Vec::new();
    if let Some(list) = inner.next() {
        for e in list.into_inner() {
            values.push(parse_expr(e));
        }
    }

    // 与 handwritten 保持一致：多变量或多值 -> LocalList
    if names.len() > 1 || values.len() > 1 {
        return Statement::LocalList(LocalList { names, values, loc });
    }

    let name = names.into_iter().next().unwrap_or_default();
    let val = values
        .into_iter()
        .next()
        .unwrap_or(Expression::Literal(Literal::Value(Value::Null), loc));
    Statement::Local(Local {
        name,
        expression: val,
        loc,
    })
}

/// `function name(params) body end`
fn parse_function_stmt(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // FUNCTION
    let name_pair = inner.next().unwrap();
    let name = name_pair.as_str().to_string();
    let name_loc = loc_from_pair(&name_pair);
    let func_body = inner.next().unwrap();
    let (parameters, vararg, body, _) = parse_func_body(func_body);
    Statement::FunctionDeclaration(FunctionDeclaration {
        name: Some(name),
        parameters,
        vararg,
        body,
        loc: name_loc,
    })
}

/// 解析函数体 `(params) block end`，返回 (parameters, vararg, body, loc)
fn parse_func_body(pair: Pair<Rule>) -> (Vec<String>, bool, Vec<Statement>, Location) {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let mut parameters = Vec::new();
    let mut vararg = false;
    let mut body = Vec::new();

    if let Some(first) = inner.next() {
        if first.as_rule() == Rule::params {
            for p in first.into_inner() {
                if p.as_rule() == Rule::vararg {
                    vararg = true;
                } else {
                    parameters.push(p.as_str().to_string());
                }
            }
            if let Some(b) = inner.next() {
                body = parse_block(b);
            }
        } else {
            body = parse_block(first);
        }
    }
    (parameters, vararg, body, loc)
}

/// `while expr do block end`
fn parse_while(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // WHILE
    let test = parse_expr(inner.next().unwrap());
    inner.next().unwrap(); // DO
    let body = parse_block(inner.next().unwrap());
    Statement::Loop(Loop { test, body, loc })
}

/// `repeat block until expr`
fn parse_repeat(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // REPEAT
    let body = parse_block(inner.next().unwrap());
    inner.next().unwrap(); // UNTIL
    let test = parse_expr(inner.next().unwrap());
    Statement::Repeat(Repeat { body, test, loc })
}

/// `for i = a, b [, step] do ... end`（desugar 成 while）或 `for k, v in expr do ... end`
fn parse_for(pair: Pair<Rule>) -> Vec<Statement> {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // FOR
    let first = inner.next().unwrap();

    if first.as_rule() == Rule::for_numeric {
        return parse_for_numeric(first, loc);
    }

    // for_in
    let mut f = first.into_inner();
    let mut vars = Vec::new();
    for n in f.next().unwrap().into_inner() {
        vars.push(n.as_str().to_string());
    }
    f.next().unwrap(); // IN
    let iterable = parse_expr(f.next().unwrap());
    f.next().unwrap(); // DO
    let body = parse_block(f.next().unwrap());
    vec![Statement::ForIn(ForInLoop {
        vars,
        iterable,
        body,
        loc,
    })]
}

/// 复现 handwritten `parse_for_numeric` 的 desugar，生成 3 个顶层语句：
///   local var = start
///   local @step = step
///   while (step >= 0 and var <= end) or (step < 0 and var >= end) do body; var = var + @step end
fn parse_for_numeric(pair: Pair<Rule>, start_loc: Location) -> Vec<Statement> {
    let mut f = pair.into_inner();
    let var_pair = f.next().unwrap();
    let var_name = var_pair.as_str().to_string();
    let var_loc = loc_from_pair(&var_pair);
    let start = parse_expr(f.next().unwrap());
    let end = parse_expr(f.next().unwrap());

    let mut step = None;
    let mut do_pair = None;
    while let Some(p) = f.next() {
        match p.as_rule() {
            Rule::expression => step = Some(parse_expr(p)),
            Rule::DO => {
                do_pair = Some(p);
                break;
            }
            _ => unreachable!("Unexpected rule in for_numeric: {:?}", p.as_rule()),
        }
    }
    debug_assert!(do_pair.is_some());
    let step = step.unwrap_or(Expression::Literal(Literal::Value(Value::Int(1)), start_loc));
    let body = parse_block(f.next().unwrap());

    // Desugar（与 handwritten 完全一致）
    let mut loop_body = body;
    loop_body.push(Statement::Assign(Assign {
        name: var_name.clone(),
        expr: Box::new(Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Identifier(var_name.clone(), var_loc)),
            operator: Operator::Add,
            right: Box::new(Expression::Identifier("@step".to_string(), start_loc)),
            loc: start_loc,
        })),
        loc: start_loc,
    }));

    let var_expr = Expression::Identifier(var_name.clone(), var_loc);
    let step_expr = Expression::Identifier("@step".to_string(), start_loc);
    let end_expr = end;

    let step_ge_zero = Expression::BinaryOperation(BinaryOperation {
        left: Box::new(step_expr.clone()),
        operator: Operator::GtE,
        right: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
        loc: start_loc,
    });
    let asc_cond = Expression::BinaryOperation(BinaryOperation {
        left: Box::new(step_ge_zero),
        operator: Operator::And,
        right: Box::new(Expression::BinaryOperation(BinaryOperation {
            left: Box::new(var_expr.clone()),
            operator: Operator::LtE,
            right: Box::new(end_expr.clone()),
            loc: start_loc,
        })),
        loc: start_loc,
    });
    let step_lt_zero = Expression::BinaryOperation(BinaryOperation {
        left: Box::new(step_expr.clone()),
        operator: Operator::Lt,
        right: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), start_loc)),
        loc: start_loc,
    });
    let desc_cond = Expression::BinaryOperation(BinaryOperation {
        left: Box::new(step_lt_zero),
        operator: Operator::And,
        right: Box::new(Expression::BinaryOperation(BinaryOperation {
            left: Box::new(var_expr.clone()),
            operator: Operator::GtE,
            right: Box::new(end_expr.clone()),
            loc: start_loc,
        })),
        loc: start_loc,
    });
    let test = Expression::BinaryOperation(BinaryOperation {
        left: Box::new(asc_cond),
        operator: Operator::Or,
        right: Box::new(desc_cond),
        loc: start_loc,
    });

    vec![
        Statement::Local(Local {
            name: var_name.clone(),
            expression: start,
            loc: var_loc,
        }),
        Statement::Local(Local {
            name: "@step".to_string(),
            expression: step,
            loc: start_loc,
        }),
        Statement::Loop(Loop {
            test,
            body: loop_body,
            loc: start_loc,
        }),
    ]
}

/// `return [expr [, expr]*]`
fn parse_return(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // RETURN
    let mut values = Vec::new();
    if let Some(list) = inner.next() {
        for e in list.into_inner() {
            values.push(parse_expr(e));
        }
    }
    Statement::Return(Return { values, loc })
}

/// `try block catch [name] block [finally block] end`
fn parse_try(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // TRY
    let try_body = parse_block(inner.next().unwrap());
    inner.next().unwrap(); // CATCH

    let mut error_name = None;
    let next = inner.next().unwrap();
    let catch_body = if next.as_rule() == Rule::catch_var {
        error_name = Some(next.as_str().trim().to_string());
        parse_block(inner.next().unwrap())
    } else {
        parse_block(next)
    };

    let mut finally_body = None;
    if let Some(p) = inner.next() {
        if p.as_rule() == Rule::FINALLY {
            finally_body = Some(parse_block(inner.next().unwrap()));
        }
    }

    Statement::TryCatch(TryCatch {
        try_body,
        error_name,
        catch_body,
        finally_body,
        loc,
    })
}

/// `a, b = e1, e2`
fn parse_assign_multi(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let mut names = Vec::new();
    while let Some(p) = inner.next() {
        if p.as_rule() == Rule::identifier {
            names.push(p.as_str().to_string());
        } else if p.as_rule() == Rule::expression_list {
            let mut exprs = Vec::new();
            for e in p.into_inner() {
                exprs.push(parse_expr(e));
            }
            return Statement::AssignMulti(AssignMulti { names, exprs, loc });
        }
    }
    unreachable!("Malformed assign_multi")
}

/// `target = expr`（含复合赋值）
fn parse_assign(pair: Pair<Rule>) -> Statement {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let target_pair = inner.next().unwrap();
    let op_pair = inner.next().unwrap();
    let value = parse_expr(inner.next().unwrap());

    let op_text = op_pair.as_str();
    let lvalue = parse_assignable(target_pair);

    let final_value = match op_text {
        "+=" => compound(lvalue.clone(), Operator::Add, value, loc),
        "-=" => compound(lvalue.clone(), Operator::Subtract, value, loc),
        "*=" => compound(lvalue.clone(), Operator::Multiply, value, loc),
        "/=" => compound(lvalue.clone(), Operator::Divide, value, loc),
        "//=" => compound(lvalue.clone(), Operator::FloorDiv, value, loc),
        "%=" => compound(lvalue.clone(), Operator::Mod, value, loc),
        "..=" => compound(lvalue.clone(), Operator::Concat, value, loc),
        _ => value,
    };

    match lvalue {
        Expression::Identifier(name, id_loc) => Statement::Assign(Assign {
            name,
            expr: Box::new(final_value),
            loc: id_loc,
        }),
        Expression::GetField { object, field, loc } => Statement::SetField {
            object: *object,
            field,
            value: final_value,
            loc,
        },
        Expression::Index { object, index, loc } => Statement::SetIndex {
            object: *object,
            index: *index,
            value: final_value,
            loc,
        },
        _ => unreachable!("Invalid assignment target"),
    }
}

fn compound(lvalue: Expression, op: Operator, value: Expression, loc: Location) -> Expression {
    Expression::BinaryOperation(BinaryOperation {
        left: Box::new(lvalue),
        operator: op,
        right: Box::new(value),
        loc,
    })
}

fn parse_assignable(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let mut expr = parse_primary(inner.next().unwrap());
    for suffix in inner {
        expr = match suffix.as_rule() {
            Rule::dot_suffix => {
                let sloc = loc_from_pair(&suffix);
                let field = suffix.into_inner().next().unwrap().as_str().to_string();
                Expression::GetField {
                    object: Box::new(expr),
                    field,
                    loc: sloc,
                }
            }
            Rule::index_suffix => {
                let sloc = loc_from_pair(&suffix);
                let index = parse_expr(suffix.into_inner().next().unwrap());
                Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    loc: sloc,
                }
            }
            _ => unreachable!("Unexpected assignable suffix"),
        };
    }
    expr
}

// --- Expression parsing ---

fn parse_expr(pair: Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::expression => parse_expr(pair.into_inner().next().unwrap()),
        Rule::or_expr
        | Rule::and_expr
        | Rule::equality
        | Rule::comparison
        | Rule::concat
        | Rule::term
        | Rule::factor => parse_binary(pair, parse_expr),
        Rule::unary => parse_unary(pair),
        Rule::power => parse_power(pair),
        Rule::postfix => parse_postfix(pair),
        Rule::primary => parse_primary(pair),
        _ => unreachable!("Unexpected expression rule: {:?}", pair.as_rule()),
    }
}

fn parse_binary(pair: Pair<Rule>, parse_sub: fn(Pair<Rule>) -> Expression) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let mut left = parse_sub(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "or" => Operator::Or,
            "and" => Operator::And,
            "==" => Operator::Equals,
            "~=" => Operator::NotEquals,
            "<" => Operator::Lt,
            "<=" => Operator::LtE,
            ">" => Operator::Gt,
            ">=" => Operator::GtE,
            ".." => Operator::Concat,
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "*" => Operator::Multiply,
            "/" => Operator::Divide,
            "//" => Operator::FloorDiv,
            "%" => Operator::Mod,
            _ => unreachable!("Unknown operator: {}", op_pair.as_str()),
        };
        let right = parse_sub(inner.next().unwrap());
        left = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator: op,
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
        Rule::NOT => {
            let right = parse_expr(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Not,
                expr: Box::new(right),
                loc,
            })
        }
        Rule::sub => {
            // -x  desugar 为 0 - x（与 handwritten 一致）
            let right = parse_expr(inner.next().unwrap());
            Expression::BinaryOperation(BinaryOperation {
                left: Box::new(Expression::Literal(Literal::Value(Value::Int(0)), loc)),
                operator: Operator::Subtract,
                right: Box::new(right),
                loc,
            })
        }
        Rule::len => {
            let right = parse_expr(inner.next().unwrap());
            Expression::Unary(Unary {
                operator: Operator::Len,
                expr: Box::new(right),
                loc,
            })
        }
        _ => parse_expr(first),
    }
}

fn parse_power(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let left = parse_expr(inner.next().unwrap());
    if let Some(_pow) = inner.next() {
        let right = parse_expr(inner.next().unwrap());
        Expression::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator: Operator::Pow,
            right: Box::new(right),
            loc,
        })
    } else {
        left
    }
}

fn parse_postfix(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    let mut expr = parse_expr(inner.next().unwrap());

    for suffix in inner {
        expr = match suffix.as_rule() {
            Rule::call_suffix => {
                let mut s_inner = suffix.into_inner();
                let args = match s_inner.next() {
                    Some(arg_list) => parse_expr_list(arg_list),
                    None => Vec::new(),
                };
                Expression::FunctionCall(FunctionCall {
                    callee: Box::new(expr),
                    arguments: args,
                    loc,
                })
            }
            Rule::dot_suffix => {
                let sloc = loc_from_pair(&suffix);
                let field = suffix.into_inner().next().unwrap().as_str().to_string();
                Expression::GetField {
                    object: Box::new(expr),
                    field,
                    loc: sloc,
                }
            }
            Rule::index_suffix => {
                let sloc = loc_from_pair(&suffix);
                let index = parse_expr(suffix.into_inner().next().unwrap());
                Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    loc: sloc,
                }
            }
            Rule::method_suffix => {
                let sloc = loc_from_pair(&suffix);
                let mut s_inner = suffix.into_inner();
                let method = s_inner.next().unwrap().as_str().to_string();
                let args = match s_inner.next() {
                    Some(arg_list) => parse_expr_list(arg_list),
                    None => Vec::new(),
                };
                Expression::MethodCall(MethodCall {
                    object: Box::new(expr),
                    method,
                    arguments: args,
                    loc: sloc,
                })
            }
            _ => unreachable!("Unexpected postfix suffix: {:?}", suffix.as_rule()),
        };
    }
    expr
}

fn parse_expr_list(pair: Pair<Rule>) -> Vec<Expression> {
    pair.into_inner().map(parse_expr).collect()
}

fn parse_primary(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let first = pair.into_inner().next().unwrap();
    match first.as_rule() {
        Rule::number => parse_number(first),
        Rule::string => {
            let raw = first.as_str();
            let s = raw[1..raw.len() - 1].to_string();
            Expression::Literal(Literal::Value(Value::string(s)), loc)
        }
        Rule::bool => Expression::Literal(Literal::Value(Value::Bool(first.as_str() == "true")), loc),
        Rule::NIL => Expression::Literal(Literal::Value(Value::Null), loc),
        Rule::identifier => Expression::Identifier(first.as_str().to_string(), loc),
        Rule::vararg => Expression::Vararg(loc),
        Rule::function_expr => parse_function_expr(first),
        Rule::if_stmt => parse_if(first),
        Rule::expression => parse_expr(first),
        Rule::table_literal => parse_table(first),
        Rule::array_literal => parse_array(first),
        _ => unreachable!("Unexpected primary rule: {:?}", first.as_rule()),
    }
}

fn parse_number(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer => {
            let v: i32 = inner.as_str().parse().expect("integer overflow");
            Expression::Literal(Literal::Value(Value::Int(v)), loc)
        }
        Rule::float => {
            let v: Decimal = inner.as_str().parse().expect("invalid float");
            Expression::Literal(Literal::Value(Value::Float(v)), loc)
        }
        _ => unreachable!("Unexpected number rule"),
    }
}

fn parse_function_expr(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // FUNCTION
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::identifier {
        let name = first.as_str().to_string();
        let name_loc = loc_from_pair(&first);
        let func_body = inner.next().unwrap();
        let (parameters, vararg, body, _) = parse_func_body(func_body);
        Expression::Function(FunctionDeclaration {
            name: Some(name),
            parameters,
            vararg,
            body,
            loc: name_loc,
        })
    } else {
        let (parameters, vararg, body, loc) = parse_func_body(first);
        Expression::Function(FunctionDeclaration {
            name: None,
            parameters,
            vararg,
            body,
            loc,
        })
    }
}

fn parse_if(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut inner = pair.into_inner();
    inner.next().unwrap(); // IF
    let test = parse_expr(inner.next().unwrap());
    inner.next().unwrap(); // THEN
    let body = parse_block(inner.next().unwrap());

    let else_body = parse_if_rest(&mut inner, loc);
    Expression::If(If {
        test: Box::new(test),
        body,
        else_body,
        loc,
    })
}

/// 解析 if 的 elseif/else/end 剩余部分，返回 else 分支语句列表
fn parse_if_rest(inner: &mut pest::iterators::Pairs<Rule>, loc: Location) -> Vec<Statement> {
    let mut else_body = Vec::new();
    loop {
        match inner.next() {
            Some(p) => match p.as_rule() {
                Rule::ELSEIF => {
                    let elseif_test = parse_expr(inner.next().unwrap());
                    inner.next().unwrap(); // THEN
                    let elseif_body = parse_block(inner.next().unwrap());
                    let nested_else = parse_if_rest(inner, loc);
                    let nested = Expression::If(If {
                        test: Box::new(elseif_test),
                        body: elseif_body,
                        else_body: nested_else,
                        loc,
                    });
                    else_body.push(Statement::Expression(nested));
                    break;
                }
                Rule::ELSE => {
                    else_body = parse_block(inner.next().unwrap());
                    break;
                }
                Rule::END => break,
                _ => unreachable!("Unexpected rule in if tail: {:?}", p.as_rule()),
            },
            None => break,
        }
    }
    else_body
}

fn parse_table(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut fields: Vec<(String, Expression)> = Vec::new();
    let mut array_elems: Vec<Expression> = Vec::new();
    let mut has_fields = false;

    for f in pair.into_inner() {
        if f.as_rule() != Rule::field {
            continue;
        }
        let mut f_inner = f.into_inner();
        let first = f_inner.next().unwrap();
        if first.as_rule() == Rule::identifier {
            let key = first.as_str().to_string();
            let val = parse_expr(f_inner.next().unwrap());
            fields.push((key, val));
            has_fields = true;
        } else {
            array_elems.push(parse_expr(first));
        }
    }

    // 与 handwritten 保持一致：有字段或无数组成员 -> ObjectLiteral
    if has_fields || array_elems.is_empty() {
        let mut result = fields;
        for (i, elem) in array_elems.into_iter().enumerate() {
            result.push(((i + 1).to_string(), elem));
        }
        Expression::ObjectLiteral(result, loc)
    } else {
        Expression::ArrayLiteral(array_elems, loc)
    }
}

fn parse_array(pair: Pair<Rule>) -> Expression {
    let loc = loc_from_pair(&pair);
    let mut elems = Vec::new();
    if let Some(list) = pair.into_inner().next() {
        elems = parse_expr_list(list);
    }
    Expression::ArrayLiteral(elems, loc)
}
