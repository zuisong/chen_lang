use crate::expression::*;

use crate::*;
use std::collections::VecDeque;

const H: i32 = 3;
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

pub fn parse_expression(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    if line.len() == 0 {
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
                Operator::ADD => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Add {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Subtract => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Subtract {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Multiply => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Multiply {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Divide => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Divide {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Mod => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Mod {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Assign => {
                    unreachable!();
                }
                Operator::And => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box And {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Equals => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Equals {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::NotEquals => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box NotEquals {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::Or => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box Or {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::NOT => {
                    box Not {
                        expr: tmp.pop_back().unwrap()
                    }
                }
                Operator::GT => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box GT {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::LT => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box LT {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::GTE => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box GTE {
                        left: o2,
                        right: o1,
                    }
                }
                Operator::LTE => {
                    let o1 = tmp.pop_back().unwrap();
                    let o2 = tmp.pop_back().unwrap();
                    box LTE {
                        left: o2,
                        right: o1,
                    }
                }
            };
            tmp.push_back(new_exp);
        } else {
            let ele: Element = match t {
                Token::Identifier(name) => Element::Variable(Variable { name }),
                Token::Int(i) => Element::Value(Value::Int(i)),
                Token::Bool(i) => Element::Value(Value::Bool(i)),
                Token::String(i) => Element::Value(Value::String(i)),
                _ => panic!("错误"),
            };
            tmp.push_back(Box::new(ele));
        }
    }

    return Ok(box box tmp);
}

pub fn parse_sequence(
    lines: &[Box<[Token]>],
    mut start_line: usize,
) -> Result<(usize, Command), failure::Error> {
    let mut v = VecDeque::new();
    while start_line < lines.len() && lines[start_line][0] != Token::RBig {
        match &lines[start_line][0] {
            Token::Keyword(Keyword::INT) => {
                let var = parse_var(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            Token::Keyword(Keyword::FOR) => {
                let var = parse_for(&lines[..], start_line)?;
                v.push_back(var.1);
                start_line = var.0 + 1;
            }
            Token::Keyword(Keyword::IF) => {
                let var = parse_if(&lines[..], start_line)?;
                v.push_back(var.1);
                start_line = var.0 + 1;
            }
            Token::StdFunction(StdFunction::Println) => {
                let var = parse_println(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            Token::StdFunction(StdFunction::Print) => {
                let var = parse_print(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            Token::Identifier(_) => {
                let var = parse_var(&lines[start_line])?;
                v.push_back(var);
                start_line += 1;
            }
            _ => {
                unimplemented!("", );
            }
        }
    }
    return Ok((start_line, Box::new(v)));
}

pub fn parse_var(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    debug!("{:?}", &line);

    match &line[0] {
        Token::Identifier(name) => {
            let var = Var {
                left: name.clone(),
                right: parse_expression(&line[2..])?,
            };
            return Ok(Box::new(var));
        }
        _ => {
            return Err(err_msg(format!("赋值语句语法不对，{:?}", line)));
        }
    };
}

pub fn parse_if(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, Box<dyn Expression>), failure::Error> {
    let cmd = parse_sequence(&lines, start_line + 1)?;
    let loop_expr = If {
        predict: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        cmd: cmd.1,
    };
    return Ok((cmd.0, Box::new(loop_expr)));
}

pub fn parse_for(
    lines: &[Box<[Token]>],
    start_line: usize,
) -> Result<(usize, Box<dyn Expression>), failure::Error> {
    let cmd = parse_sequence(&lines, start_line + 1)?;
    let loop_expr = Loop {
        predict: parse_expression(&lines[start_line][1..(lines[start_line].len() - 1)])?,
        cmd: cmd.1,
    };
    return Ok((cmd.0, Box::new(loop_expr)));
}

fn parse_println(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    Ok(Box::new(Println {
        expression: parse_expression(&line[2..(line.len() - 1)])?,
    }))
}

fn parse_print(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    Ok(Box::new(Print {
        expression: parse_expression(&line[2..(line.len() - 1)])?,
    }))
}
