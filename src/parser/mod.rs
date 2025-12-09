//! 统一的解析器模块
//! 
//! 本模块提供统一的解析器接口，内部包含两个解析器实现：
//! - `handwritten`: 手写递归下降解析器
//! - `pest_impl`: 基于 Pest 的解析器（通过 pest-parser feature 启用）
//! 
//! 根据 feature flags 自动选择使用哪个解析器实现。

use crate::expression::Ast;
use crate::token::*;
use thiserror::Error;

// 私有子模块 - 手写解析器
#[cfg(not(feature = "pest-parser"))]
mod handwritten;

// 私有子模块 - Pest 解析器
#[cfg(feature = "pest-parser")]
mod pest_impl;

// Re-export Rule type for use in error types
#[cfg(feature = "pest-parser")]
pub use pest_impl::Rule;

/// 统一的解析器错误类型
#[derive(Error, Debug)]
pub enum ParserError {
    /// 词法分析错误
    #[error(transparent)]
    Token(#[from] TokenError),
    
    /// 手写解析器错误
    #[cfg(not(feature = "pest-parser"))]
    #[error(transparent)]
    Handwritten(#[from] handwritten::ParseError),
    
    /// Pest 解析器错误
    #[cfg(feature = "pest-parser")]
    #[error(transparent)]
    Pest(#[from] Box<pest::error::Error<pest_impl::Rule>>),
}

/// 从源代码字符串解析 AST（统一接口）
/// 
/// 这是推荐使用的解析入口函数，它提供了统一的接口，
/// 无论使用哪种解析器实现都接受源代码字符串作为输入。
/// 
/// # 参数
/// - `code`: 源代码字符串
/// 
/// # 返回
/// - `Ok(Ast)`: 解析成功
/// - `Err(ParserError)`: 解析失败
/// 
/// # 示例
/// ```ignore
/// use chen_lang::parser;
/// 
/// let code = "let x = 10";
/// let ast = parser::parse_from_source(code)?;
/// ```
#[cfg(not(feature = "pest-parser"))]
pub fn parse_from_source(code: &str) -> Result<Ast, ParserError> {
    let tokens = tokenlizer(code.to_string())?;
    handwritten::parse(tokens).map_err(Into::into)
}

/// 从源代码字符串解析 AST（Pest 版本）
#[cfg(feature = "pest-parser")]
pub fn parse_from_source(code: &str) -> Result<Ast, ParserError> {
    pest_impl::parse(code).map_err(Into::into)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_parser() {
        let code = "let x = 10";
        let ast = parse_from_source(code);
        assert!(ast.is_ok());
    }
}
