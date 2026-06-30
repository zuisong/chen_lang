use std::num::ParseIntError;

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("UnknownToken {token:?}")]
    UnknownToken { token: char },
    #[error("parse int error")]
    Disconnect(#[from] ParseIntError),
    #[error("parse decimal error")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Parse error at line {line}: {msg}")]
    ParseErrorWithLocation { msg: String, line: u32 },
    #[error("unknown error")]
    Unknown,
}

/// 关键字 (Luau 风格)
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Keyword {
    IF,
    THEN,
    ELSE,
    ELSEIF,
    END,
    /// local (replaces let)
    LOCAL,
    /// function (replaces def)
    FUNCTION,
    RETURN,
    BREAK,
    CONTINUE,
    WHILE,
    DO,
    REPEAT,
    UNTIL,
    FOR,
    IN,
    AND,
    OR,
    NOT,
    NIL,
    TRUE,
    FALSE,
    TRY,
    CATCH,
    FINALLY,
}

/// 操作符 (Luau 风格)
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    FloorDiv,
    Concat,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    FloorDivAssign,
    ModAssign,
    ConcatAssign,
    Equals,
    NotEquals,
    Lt,
    LtE,
    Gt,
    GtE,
    And,
    Or,
    Not,
    Len,
}

/// token 类型 (Luau 风格)
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Operator(Operator),
    Int(i32),
    Float(Decimal),
    Bool(bool),
    String(String),
    Identifier(String),
    Dot,
    LBig,
    RBig,
    LSquare,
    RSquare,
    Colon,
    COMMA,
    LParen,
    RParen,
    NewLine,
    Space,
    Vararg,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub col: u32,
    pub line: u32,
    pub index: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
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
            col: self.col + n as u32,
            line: self.line,
        }
    }
}

/// 默认使用手写分词器 (handwritten) 因为它目前更稳定且支持行号追踪
#[cfg(not(feature = "winnow-tokenizer"))]
pub fn tokenizer(code: String) -> Result<Vec<(Token, Location)>, TokenError> {
    handwritten::tokenizer(code)
}

/// 使用 Feature Flag 启用 Winnow 分词器
#[cfg(feature = "winnow-tokenizer")]
pub fn tokenizer(code: String) -> Result<Vec<(Token, Location)>, TokenError> {
    winnow::tokenizer(code)
}

/// 仅用于测试/教学目的，显式调用手写分词器
pub fn tokenizer_handwritten(code: String) -> Result<Vec<(Token, Location)>, TokenError> {
    handwritten::tokenizer(code)
}

#[cfg(feature = "winnow-tokenizer")]
pub mod winnow {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use tracing::debug;
    use winnow::{
        ModalResult, Parser,
        ascii::{digit1, line_ending, till_line_ending},
        combinator::{alt, delimited, opt},
        token::{literal, one_of, take_until, take_while},
    };

    use super::{Keyword, Location, Operator, Token, TokenError};

    pub fn parse_with_winnow(chars: &str) -> ModalResult<(&str, Token)> {
        alt((
            (literal("--"), till_line_ending).map(|_| Token::Space),
            alt((
                line_ending.value(Token::NewLine),
                one_of((' ', '\t', '\r', '\n')).value(Token::Space),
                literal("{").value(Token::LBig),
                literal("}").value(Token::RBig),
                literal("[").value(Token::LSquare),
                literal("]").value(Token::RSquare),
                literal("(").value(Token::LParen),
                literal(")").value(Token::RParen),
                literal(":").value(Token::Colon),
                literal(",").value(Token::COMMA),
            )),
            alt((
                alt((
                    literal("..=").value(Token::Operator(Operator::ConcatAssign)),
                    literal("..").value(Token::Operator(Operator::Concat)),
                    literal(".").value(Token::Dot),
                    literal("+=").value(Token::Operator(Operator::AddAssign)),
                    literal("+").value(Token::Operator(Operator::Add)),
                    literal("-=").value(Token::Operator(Operator::SubAssign)),
                    literal("-").value(Token::Operator(Operator::Subtract)),
                    literal("*=").value(Token::Operator(Operator::MulAssign)),
                    literal("*").value(Token::Operator(Operator::Multiply)),
                    literal("//=").value(Token::Operator(Operator::FloorDivAssign)),
                    literal("//").value(Token::Operator(Operator::FloorDiv)),
                    literal("/=").value(Token::Operator(Operator::DivAssign)),
                )),
                alt((
                    literal("/").value(Token::Operator(Operator::Divide)),
                    literal("%=").value(Token::Operator(Operator::ModAssign)),
                    literal("%").value(Token::Operator(Operator::Mod)),
                    literal("==").value(Token::Operator(Operator::Equals)),
                    literal("=").value(Token::Operator(Operator::Assign)),
                    literal("~=").value(Token::Operator(Operator::NotEquals)),
                    literal("<=").value(Token::Operator(Operator::LtE)),
                    literal("<").value(Token::Operator(Operator::Lt)),
                    literal(">=").value(Token::Operator(Operator::GtE)),
                    literal(">").value(Token::Operator(Operator::Gt)),
                    literal("#").value(Token::Operator(Operator::Len)),
                )),
                alt((
                    delimited(literal("\""), take_until(0.., "\""), literal("\"")),
                    delimited(literal("'"), take_until(0.., "'"), literal("'")),
                ))
                .map(|s: &str| Token::String(s.to_string())),
                //
                // 浮点数解析（必须在整数之前，因为更具体）
                (digit1, literal("."), opt(digit1)).try_map(|(int_part, _, frac_part): (&str, _, Option<&str>)| {
                    let frac = frac_part.unwrap_or("0");
                    let float_str = format!("{}.{}", int_part, frac);
                    Decimal::from_str(&float_str).map(Token::Float)
                }),
                // 整数解析
                digit1.try_map(|s: &str| s.parse::<i32>().map(Token::Int)),
                take_while(1.., |c: char| c.is_alphanumeric() || c == '_').map(|arr: &str| {
                    let s = arr;
                    match s {
                        "local" => Token::Keyword(Keyword::LOCAL),
                        "function" => Token::Keyword(Keyword::FUNCTION),
                        "return" => Token::Keyword(Keyword::RETURN),
                        "if" => Token::Keyword(Keyword::IF),
                        "then" => Token::Keyword(Keyword::THEN),
                        "else" => Token::Keyword(Keyword::ELSE),
                        "elseif" => Token::Keyword(Keyword::ELSEIF),
                        "end" => Token::Keyword(Keyword::END),
                        "while" => Token::Keyword(Keyword::WHILE),
                        "do" => Token::Keyword(Keyword::DO),
                        "repeat" => Token::Keyword(Keyword::REPEAT),
                        "until" => Token::Keyword(Keyword::UNTIL),
                        "for" => Token::Keyword(Keyword::FOR),
                        "in" => Token::Keyword(Keyword::IN),
                        "break" => Token::Keyword(Keyword::BREAK),
                        "continue" => Token::Keyword(Keyword::CONTINUE),
                        "and" => Token::Keyword(Keyword::AND),
                        "or" => Token::Keyword(Keyword::OR),
                        "not" => Token::Keyword(Keyword::NOT),
                        "nil" => Token::Keyword(Keyword::NIL),
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        "try" => Token::Keyword(Keyword::TRY),
                        "catch" => Token::Keyword(Keyword::CATCH),
                        "finally" => Token::Keyword(Keyword::FINALLY),
                        _ => Token::Identifier(s.to_string()),
                    }
                }),
            )),
        ))
        .parse_peek(chars)
    }

    /// 代码转成token串
    pub fn tokenizer(code: String) -> Result<Vec<(Token, Location)>, TokenError> {
        let mut input = code.as_str();
        let mut loc = Location::default();
        let mut tokens = vec![];

        loop {
            debug!(?input);
            let start_loc = loc;
            let (remain_input, token) = parse_with_winnow(input).map_err(|e| TokenError::ParseErrorWithLocation {
                msg: e.to_string(),
                line: loc.line,
            })?;

            let consumed_len = input.len() - remain_input.len();
            let consumed_text = &input[..consumed_len];
            loc = advance_location(loc, consumed_text);

            if !matches!(token, Token::Space) {
                tokens.push((token, start_loc));
            }
            if remain_input.is_empty() {
                break;
            }
            input = remain_input
        }

        Ok(tokens)
    }

    fn advance_location(mut loc: Location, consumed_text: &str) -> Location {
        for ch in consumed_text.chars() {
            loc.index += 1;
            if ch == '\n' || ch == '\r' {
                loc.line += 1;
                loc.col = 1;
            } else {
                loc.col += 1;
            }
        }
        loc
    }
}

#[cfg(test)]
mod tests;

mod handwritten {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use super::{Keyword, Location, Operator, Token, TokenError};

    fn parse_token(input: &str, loc: &Location) -> Result<(Token, Location), TokenError> {
        let chars: Vec<char> = input.chars().collect();
        let cur = *chars.get(loc.index).unwrap_or(&' ');
        let next = *chars.get(loc.index + 1).unwrap_or(&' ');
        let third = *chars.get(loc.index + 2).unwrap_or(&' ');
        let res = match cur {
            // -- line comment
            '-' if next == '-' => {
                let mut l = loc.incr2();
                while l.index < chars.len() && chars[l.index] != '\n' {
                    l = l.incr();
                }
                (Token::Space, l)
            }
            '\n' | '\r' => (Token::NewLine, loc.new_line()),
            _ if cur.is_whitespace() => (Token::Space, loc.incr()),
            '{' => (Token::LBig, loc.incr()),
            '}' => (Token::RBig, loc.incr()),
            '[' => (Token::LSquare, loc.incr()),
            ']' => (Token::RSquare, loc.incr()),
            '(' => (Token::LParen, loc.incr()),
            ')' => (Token::RParen, loc.incr()),
            ':' => (Token::Colon, loc.incr()),
            '.' if next == '.' => (Token::Operator(Operator::Concat), loc.incr2()),
            '.' => (Token::Dot, loc.incr()),
            ',' => (Token::COMMA, loc.incr()),
            '+' if next == '=' => (Token::Operator(Operator::AddAssign), loc.incr2()),
            '+' => (Token::Operator(Operator::Add), loc.incr()),
            '-' if next == '=' => (Token::Operator(Operator::SubAssign), loc.incr2()),
            '-' => (Token::Operator(Operator::Subtract), loc.incr()),
            '*' if next == '=' => (Token::Operator(Operator::MulAssign), loc.incr2()),
            '*' => (Token::Operator(Operator::Multiply), loc.incr()),
            '/' if next == '/' && third == '=' => (Token::Operator(Operator::FloorDivAssign), loc.incr_n(3)),
            '/' if next == '/' => (Token::Operator(Operator::FloorDiv), loc.incr2()),
            '/' if next == '=' => (Token::Operator(Operator::DivAssign), loc.incr2()),
            '/' => (Token::Operator(Operator::Divide), loc.incr()),
            '%' if next == '=' => (Token::Operator(Operator::ModAssign), loc.incr2()),
            '%' => (Token::Operator(Operator::Mod), loc.incr()),
            '#' => (Token::Operator(Operator::Len), loc.incr()),
            '~' if next == '=' => (Token::Operator(Operator::NotEquals), loc.incr2()),
            '=' if next == '=' => (Token::Operator(Operator::Equals), loc.incr2()),
            '=' if next != '=' => (Token::Operator(Operator::Assign), loc.incr()),
            '<' if next == '=' => (Token::Operator(Operator::LtE), loc.incr2()),
            '<' if next != '=' => (Token::Operator(Operator::Lt), loc.incr()),
            '>' if next == '=' => (Token::Operator(Operator::GtE), loc.incr2()),
            '>' if next != '=' => (Token::Operator(Operator::Gt), loc.incr()),
            '"' | '\'' => {
                let mut l = loc.incr();
                while l.index < chars.len() && cur != chars[l.index] {
                    l = match chars[l.index] {
                        '\n' => l.new_line(),
                        _ => l.incr(),
                    };
                }
                if l.index >= chars.len() {
                    return Err(TokenError::ParseErrorWithLocation {
                        msg: "Unterminated string".to_string(),
                        line: loc.line,
                    });
                }
                let s: String = chars.as_slice()[(loc.index + 1)..l.index].iter().collect();
                (Token::String(s), l.incr())
            }
            _ if cur.is_numeric() => {
                let mut l = loc.incr();
                let mut has_dot = false;

                while l.index < chars.len() {
                    let c = chars[l.index];
                    if c.is_numeric() {
                        l = l.incr();
                    } else if c == '.' {
                        if has_dot {
                            break; // Second dot, stop
                        }
                        // Lookahead for next digit to ensure it's a float, not method call like 1.toString()
                        // But for simple float like 1.2, next must be digit.
                        // If we have 1. , it is treated as float 1.0 in some langs, but let's see.
                        // Winnow parser: (digit1, literal("."), opt(digit1))
                        let next_char = chars.get(l.index + 1).copied().unwrap_or(' ');
                        if next_char.is_numeric() {
                            has_dot = true;
                            l = l.incr();
                        } else {
                            break; // Dot not followed by digit, likely method call or range
                        }
                    } else {
                        break;
                    }
                }

                let s: String = chars.iter().skip(loc.index).take(l.index - loc.index).collect();

                if has_dot {
                    match Decimal::from_str(&s) {
                        Ok(d) => (Token::Float(d), l),
                        Err(e) => return Err(TokenError::DecimalError(e)),
                    }
                } else {
                    match s.parse::<i32>() {
                        Ok(i) => (Token::Int(i), l),
                        Err(e) => return Err(TokenError::Disconnect(e)),
                    }
                }
            }

            _ if cur.is_ascii_alphabetic() || cur == '_' => {
                let mut l = loc.incr();
                while l.index < chars.len() && (chars[l.index].is_alphanumeric() || chars[l.index] == '_') {
                    l = l.incr();
                }

                let s: String = chars.as_slice()[loc.index..l.index].iter().collect();
                let token = match s.as_str() {
                    "local" => Token::Keyword(Keyword::LOCAL),
                    "function" => Token::Keyword(Keyword::FUNCTION),
                    "return" => Token::Keyword(Keyword::RETURN),
                    "if" => Token::Keyword(Keyword::IF),
                    "then" => Token::Keyword(Keyword::THEN),
                    "else" => Token::Keyword(Keyword::ELSE),
                    "elseif" => Token::Keyword(Keyword::ELSEIF),
                    "end" => Token::Keyword(Keyword::END),
                    "while" => Token::Keyword(Keyword::WHILE),
                    "do" => Token::Keyword(Keyword::DO),
                    "repeat" => Token::Keyword(Keyword::REPEAT),
                    "until" => Token::Keyword(Keyword::UNTIL),
                    "for" => Token::Keyword(Keyword::FOR),
                    "in" => Token::Keyword(Keyword::IN),
                    "break" => Token::Keyword(Keyword::BREAK),
                    "continue" => Token::Keyword(Keyword::CONTINUE),
                    "and" => Token::Keyword(Keyword::AND),
                    "or" => Token::Keyword(Keyword::OR),
                    "not" => Token::Keyword(Keyword::NOT),
                    "nil" => Token::Keyword(Keyword::NIL),
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    "try" => Token::Keyword(Keyword::TRY),
                    "catch" => Token::Keyword(Keyword::CATCH),
                    "finally" => Token::Keyword(Keyword::FINALLY),
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

    /// 手写分词器入口
    pub fn tokenizer(code: String) -> Result<Vec<(Token, Location)>, TokenError> {
        let mut loc = Location::default();
        let mut tokens = vec![];
        let len = code.chars().count(); // Note: this is O(N) for UTF-8

        // Helper to check bounds
        while loc.index < len {
            // debug!("Parsing at loc: {:?}", loc);
            let (token, new_loc) = parse_token(&code, &loc)?;

            if !matches!(token, Token::Space) {
                tokens.push((token, loc));
            }

            loc = new_loc;
        }

        Ok(tokens)
    }

    // Location is defined in the parent module.
}
