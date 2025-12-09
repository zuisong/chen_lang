//! 统一的解析器接口
//! 
//! 根据 feature flags 自动选择使用哪个解析器：
//! - 默认：手写解析器（parse.rs）
//! - pest-parser feature：Pest 解析器（parse_pest.rs）

use crate::expression::Ast;
use crate::token::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[cfg(not(feature = "pest-parser"))]
    #[error(transparent)]
    Handwritten(#[from] crate::parse::ParseError),
    
    #[cfg(feature = "pest-parser")]
    #[error(transparent)]
    Pest(#[from] Box<pest::error::Error<crate::parse_pest::Rule>>),
}

/// 统一的解析函数
/// 
/// 默认使用手写解析器，如果启用了 pest-parser feature 则使用 Pest
#[cfg(not(feature = "pest-parser"))]
pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParserError> {
    Ok(crate::parse::parse(tokens)?)
}

#[cfg(feature = "pest-parser")]
pub fn parse(code: &str) -> Result<Ast, ParserError> {
    Ok(crate::parse_pest::parse(code)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_parser() {
        #[cfg(not(feature = "pest-parser"))]
        {
            let code = "let x = 10";
            let tokens = tokenlizer(code.to_string()).unwrap();
            let ast = parse(tokens);
            assert!(ast.is_ok());
        }

        #[cfg(feature = "pest-parser")]
        {
            let code = "let x = 10";
            let ast = parse(code);
            assert!(ast.is_ok());
        }
    }
}
