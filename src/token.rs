#[allow(unused_imports)]
use log::*;

#[derive(Debug, PartialEq)]
pub enum Keyword {
    INT,
    IF,
    FOR,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum StdFunction {
    Println,
    Print,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Operator(Operator),
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
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

pub fn tokenlizer(code: String) -> Result<Vec<Token>, failure::Error> {
    let chars: Vec<_> = code.chars().collect();

    let mut tokens = vec![];

    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '#' {
            while chars[i] != '\r' && chars[i] != '\n' {
                i += 1;
            }
            i = i + 1;
            continue;
        }

        if chars[i] == '\r' || chars[i] == '\n' {
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
        if chars[i] == '&' && chars[i + 1] == '&' {
            tokens.push(Token::Operator(Operator::And));
            i += 2;
            continue;
        }
        if chars[i] == '|' && chars[i + 1] == '|' {
            tokens.push(Token::Operator(Operator::Or));
            i += 2;
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

        if chars[i] == '"' || chars[i] == '\'' {
            let mut j = i + 1;

            while chars[i] != chars[j] {
                j += 1;
            }
            let s: String = chars.as_slice()[(i + 1)..j].iter().collect();
            tokens.push(Token::String(s));

            i = j + 1;
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

        if chars[i] == '-' && !chars[i + 1].is_numeric() {
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
                tokens.push(Token::Float(s.parse()?));
            } else {
                tokens.push(Token::Int(s.parse()?));
            }

            i = j;
            continue;
        }

        if chars[i].is_ascii_alphabetic() {
            let mut j = i + 1;

            while chars[j].is_ascii_alphabetic() {
                j += 1;
            }

            let s: String = chars.as_slice()[i..j].iter().collect();

            match s.as_str() {
                "println" => {
                    tokens.push(Token::StdFunction(StdFunction::Println));
                }
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
                "true" => {
                    tokens.push(Token::Bool(true));
                }
                "false" => {
                    tokens.push(Token::Bool(false));
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
