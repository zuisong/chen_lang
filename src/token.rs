use std::{char, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("UnknownToken {token:?}")]
    UnknownToken { token: char },
    #[error("parse int error")]
    Disconnect(#[from] ParseIntError),
    #[error("unknown error")]
    Unknown,
}

/// 关键字
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Keyword {
    /// if
    IF,
    LET,
    /// else
    ELSE,
    /// for
    FOR,
    /// def
    DEF,
    /// return
    RETURN,
}

/// 操作符
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Operator {
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
    And,
    /// ==
    Equals,
    /// !=
    NotEquals,
    /// ||
    Or,
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

/// 标准库函数
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum StdFunction {
    /// print  bool表示是否换行
    Print(bool),
}

/// token 类型
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    /// 关键字
    Keyword(Keyword),
    /// 操作符
    Operator(Operator),
    /// int
    Int(i32),
    /// bool
    Bool(bool),
    /// string
    String(String),
    /// 标识符
    Identifier(String),
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
    /// 换行符
    NewLine,
    // 注释
    Comment,
    // 空格
    Space,
}

fn parse_token(chars: &Vec<char>, loc: &Location) -> Result<(Token, Location), TokenError> {
    let cur = *chars.get(loc.index).unwrap_or(&' ');
    let next = *chars.get(loc.index + 1).unwrap_or(&' ');
    let res = match cur {
        '#' => {
            let mut l = loc.incr();
            while chars[l.index] != '\n' {
                l = l.incr();
            }
            (Token::Comment, l.new_line())
        }
        '\n' | '\r' => (Token::NewLine, loc.new_line()),
        _ if cur.is_whitespace() => (Token::Space, loc.incr()),
        '{' => (Token::LBig, loc.incr()),
        '}' => (Token::RBig, loc.incr()),
        '[' => (Token::LSquare, loc.incr()),
        ']' => (Token::RSquare, loc.incr()),
        '(' => (Token::LParen, loc.incr()),
        ')' => (Token::RParen, loc.incr()),
        ':' => (Token::COLON, loc.incr()),
        ',' => (Token::COMMA, loc.incr()),
        '+' => (Token::Operator(Operator::ADD), loc.incr()),
        '*' => (Token::Operator(Operator::Multiply), loc.incr()),
        '/' => (Token::Operator(Operator::Divide), loc.incr()),
        '%' => (Token::Operator(Operator::Mod), loc.incr()),
        '=' if next == '=' => (Token::Operator(Operator::Equals), loc.incr2()),
        '=' if next != '=' => (Token::Operator(Operator::Assign), loc.incr()),
        '&' if next == '&' => (Token::Operator(Operator::And), loc.incr2()),
        '|' if next == '|' => (Token::Operator(Operator::Or), loc.incr2()),
        '!' if next == '=' => (Token::Operator(Operator::NotEquals), loc.incr2()),
        '!' if next != '=' => (Token::Operator(Operator::NOT), loc.incr()),
        '<' if next == '=' => (Token::Operator(Operator::LTE), loc.incr2()),
        '<' if next != '=' => (Token::Operator(Operator::LT), loc.incr()),
        '>' if next == '=' => (Token::Operator(Operator::GTE), loc.incr2()),
        '>' if next != '=' => (Token::Operator(Operator::GT), loc.incr()),
        '-' if !next.is_numeric() => (Token::Operator(Operator::Subtract), loc.incr()),
        '"' | '\'' => {
            let mut l = loc.incr();
            while cur != chars[l.index] {
                l = match chars[l.index] {
                    '\n' => l.new_line(),
                    _ => l.incr(),
                };
            }
            let s: String = chars.as_slice()[(loc.index + 1)..(l.index)]
                .iter()
                .collect();
            (Token::String(s), l.incr())
        }
        _ if cur == '-' || cur.is_numeric() => {
            let mut l = loc.incr();
            while chars[l.index].is_numeric() {
                l = l.incr();
            }

            let s: String = chars
                .iter()
                .skip(loc.index)
                .take(l.index - loc.index)
                .collect();

            (Token::Int(s.parse()?), l)
        }

        _ if cur.is_ascii_alphabetic() => {
            let mut l = loc.incr();
            while l.index < chars.len()
                && matches!(chars[l.index], 'A'..='Z' | 'a'..='z' | '0'..='9')
            {
                l = l.incr();
            }

            let s: String = chars.as_slice()[loc.index..l.index].iter().collect();
            let token = match s.as_str() {
                "let" => Token::Keyword(Keyword::LET),
                "return" => Token::Keyword(Keyword::RETURN),
                "if" => Token::Keyword(Keyword::IF),
                "def" => Token::Keyword(Keyword::DEF),
                "else" => Token::Keyword(Keyword::ELSE),
                "for" => Token::Keyword(Keyword::FOR),
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                _ => Token::Identifier(s),
            };
            (token, l)
        }
        _ => {
            return Err(TokenError::UnknownToken { token: cur });
        }
    };
    return Ok(res);
}

/// 代码转成token串
pub fn tokenlizer(code: String) -> Result<Vec<Token>, TokenError> {
    let chars: Vec<_> = code.chars().collect();

    let mut tokens = vec![];

    let mut loc = Location::default();
    while loc.index < chars.len() {
        let (token, new_loc) = parse_token(&chars, &loc)?;
        if !matches!(token, Token::Comment | Token::Space) {
            tokens.push(token);
        }
        loc = new_loc;
    }

    Ok(tokens)
}

#[derive(Copy, Clone, Debug)]
pub struct Location {
    col: usize,
    line: usize,
    index: usize,
}

impl Default for Location {
    fn default() -> Self {
        Location {
            col: 1,
            line: 1,
            index: 0,
        }
    }
}

impl Location {
    fn new_line(&self) -> Location {
        Location {
            index: self.index + 1,
            col: 1,
            line: self.line + 1,
        }
    }
    #[inline]
    fn incr(&self) -> Location {
        self.incr_n(1)
    }
    #[inline]
    fn incr2(&self) -> Location {
        self.incr_n(2)
    }
    #[inline]
    fn incr_n(&self, n: usize) -> Location {
        Location {
            index: self.index + n,
            col: self.col + n,
            line: self.line,
        }
    }

    pub fn debug<S: Into<String>>(&self, raw: &[char], msg: S) -> String {
        let mut line = 0;
        let mut line_str = String::new();
        // Find the whole line of original source
        for c in raw {
            if *c == '\n' {
                line += 1;

                // Done discovering line in question
                if !line_str.is_empty() {
                    break;
                }

                continue;
            }

            if self.line == line {
                line_str.push_str(&c.to_string());
            }
        }

        let space = " ".repeat(self.col as usize);
        format!("{}\n\n{}\n{}^ Near here", msg.into(), line_str, space)
    }
}
