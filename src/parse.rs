use crate::expression::*;

use crate::*;
use std::collections::VecDeque;

pub fn parse_expression(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    if line.len() == 0 {
        return Err(failure::err_msg("不是一个表达式"));
    }

    if line.len() == 1 {
        return match &line[0] {
            Token::Num(i) => Ok(Box::new(Const::Int(*i))),
            Token::Identifier(name) => Ok(Box::new(Variable { name: name.clone() })),
            _ => Err(failure::err_msg(format!("错误的表达式, {:?}", line))),
        };
    }

    if let Token::Operator(Operator::ADD) = line[1] {
        return Ok(Box::new(Add {
            left: parse_expression(&line[0..1])?,
            right: parse_expression(&line[2..])?,
        }));
    } else {
        return Err(failure::err_msg(format!(
            "暂未支持 其他她运算符,{:?}",
            line
        )));
    }
}

pub fn parse_sequence(
    lines: &[Box<[Token]>],
    mut start_line: usize,
) -> Result<(usize, Cmd), failure::Error> {
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

fn parse_print(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    Ok(Box::new(Print {
        expression: parse_expression(&line[2..(line.len() - 1)])?,
    }))
}
