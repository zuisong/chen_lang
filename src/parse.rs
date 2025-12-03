#![deny(missing_docs)]

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::vec;

use crate::parse::OperatorPriority::*;
use crate::*;
use crate::value::Value;

#[derive(Debug, Eq, PartialEq, Clone)]
enum OperatorPriority {
    Middle,
    Small,
    Normal,
    Minimal,
}

impl OperatorPriority {
    fn priority_value(&self) -> i32 {
        match self {
            Middle => 2,
            Small => 1,
            Normal => 0,
            Minimal => -1,
        }
    }
}

impl PartialOrd for OperatorPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority_value().partial_cmp(&other.priority_value())
    }
}

fn get_priority(opt: &Operator) -> OperatorPriority {
    match opt {
        Operator::Add => Small,
        Operator::Subtract => Small,
        Operator::Multiply => Middle,
        Operator::Divide => Middle,
        Operator::Mod => Middle,
        Operator::Assign => Small,
        Operator::And => Minimal,
        Operator::Equals => Middle,
        Operator::NotEquals => Middle,
        Operator::Or => Minimal,
        Operator::Not => Normal,
        Operator::Gt => Middle,
        Operator::Lt => Middle,
        Operator::GtE => Middle,
        Operator::LtE => Middle,
    }
}

/// 简单表达式分析 (只有运算的 一行)
pub fn parse_expression(line: &[Token]) -> anyhow::Result<Expression> {
    // 中缀表达式变后缀表达式
    let mut result: Vec<&Token> = Vec::new();
    let mut stack: Vec<&Token> = vec![];
    for token in line {
        match *token {
            Token::LParen => stack.push(token),
            Token::RParen => {
                while let Some(top) = stack.pop() {
                    if *top == Token::LParen {
                        break;
                    }
                    result.push(top);
                }
            }
            Token::Operator(opt) => {
                while let Some(Token::Operator(opt2)) = stack.last() {
                    if get_priority(opt2) >= get_priority(&opt) {
                        result.push(stack.pop().unwrap());
                        continue;
                    }
                    break;
                }
                stack.push(token);
            }
            _ => result.push(token),
        }
    }
    while let Some(t) = stack.pop() {
        result.push(t);
    }

    let mut result: VecDeque<_> = result
        .into_iter()
        .filter(|&it| it != &Token::LParen && it != &Token::RParen)
        .cloned()
        .collect();

    let mut tmp: Vec<Expression> = Vec::new();

    while let Some(t) = result.pop_front() {
        if let Token::Operator(opt) = t {
            let new_exp: Expression = match opt {
                Operator::Assign => {
                    unreachable!();
                }

                Operator::Not => Expression::NotStatement(NotStatement {
                    expr: Box::new(tmp.pop().unwrap()),
                }),

                _ => {
                    let o1 = tmp.pop().unwrap();
                    let o2 = tmp.pop().unwrap();
                    Expression::BinaryOperation(BinaryOperation {
                        left: Box::new(o2),
                        right: Box::new(o1),
                        operator: opt,
                    })
                }
            };
            tmp.push(new_exp);
        } else {
            let ele: Literal = match t {
                Token::Identifier(name) => Literal::Identifier(name),
                Token::Int(i) => Literal::Value(Value::Int(i)),
                Token::Float(f) => Literal::Value(Value::Float(f)),
                Token::Bool(i) => Literal::Value(Value::Bool(i)),
                Token::String(i) => Literal::Value(Value::string(i)),
                _ => panic!("错误,{t:?}"),
            };
            tmp.push(Expression::Literal(ele));
        }
    }

    tmp.pop().ok_or(err_msg("parse error"))
}

/// 分析很多行的方法
pub fn parse_block(lines: &[Box<[Token]>], mut start_line: usize) -> Result<(usize, Ast)> {
    let mut v = Vec::new();
    while start_line < lines.len() && lines[start_line][0] != Token::RBig {
        match &lines[start_line][0] {
            Token::LBig => {
                let (endline, stmts) = parse_block(lines, start_line + 1)?;
                v.push(Statement::Expression(Expression::Block(stmts)));
                start_line = endline + 1;
            }
            Token::Keyword(Keyword::LET) => {
                let (endline, var) = parse_declare(lines, start_line)?;
                v.push(Statement::Local(var));
                start_line = endline;
            }
            Token::Keyword(Keyword::FOR) => {
                let var = parse_for(lines, start_line)?;
                v.push(Statement::Loop(var.1));
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::DEF) => {
                let var = parse_define_function(lines, start_line)?;
                v.push(Statement::FunctionDeclaration(var.1));
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::IF) => {
                let var = parse_if(lines, start_line)?;
                v.push(Statement::If(var.1));
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::RETURN) => {
                let var = parse_return(&lines[start_line])?;
                v.push(Statement::Return(var));
                start_line += 1;
            }
            // 赋值
            Token::Identifier(_)
                if lines[start_line].get(1) == Some(&Token::Operator(Operator::Assign)) =>
            {
                let (endline, var) = parse_assign(lines, start_line)?;
                v.push(Statement::Assign(var));
                start_line = endline;
            }
            // 函数调用
            Token::Identifier(_) if lines[start_line].get(1) == Some(&Token::LParen) => {
                let var = parse_func_call(&lines[start_line])?;
                v.push(Statement::Expression(Expression::FunctionCall(var)));
                start_line += 1;
            }
            // 返回值
            Token::Identifier(_) => {
                let var = parse_expression(&lines[start_line])?;
                v.push(Statement::Expression(var));
                start_line += 1;
            }
            // 返回值
            Token::Int(_) | Token::Float(_) | Token::Bool(_) | Token::String(_) | Token::LParen | Token::Operator(Operator::Not) | Token::Operator(Operator::Subtract) => {
                let var = parse_expression(&lines[start_line])?;
                v.push(Statement::Expression(var));
                start_line += 1;
            }
            _ => {
                unimplemented!("语法错误 {:?}", lines[start_line]);
            }
        }
    }
    Ok((start_line, v.into_iter().collect()))
}

fn parse_func_call(line: &[Token]) -> Result<FunctionCall> {
    let func_name = if let Token::Identifier(name) = &line[0] {
        name.to_string()
    } else {
        return Err(err_msg("不是函数定义语句"));
    };

    assert_eq!(&line[1], &Token::LParen);
    let param_idx: Vec<_> = line
        .iter()
        .enumerate()
        .skip(2)
        .filter(|it| it.1 == &Token::COMMA)
        .map(|it| it.0)
        .collect();

    let mut params = vec![];

    match param_idx.len() {
        0 => {
            // 检查是否是无参数函数调用
            if line.len() == 3 && line[1] == Token::LParen && line[2] == Token::RParen {
                // 无参数函数调用，不需要解析参数
                params = vec![];
            } else {
                params.push(parse_expression(&line[2..(line.len() - 1)])?);
            }
        }
        _ => {
            params.push(parse_expression(&line[2..param_idx[0]])?);
            for i in 0..(param_idx.len() - 1) {
                params.push(parse_expression(&line[param_idx[i]..param_idx[i + 1]])?);
            }
            params.push(parse_expression(
                &line[(param_idx[param_idx.len() - 1] + 1)..(line.len() - 1)],
            )?);
        }
    }

    Ok(FunctionCall {
        name: func_name,
        arguments: params,
    })
}

/// 分析返回语句
pub fn parse_return(line: &[Token]) -> Result<Return> {
    debug!("{:?}", &line);

    // let var_type = match &line[0] {
    //     Token::Keyword(Keyword::LET) => VarType::Let,
    //     Token::Keyword(Keyword::CONST) => VarType::Const,
    //     _ => unreachable!(),
    // };

    let var = Return {
        expression: parse_expression(&line[1..])?,
    };
    Ok(var)
}

/// 分析声明语句
pub fn parse_declare(lines: &[Box<[Token]>], mut start_line: usize) -> Result<(usize, Local)> {
    let line = &lines[start_line];
    // let var_type = match &line[0] {
    //     Token::Keyword(Keyword::LET) => VarType::Let,
    //     Token::Keyword(Keyword::CONST) => VarType::Const,
    //     _ => unreachable!(),
    // };

    let name = match &line[1] {
        Token::Identifier(name) => name,
        _ => unreachable!(),
    };

    // 检查是否有初始值
    if line.len() < 4 || line[2] != Token::Operator(Operator::Assign) {
        return Err(err_msg("语法错误：变量定义必须有初始值。正确格式：let x = 表达式"));
    }

    // 检查是否是函数调用语法（不支持）
    if let Token::Identifier(_) = line[3] {
        if line.get(4) == Some(&Token::LParen) {
            return Err(err_msg("语法错误：不支持在变量声明中直接调用函数。请使用两行：\nlet x = 0\nx = test()"));
        }
    }

    let expr = if line[3] == Token::LBig {
        if line.last() == Some(&Token::RBig) {
             // Single line block
             start_line += 1;
             if line.len() == 5 {
                 // Empty block: let x = { }
                 Expression::Block(vec![])
             } else {
                 // Try to parse content as expression: let x = { expr }
                 let content = &line[4..line.len()-1];
                 let expr = parse_expression(content)?;
                 Expression::Block(vec![Statement::Expression(expr)])
             }
        } else {
            let (endline, stmts) = parse_block(lines, start_line + 1)?;
            start_line = endline + 1;
            Expression::Block(stmts)
        }
    } else {
        start_line += 1;
        parse_expression(&line[3..])?
    };

    let var = Local {
        name: name.clone(),
        expression: expr,
    };
    Ok((start_line, var))
}

fn parse_define_function(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, FunctionDeclaration)> {
    let func_name = if let Token::Identifier(name) = &lines[start_line][1] {
        name.to_string()
    } else {
        return Err(err_msg("不是函数定义语句"));
    };

    let (endline, body) = parse_block(lines, start_line + 1)?;

    let params = lines[start_line]
        .iter()
        .skip(3)
        .filter_map(|it| match it {
            Token::Identifier(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    let func = FunctionDeclaration {
        name: func_name,
        parameters: params,
        body,
    };
    Ok((endline, (func)))
}

/// 赋值语句分析
pub fn parse_assign(lines: &[Box<[Token]>], mut start_line: usize) -> Result<(usize, Assign)> {
    let line = &lines[start_line];
    debug!("{:?}", &line);

    match &line[0] {
            Token::Identifier(name) => {
                assert_eq!(&line[1], &Token::Operator(Operator::Assign));
                
                let expr = if line[2] == Token::LBig {
                    if line.last() == Some(&Token::RBig) {
                        start_line += 1;
                        Expression::Block(if line.len() == 4 { vec![] } else { vec![Statement::Expression(parse_expression(&line[3..line.len()-1])?)] })
                    } else {
                        let (endline, stmts) = parse_block(lines, start_line + 1)?;
                        start_line = endline + 1;
                        Expression::Block(stmts)
                    }
                } else {
                    let e = match &line[2] {
                        Token::Identifier(_) if line.get(3) == Some(&Token::LParen) => {
                            Expression::FunctionCall(parse_func_call(&line[2..])?)
                        }
                        _ => parse_expression(&line[2..])?,
                    };
                    start_line += 1;
                    e
                };

            let var = Assign {
                name: name.clone(),
                expr: Box::new(expr),
            };
            Ok((start_line, var))
        }
        _ => Err(err_msg(format!("赋值语句语法不对，{line:?}"))),
    }
}

/// 分析条件语句
pub fn parse_if(lines: &[Box<[Token]>], start_line: usize) -> Result<(usize, If)> {
    let (mut endline, if_cmd) = parse_block(lines, start_line + 1)?;
    let else_cmd = if let Some(Token::Keyword(Keyword::ELSE)) = lines[endline].get(1) {
        assert_eq!(lines[endline][0], Token::RBig);
        assert_eq!(lines[endline][2], Token::LBig);
        let (new_endline, cmd) = parse_block(lines, endline + 1)?;
        endline = new_endline;
        cmd
    } else {
        Vec::new()
    };
    let loop_expr = If {
        test: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        body: if_cmd,
        else_body: else_cmd,
    };
    Ok((endline, loop_expr))
}

/// 分析循环语句
pub fn parse_for(lines: &[Box<[Token]>], start_line: usize) -> Result<(usize, Loop)> {
    let cmd = parse_block(lines, start_line + 1)?;
    let loop_expr = Loop {
        test: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        body: cmd.1,
    };
    Ok((cmd.0, loop_expr))
}
