//! 一个小的玩具语言
#![allow(soft_unstable)]
// #![deny(missing_docs)]
// #![deny(unused_imports)]
// #![deny(unused_parens)]
// #![deny(dead_code)]
// #![deny(unused_mut)]
// #![deny(unreachable_code)]

use std::io::Write;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use tracing::debug;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

use crate::expression::*;
use crate::token::*;

/// 编译器模块
pub mod compiler;
/// 表达式模块
pub mod expression;
/// 手写语法分析模块
pub mod parse;
/// Pest 解析模块 (可选，通过 pest-parser feature 启用)
#[cfg(feature = "pest-parser")]
pub mod parse_pest;
/// 统一解析器接口（内部根据 feature 选择实现）
pub mod parser;
/// 词法分析模块
pub mod token;
/// 值系统模块
pub mod value;
/// 虚拟机模块
pub mod vm;

/// 测试模块
#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum ChenError {
    #[error(transparent)]
    Token(#[from] token::TokenError),
    #[error(transparent)]
    Parser(#[from] parser::ParserError),
    #[error(transparent)]
    Runtime(#[from] value::RuntimeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[test]
fn test_run_captured() {
    let code = r#"print("Hello World")"#;
    let output = run_captured(code.to_string()).unwrap();
    assert_eq!(output, "Hello World");
}

/// 运行代码
#[unsafe(no_mangle)]
pub fn run(code: String) -> Result<(), ChenError> {
    #[cfg(not(feature = "pest-parser"))]
    let ast = {
        let tokens = tokenlizer(code.clone())?;
        parser::parse(tokens)?
    };
    
    #[cfg(feature = "pest-parser")]
    let ast = parser::parse(&code)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let mut vm = vm::VM::new();
    let result = vm.execute(&program);
    match result {
        vm::VMResult::Ok(value) => {
            debug!("Execution result: {:?}", value);
        }
        vm::VMResult::Error(error) => {
            eprintln!("Runtime error: {:?}", error);
        }
    }
    Ok(())
}

/// 运行代码并捕获输出
pub fn run_captured(code: String) -> Result<String, ChenError> {
    #[cfg(not(feature = "pest-parser"))]
    let ast = {
        let tokens = tokenlizer(code.clone())?;
        parser::parse(tokens)?
    };
    
    #[cfg(feature = "pest-parser")]
    let ast = parser::parse(&code)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let output = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter(output.clone());

    {
        let mut vm = vm::VM::with_writer(Box::new(writer));
        let result = vm.execute(&program);
        match result {
            vm::VMResult::Ok(value) => {
                debug!("Execution result: {:?}", value);
            }
            vm::VMResult::Error(error) => {
                let mut guard = output.lock().unwrap();
                writeln!(guard, "Runtime error: {:?}", error)?;
            }
        }
    }

    let output_vec = output.lock().unwrap().clone();
    Ok(String::from_utf8(output_vec)?)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_wasm(code: String) -> String {
    match run_captured(code) {
        Ok(output) => output,
        Err(e) => format!("Error: {}", e),
    }
}
