use std::num::ParseIntError;

#[allow(unused_imports)]
use log::*;
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
    CONST,
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
    /// 标准库函数
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
    /// 换行符
    NewLine,
}

/// 代码转成token串
pub fn tokenlizer(code: String) -> Result<Vec<Token>, TokenError> {
    let chars: Vec<_> = code.chars().collect();

    let mut tokens = vec![];

    let mut i = 0;
    while i < chars.len() {
        let (token, size) = match chars[i] {
            '#' => {
                while chars[i] != '\r' && chars[i] != '\n' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            '\r' | '\n' => (Token::NewLine, 1),
            '{' => (Token::LBig, 1),
            '}' => (Token::RBig, 1),
            '[' => (Token::LSquare, 1),
            ']' => (Token::RSquare, 1),
            '(' => (Token::LParen, 1),
            ')' => (Token::RParen, 1),
            ':' => (Token::COLON, 1),
            ',' => (Token::COMMA, 1),
            '+' => (Token::Operator(Operator::ADD), 1),
            '*' => (Token::Operator(Operator::Multiply), 1),
            '/' => (Token::Operator(Operator::Divide), 1),
            '%' => (Token::Operator(Operator::Mod), 1),
            '=' if chars[i + 1] == '=' => (Token::Operator(Operator::Equals), 2),
            '=' if chars[i + 1] != '=' => (Token::Operator(Operator::Assign), 1),
            '&' if chars[i + 1] == '&' => (Token::Operator(Operator::And), 2),
            '|' if chars[i + 1] == '|' => (Token::Operator(Operator::Or), 2),
            '!' if chars[i + 1] == '=' => (Token::Operator(Operator::NotEquals), 2),
            '!' if chars[i + 1] != '=' => (Token::Operator(Operator::NOT), 1),
            '<' if chars[i + 1] == '=' => (Token::Operator(Operator::LTE), 2),
            '<' if chars[i + 1] != '=' => (Token::Operator(Operator::LT), 1),
            '>' if chars[i + 1] == '=' => (Token::Operator(Operator::GTE), 2),
            '>' if chars[i + 1] != '=' => (Token::Operator(Operator::GT), 1),
            '-' if !chars[i + 1].is_numeric() => (Token::Operator(Operator::Subtract), 1),

            '"' | '\'' => {
                let mut j = i + 1;

                while chars[i] != chars[j] {
                    j += 1;
                }
                let s: String = chars.as_slice()[(i + 1)..j].iter().collect();
                (Token::String(s), j + 1 - i)
            }
            _ if chars[i] == '-' || chars[i].is_numeric() => {
                let mut j = i + 1;
                while chars[j].is_numeric() {
                    j += 1;
                }

                let s: String = chars.iter().skip(i).take(j - i).collect();
                (Token::Int(s.parse()?), j - i)
            }

            _ if chars[i].is_ascii_alphabetic() => {
                let mut j = i + 1;

                while chars[j].is_ascii_alphabetic() || chars[j].is_numeric() {
                    j += 1;
                }
                let s: String = chars.as_slice()[i..j].iter().collect();
                let token = match s.as_str() {
                    "println" => Token::StdFunction(StdFunction::Print(true)),
                    "print" => Token::StdFunction(StdFunction::Print(false)),
                    "let" => Token::Keyword(Keyword::LET),
                    "return" => Token::Keyword(Keyword::RETURN),
                    "const" => Token::Keyword(Keyword::CONST),
                    "if" => Token::Keyword(Keyword::IF),
                    "def" => Token::Keyword(Keyword::DEF),
                    "else" => Token::Keyword(Keyword::ELSE),
                    "for" => Token::Keyword(Keyword::FOR),
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Identifier(s),
                };
                (token, j - i)
            }
            _ if chars[i].is_whitespace() => {
                i += 1;
                continue;
            }
            _ => {
                return Err(TokenError::UnknownToken { token: chars[i] });
            }
        };
        tokens.push(token);
        i += size;
    }

    Ok(tokens)
}
