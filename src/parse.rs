#![deny(missing_docs)]

use std::collections::VecDeque;

use crate::context::VarType;
use crate::*;
use failure::err_msg;
use std::rc::Rc;

//const H: i32 = 3;
const M: i32 = 2;
const S: i32 = 1;

fn get_priority(token: &Token) -> i32 {
    match token {
        Token::LParen => 0,
        Token::RParen => 0,
        Token::Operator(opt) => match opt {
            Operator::ADD => S,
            Operator::Subtract => S,
            Operator::Multiply => M,
            Operator::Divide => M,
            Operator::Mod => M,
            Operator::Assign => S,
            Operator::And => -1,
            Operator::Equals => M,
            Operator::NotEquals => M,
            Operator::Or => -1,
            Operator::NOT => 0,
            Operator::GT => M,
            Operator::LT => M,
            Operator::GTE => M,
            Operator::LTE => M,
        },
        _ => 100,
    }
}

/// 简单表达式分析 (只有运算的 一行)
pub fn parse_expression(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    if line.is_empty() {
        return Err(failure::err_msg("不是一个表达式"));
    }

    // 中缀表达式变后缀表达式
    let mut result = VecDeque::new();
    let mut stack = vec![];
    for token in line {
        loop {
            let o2 = get_priority(token);
            if o2 == 100 {
                result.push_back(token.clone());
                break;
            }
            match stack.last() {
                Some(o) => match token {
                    Token::LParen => {
                        stack.push(token.clone());
                        break;
                    }
                    Token::RParen => {
                        if o == &Token::LParen {
                            stack.pop().unwrap();
                            break;
                        } else {
                            result.push_back(stack.pop().unwrap());
                        }
                    }
                    _ => {
                        let o1 = get_priority(o);
                        if o1 < o2 {
                            stack.push(token.clone());
                            break;
                        } else {
                            result.push_back(stack.pop().unwrap());
                        }
                    }
                },
                None => {
                    stack.push(token.clone());
                    break;
                }
            }
        }
    }
    while let Some(t) = stack.pop() {
        result.push_back(t);
    }
    let mut result: VecDeque<_> = result
        .into_iter()
        .filter(|it| it != &Token::LParen && it != &Token::RParen)
        .collect();

    //    dbg!(&result);

    let mut tmp: VecDeque<Box<dyn Expression>> = VecDeque::new();

    while let Some(t) = result.pop_front() {
        if let Token::Operator(opt) = t {
            let new_exp: Box<dyn Expression> = match opt {
                Operator::Assign => {
                    unreachable!();
                }

                Operator::NOT => box NotStatement {
                    expr: tmp.pop_back().unwrap(),
                },

                _ => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box BinaryStatement {
                        left: o2,
                        right: o1,
                        operator: opt,
                    }
                }
            };
            tmp.push_back(new_exp);
        } else {
            let ele: Element = match t {
                Token::Identifier(name) => Element::Variable(VariableStatement { name }),
                Token::Int(i) => Element::Value(Value::Int(i)),
                Token::Bool(i) => Element::Value(Value::Bool(i)),
                Token::String(i) => Element::Value(Value::Str(i)),
                _ => panic!("错误"),
            };
            tmp.push_back(box ele);
        }
    }

    Ok(box tmp)
}

/// 分析很多行的方法
pub fn parse_block(
    lines: &[Box<[Token]>],
    mut start_line: usize,
) -> Result<(usize, BlockStatement), failure::Error> {
    let mut v = VecDeque::new();
    while start_line < lines.len() && lines[start_line][0] != Token::RBig {
        match &lines[start_line][0] {
            Token::Keyword(Keyword::LET) | Token::Keyword(Keyword::CONST) => {
                let var = parse_declare(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            Token::Keyword(Keyword::FOR) => {
                let var = parse_for(&lines[..], start_line)?;
                v.push_back(var.1);
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::DEF) => {
                let var = parse_define_function(&lines[..], start_line)?;
                v.push_back(var.1);
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::IF) => {
                let var = parse_if(&lines[..], start_line)?;
                v.push_back(var.1);
                start_line = var.0 + 1;
            }
            Token::StdFunction(StdFunction::Print(is_newline)) => {
                let var = parse_print(&lines[start_line], *is_newline)?;
                v.push_back(var);
                start_line += 1;
            }
            // 赋值
            Token::Identifier(_) if lines[start_line][1] == Token::Operator(Operator::Assign) => {
                let var = parse_assign(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            // 函数调用
            Token::Identifier(_) if lines[start_line][1] == Token::LParen => {
                let var = parse_func_call(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            Token::LBig => {
                let var = parse_block(&lines[..], start_line + 1)?;
                v.push_back(box var.1);
                start_line += var.0 + 1;
            }
            _ => {
                unimplemented!("{:?}", lines[start_line]);
            }
        }
    }
    Ok((start_line, v))
}

fn parse_func_call(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    unimplemented!()
}

/// 分析赋值语句
pub fn parse_declare(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    debug!("{:?}", &line);

    let var_type = match &line[0] {
        Token::Keyword(Keyword::LET) => VarType::Let,
        Token::Keyword(Keyword::CONST) => VarType::Const,
        _ => unreachable!(),
    };

    let name = match &line[1] {
        Token::Identifier(name) => name,
        _ => unreachable!(),
    };

    let var = DeclareStatement {
        var_type,
        left: name.clone(),
        right: parse_expression(&line[3..])?,
    };
    Ok(box var)
}

///
/// ```java
/// def f1(){
///
/// }
///  def f2(a,b,c){
///
/// }
/// ```
///
fn parse_define_function(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, Box<dyn Expression>), failure::Error> {
    let func_name = if let Token::Identifier(name) = &lines[start_line][1] {
        name.to_string()
    } else {
        return Err(err_msg(""));
    };

    let (endline, body) = parse_block(&lines, start_line + 1)?;
    let func = FunctionStatement {
        name: func_name,
        // todo params parse
        params: vec![],
        body: Rc::new(body),
    };
    Ok((endline, box func))
}

///
/// 赋值语句分析
pub fn parse_assign(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    debug!("{:?}", &line);

    match &line[0] {
        Token::Identifier(name) => {
            let var = AssignStatement {
                left: name.clone(),
                right: parse_expression(&line[2..])?,
            };
            Ok(box var)
        }
        _ => Err(err_msg(format!("赋值语句语法不对，{:?}", line))),
    }
}

/// 分析条件语句
pub fn parse_if(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, Box<dyn Expression>), failure::Error> {
    let (mut endline, if_cmd) = parse_block(&lines, start_line + 1)?;
    let else_cmd = if let Some(Token::Keyword(Keyword::ELSE)) = lines[endline].get(1) {
        assert_eq!(lines[endline][0], Token::RBig);
        assert_eq!(lines[endline][2], Token::LBig);
        let (new_endline, cmd) = parse_block(&lines, endline + 1)?;
        endline = new_endline;
        cmd
    } else {
        VecDeque::new()
    };
    let loop_expr = IfStatement {
        predict: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        if_block: if_cmd,
        else_block: else_cmd,
    };
    Ok((endline, box loop_expr))
}

/// 分析循环语句
pub fn parse_for(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, Box<dyn Expression>), failure::Error> {
    let cmd = parse_block(&lines, start_line + 1)?;
    let loop_expr = LoopStatement {
        predict: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        loop_block: cmd.1,
    };
    Ok((cmd.0, box loop_expr))
}

fn parse_print(line: &[Token], is_newline: bool) -> Result<Box<dyn Expression>, failure::Error> {
    debug!("{:?}", line);
    let expression = parse_expression(&line[2..(line.len() - 1)])?;
    Ok(box PrintStatement {
        expression,
        is_newline,
    })
}
