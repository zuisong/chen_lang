use std::{char, num::ParseIntError};

use anyhow::anyhow;
use thiserror::Error;
use tracing::debug;
use winnow::{
    ascii::not_line_ending,
    ascii::{alphanumeric1, digit1, line_ending},
    combinator::separated_pair,
    combinator::{alt, delimited, not, opt},
    token::one_of,
    token::tag,
    token::take_until0,
    IResult, Parser,
};

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

fn parse_with_winnow(chars: &str) -> IResult<&str, Token> {
    alt((
        separated_pair(tag("#"), not_line_ending, line_ending).map(|_| Token::Comment),
        alt((
            line_ending.value(Token::NewLine),
            one_of((' ', '\t', '\r', '\n')).value(Token::Space),
            tag("{").value(Token::LBig),
            tag("}").value(Token::RBig),
            tag("[").value(Token::LSquare),
            tag("]").value(Token::RSquare),
            tag("(").value(Token::LParen),
            tag(")").value(Token::RParen),
            tag(":").value(Token::COLON),
            tag(",").value(Token::COMMA),
        )),
        alt((
            tag("+").value(Token::Operator(Operator::ADD)),
            tag("*").value(Token::Operator(Operator::Multiply)),
            tag("/").value(Token::Operator(Operator::Divide)),
            tag("%").value(Token::Operator(Operator::Mod)),
            tag("==").value(Token::Operator(Operator::Equals)),
            tag("=").value(Token::Operator(Operator::Assign)),
            tag("&&").value(Token::Operator(Operator::And)),
            tag("||").value(Token::Operator(Operator::Or)),
            tag("!=").value(Token::Operator(Operator::NotEquals)),
            tag("!").value(Token::Operator(Operator::NOT)),
            tag("<=").value(Token::Operator(Operator::LTE)),
            tag("<").value(Token::Operator(Operator::LT)),
            tag(">=").value(Token::Operator(Operator::GTE)),
            tag(">").value(Token::Operator(Operator::GT)),
            (tag("-"), not(digit1)).value(Token::Operator(Operator::Subtract)),
            alt((
                delimited(tag("\""), take_until0("\""), tag("\"")),
                delimited(tag("\'"), take_until0("\'"), tag("\'")),
            ))
                .map(|s: &str| Token::String(s.to_string())),
            //
            (opt(tag("-")), digit1).try_map(|(sig, s): (Option<&str>, &str)| {
                s.parse::<i32>().map(|i| match sig {
                    Some(_) => Token::Int(-i),
                    None => Token::Int(i),
                })
            }),
            alphanumeric1.map(|arr: &str| {
                let s = arr;
                match s {
                    "let" => Token::Keyword(Keyword::LET),
                    "return" => Token::Keyword(Keyword::RETURN),
                    "if" => Token::Keyword(Keyword::IF),
                    "def" => Token::Keyword(Keyword::DEF),
                    "else" => Token::Keyword(Keyword::ELSE),
                    "for" => Token::Keyword(Keyword::FOR),
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Identifier(s.to_string()),
                }
            }),
        )),
    ))
        .parse_peek(chars)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_matches;

    use crate::token::parse_with_winnow;
    use crate::Operator;
    use crate::Token;

    #[test]
    fn test() {
        assert_matches!(parse_with_winnow("-1"), Ok(("", Token::Int(-1))));
        assert_matches!(
            parse_with_winnow("-a"),
            Ok(("a", Token::Operator(Operator::Subtract)))
        );
        assert_matches!(parse_with_winnow("10a"), Ok(("a", Token::Int(10))));
        assert_matches!(parse_with_winnow("\"aaaa\""), Ok(("", Token::String(ref a))) if a == "aaaa");
        assert_matches!(parse_with_winnow("\'aaaa\'"),Ok(("", Token::String(ref a))) if a == "aaaa");
        assert_matches!(parse_with_winnow("\'\'"), Ok(("", Token::String(ref a))) if a== "" );
    }
}

#[allow(unused)]
fn parse_token(input: &str, loc: &Location) -> Result<(Token, Location), TokenError> {
    let chars: Vec<char> = input.chars().collect();
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
            let s: String = chars.as_slice()[(loc.index + 1)..l.index].iter().collect();
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
    Ok(res)
}

/// 代码转成token串
pub fn tokenlizer(code: String) -> anyhow::Result<Vec<Token>> {
    let mut input = code.as_str();

    let mut tokens = vec![];

    loop {
        debug!(?input);
        let (remain_input, token) =
            parse_with_winnow(input).map_err(|e| anyhow!(e.to_string()))?;
        if !matches!(token, Token::Comment | Token::Space) {
            tokens.push(token);
        }
        if remain_input.is_empty() {
            break;
        }
        input = remain_input
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

        let space = " ".repeat(self.col);
        format!("{}\n\n{}\n{}^ Near here", msg.into(), line_str, space)
    }
}
