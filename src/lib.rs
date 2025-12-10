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

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use std::ops::Range;
use codespan_reporting::term::{emit_into_string, Config};

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

/// 编译器模块
pub mod compiler;
/// 表达式模块
pub mod expression;
/// 统一解析器模块（内部包含手写和 Pest 两种实现）
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
    #[error("Runtime error at line {1}: {0}")]
    Runtime(#[source] value::RuntimeError, u32),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl From<value::RuntimeError> for ChenError {
    fn from(e: value::RuntimeError) -> Self {
        ChenError::Runtime(e, 0)
    }
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
    let ast = parser::parse_from_source(&code)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let mut vm = vm::VM::new();
    let result = vm.execute(&program);
    match result {
        vm::VMResult::Ok(value) => {
            debug!("Execution result: {:?}", value);
        }
        vm::VMResult::Error { error, line, .. } => {
            return Err(ChenError::Runtime(error, line));
        }
    }
    Ok(())
}

/// 运行代码并捕获输出
pub fn run_captured(code: String) -> Result<String, ChenError> {
    let ast = parser::parse_from_source(&code)?;

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
            vm::VMResult::Error { error, line, .. } => {
                return Err(ChenError::Runtime(error, line));
            }
        }

    }

    let output_vec = output.lock().unwrap().clone();
    Ok(String::from_utf8(output_vec)?)
}

fn build_diagnostic(code: &str, file_id: usize, error: &ChenError) -> Diagnostic<usize> {
    match error {
        ChenError::Runtime(err, line) => {
            let range = get_line_range(code, *line);
            Diagnostic::error()
                .with_message(err.to_string())
                .with_labels(vec![
                    Label::primary(file_id, range).with_message("Runtime error occurred here"),
                ])
        }
        _ => Diagnostic::error().with_message(error.to_string()),
    }
}

pub fn report_error(code: &str, filename: &str, error: &ChenError) -> String {
    let mut files = SimpleFiles::new();
    let file_id = files.add(filename, code);
    let diagnostic = build_diagnostic(code, file_id, error);

    let config = Config::default();
    match emit_into_string(&config, &files, &diagnostic) {
        Err(e) => {

        eprintln!("Failed to emit diagnostic: {}", e);
        eprintln!("Original error: {}", error);
            Default::default()
        }
        Ok(s) => {
            s
        }
    }
}


fn get_line_range(code: &str, line: u32) -> Range<usize> {
    if line == 0 {
        return 0..code.len();
    }

    let mut current_line = 1;
    let mut start_byte = 0;

    for line_str in code.split_inclusive('\n') {
        if current_line == line {
            let len = line_str.trim_end().len();
            let end_byte = if len == 0 {
                start_byte + 1
            } else {
                start_byte + len
            };
            let end_byte = std::cmp::min(end_byte, code.len());

            return start_byte..end_byte;
        }
        start_byte += line_str.len();
        current_line += 1;
    }

    let len = code.len();
    if len > 0 {
        len - 1..len
    } else {
        0..0
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_wasm(code: String) -> String {
    match run_captured(code.clone()) {
        Ok(output) => output,
        Err(e) => {
            let error_report = report_error(&code, "<input>", &e);
            format!("Error:\n{}", error_report)
        }
    }
}
