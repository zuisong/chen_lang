#[allow(unused_imports)]
#[allow(unused_parens)]
#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unreachable_code)]
///
/// 关键字   if for int
/// 函数库   print
/// 操作符  = +-*/  ==
/// 逻辑运算符  && || ！
/// 标识符   纯字母
///
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use failure::*;

const KEY_WORD: [&str; 3] = ["int", "for", "if"];

#[derive(Debug, PartialEq)]
enum Keyword {
    INT,
    IF,
    FOR,
}

#[derive(Debug, PartialEq)]
enum Operator {
    /// +
    ADD,
    /// -
    Subtract,
    /// *
    Multiply,
    /// /
    Divide,
    /// %
    Mod,
    /// =
    Assign,
    /// &&
    AND,
    /// ==
    Equals,
    /// !=
    NotEquals,
    /// ||
    OR,
    /// !
    NOT,
    /// >
    GT,
    /// <
    LT,
    /// >=
    GTE,
    /// <=
    LTE,
}

#[derive(Debug, PartialEq)]
enum StdFunction {
    Print,
}

#[derive(Debug, PartialEq)]
enum Token {
    Keyword(Keyword),
    Operator(Operator),
    Num(i32),
    NumFloat(f32),
    Identifier(String),
    StdFunction(StdFunction),
    /// 左大括号
    LBig,
    /// 右大括号
    RBig,
    /// 左方括号
    LSquare,
    /// 右方括号
    RSquare,
    /// 冒号
    COLON,
    /// 逗号
    COMMA,
    /// (
    LParen,
    /// )
    RParen,
    NewLine,
}

pub fn run(code: String) -> Result<(), failure::Error> {
    let tokens = tokenlizer(code)?;
    dbg!(&tokens);
    let ast = parser(tokens)?;
    evlate(ast);
    Ok(())
}

fn parse_expression(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
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

fn parse_sequence(
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

fn parse_var(line: &[Token]) -> Result<Box<dyn Expression>, failure::Error> {
    dbg!(&line);

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
    unreachable!("")
}

fn parse_for(
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

fn parser(tokens: Vec<Token>) -> Result<Cmd, failure::Error> {
    let mut lines: Vec<Box<[Token]>> = vec![];

    let mut temp = vec![];
    for x in tokens {
        if let Token::NewLine = x {
            if !temp.is_empty() {
                lines.push(temp.into_boxed_slice());
                temp = vec![];
            }
        } else {
            temp.push(x)
        }
    }
    let (_, ast) = parse_sequence(lines.as_slice(), 0)?;

    return Ok(ast);
}

fn evlate(ast: Cmd) {
    let mut ctx = Context {
        output: vec![],
        variables: Default::default(),
    };
    for cmd in ast.iter() {
        cmd.evaluate(&mut ctx);
    }

    dbg!(&ast);

    for x in ctx.output {
        println!("{}", x.to_string());
    }
}

fn tokenlizer(code: String) -> Result<Vec<Token>, failure::Error> {
    let chars: Vec<_> = code.chars().collect();

    let mut tokens = vec![];

    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\n' {
            i += 1;
            tokens.push(Token::NewLine);
            continue;
        }

        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i] == '{' {
            tokens.push(Token::LBig);
            i += 1;
            continue;
        }
        if chars[i] == '}' {
            tokens.push(Token::RBig);
            i += 1;
            continue;
        }
        if chars[i] == '}' {
            tokens.push(Token::RBig);
            i += 1;
            continue;
        }
        if chars[i] == '[' {
            tokens.push(Token::LSquare);
            i += 1;
            continue;
        }

        if chars[i] == ']' {
            tokens.push(Token::RSquare);
            i += 1;
            continue;
        }

        if chars[i] == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }

        if chars[i] == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }

        if chars[i] == ':' {
            tokens.push(Token::COLON);
            i += 1;
            continue;
        }
        if chars[i] == ',' {
            tokens.push(Token::COMMA);
            i += 1;
            continue;
        }

        if chars[i] == '+' {
            tokens.push(Token::Operator(Operator::ADD));
            i += 1;
            continue;
        }

        if chars[i] == '*' {
            tokens.push(Token::Operator(Operator::Multiply));
            i += 1;
            continue;
        }
        if chars[i] == '/' {
            tokens.push(Token::Operator(Operator::Divide));
            i += 1;
            continue;
        }

        if chars[i] == '%' {
            tokens.push(Token::Operator(Operator::Mod));
            i += 1;
            continue;
        }

        if chars[i] == '=' {
            if chars[i + 1] == '=' {
                tokens.push(Token::Operator(Operator::Equals));
                i += 2;
            } else {
                tokens.push(Token::Operator(Operator::Assign));
                i += 1;
            }
            continue;
        }
        if chars[i] == '!' {
            if chars[i + 1] == '=' {
                tokens.push(Token::Operator(Operator::NotEquals));
                i += 2;
            } else {
                tokens.push(Token::Operator(Operator::NOT));
                i += 1;
            }
            continue;
        }

        if chars[i] == '<' {
            if chars[i + 1] == '=' {
                tokens.push(Token::Operator(Operator::LTE));
                i += 2;
            } else {
                tokens.push(Token::Operator(Operator::LT));
                i += 1;
            }
            continue;
        }
        if chars[i] == '>' {
            if chars[i + 1] == '=' {
                tokens.push(Token::Operator(Operator::GTE));
                i += 2;
            } else {
                tokens.push(Token::Operator(Operator::GT));
                i += 1;
            }
            continue;
        }

        if chars[i] == '-' && chars[i + 1].is_whitespace() {
            tokens.push(Token::Operator(Operator::Subtract));
            i += 1;
            continue;
        }

        if chars[i].is_numeric() || chars[i] == '-' {
            let mut j = i + 1;
            let mut is_float = false;
            while chars[j].is_numeric() || chars[j] == '.' {
                j += 1;
                if chars[j] == '.' {
                    is_float = true;
                }
            }

            let s: String = chars.iter().skip(i).take(j - i).collect();
            if is_float {
                tokens.push(Token::NumFloat(s.parse()?));
            } else {
                tokens.push(Token::Num(s.parse()?));
            }

            i = j;
            continue;
        }

        if chars[i].is_ascii_alphabetic() {
            let mut j = i + 1;

            while chars[j].is_ascii_alphabetic() {
                j += 1;
            }

            let s: String = chars.iter().skip(i).take(j - i).collect();

            match s.as_str() {
                "print" => {
                    tokens.push(Token::StdFunction(StdFunction::Print));
                }
                "int" => {
                    tokens.push(Token::Keyword(Keyword::INT));
                }
                "if" => {
                    tokens.push(Token::Keyword(Keyword::IF));
                }
                "for" => {
                    tokens.push(Token::Keyword(Keyword::FOR));
                }
                _ => {
                    tokens.push(Token::Identifier(s));
                }
            }
            i = j;
            continue;
        }

        return Err(failure::err_msg(format!(
            "token 解析错误  请检查语法, {}",
            chars[i]
        )));
    }

    Ok(tokens)
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Const {
    // 仅支持 int  bool类型
    Int(i32),
    Bool(bool),
    Void,
}

impl Expression for Const {
    fn evaluate(&self, _: &mut Context) -> Option<Const> {
        Some(self.clone())
    }
}

impl ToString for Const {
    fn to_string(&self) -> String {
        match self {
            Const::Int(int) => (*int).to_string(),
            Const::Bool(b) => (*b).to_string(),
            Const::Void => String::new(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Context {
    output: Vec<Const>,
    variables: HashMap<String, Const>,
}

#[derive(Debug)]
struct Add {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl Expression for Add {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let l = self.left.evaluate(ctx);
        let r = self.right.evaluate(ctx);
        match (l, r) {
            (Some(Const::Int(l_int)), Some(Const::Int(r_int))) => Some(Const::Int(l_int + r_int)),
            (_, _) => panic!("不是数字不能做加运算"),
        }
    }
}

#[derive(Debug)]
struct Print {
    expression: Box<dyn Expression>,
}

impl Expression for Print {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let res = self.expression.evaluate(ctx).unwrap();
        ctx.output.push(res);
        None
    }
}

// 赋值语句
#[derive(Debug)]
struct Var {
    left: String,
    right: Box<dyn Expression>,
}

impl Expression for Var {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        let e = &self.right;
        let res = e.evaluate(ctx).unwrap().clone();
        ctx.variables.insert((&self.left).clone(), res);
        None
    }
}

type Cmd = Box<VecDeque<Box<dyn Expression>>>;

impl Expression for Cmd {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        for expre in self.iter() {
            expre.evaluate(ctx);
        }
        None
    }
}

#[derive(Debug)]
struct Loop {
    predict: Box<dyn Expression>,
    cmd: Cmd,
}

impl Expression for Loop {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const> {
        loop {
            match self.predict.evaluate(ctx) {
                Some(Const::Int(0)) => {
                    break;
                }
                Some(Const::Int(_)) => {
                    self.cmd.evaluate(ctx);
                }
                _ => {
                    panic!("循环判断条件 返回值 只能是 bool 类型");
                }
            }
        }
        None
    }
}

enum Element {
    /// 变量
    Variable(Variable),
    /// 常量
    Const(Const),
}

trait Expression: Debug {
    fn evaluate(&self, ctx: &mut Context) -> Option<Const>;
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Variable {
    name: String,
}

impl Expression for Variable {
    fn evaluate(&self, context: &mut Context) -> Option<Const> {
        return Some(context.variables.get(&self.name).unwrap().clone());
    }
}
