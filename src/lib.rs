//! 一个小的玩具语言
#![allow(soft_unstable)]
// #![deny(missing_docs)]
// #![deny(unused_imports)]
// #![deny(unused_parens)]
// #![deny(dead_code)]
// #![deny(unused_mut)]
// #![deny(unreachable_code)]

use std::io::Write;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{Config, emit_into_string};
use thiserror::Error;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

use crate::vm::RuntimeErrorWithContext;

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
pub mod tokenizer;
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
    Token(#[from] tokenizer::TokenError),
    #[error(transparent)]
    Parser(#[from] parser::ParserError),
    #[error("Runtime error")]
    Runtime(#[from] RuntimeErrorWithContext), // Changed to VMRuntimeError
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[test]
fn test_run_captured() {
    let code = r#"let io = import "stdlib/io"
    io.print("Hello World")"#;
    let output = run_captured(code.to_string()).unwrap();
    assert_eq!(output, "Hello World");
}

/// 运行代码
#[unsafe(no_mangle)]
pub fn run(code: String) -> Result<(), ChenError> {
    let ast = parser::parse_from_source(&code)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let mut vm = vm::VM::new();
    let _result = vm.execute(&program)?;
    // debug!("Execution result: {:?}", _result); // _result is unit from Ok(())
    Ok(())
}

/// 运行代码并捕获输出
pub fn run_captured(code: String) -> Result<String, ChenError> {
    run_captured_with_vm_setup(code, |_| {})
}

/// 运行代码并捕获输出，允许配置 VM
pub fn run_captured_with_vm_setup<F>(code: String, setup: F) -> Result<String, ChenError>
where
    F: FnOnce(&mut vm::VM),
{
    let ast = parser::parse_from_source(&code)?;

    let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

    let output = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter(output.clone());

    {
        let mut vm = vm::VM::with_writer(Box::new(writer));
        setup(&mut vm);
        let _result = vm.execute(&program)?;
        // debug!("Execution result: {:?}", _result);
    } // writer goes out of scope here, flushing

    let output_vec = output.lock().unwrap().clone();
    Ok(String::from_utf8(output_vec)?)
}

fn build_diagnostic(code: &str, file_id: usize, error: &ChenError) -> Diagnostic<usize> {
    match error {
        ChenError::Runtime(err) => {
            let range = get_line_range(code, err.line);
            Diagnostic::error().with_message(err.to_string()).with_labels(vec![
                Label::primary(file_id, range).with_message("Runtime error occurred here"),
            ])
        }
        ChenError::Parser(parser::ParserError::Handwritten(err)) => match err {
            parser::handwritten::ParseError::Message { msg, line } => {
                let range = get_line_range(code, *line);
                Diagnostic::error()
                    .with_message(msg)
                    .with_labels(vec![Label::primary(file_id, range).with_message("Parse error here")])
            }
            parser::handwritten::ParseError::UnexpectedToken { token, line } => {
                let range = get_line_range(code, *line);
                Diagnostic::error()
                    .with_message(format!("Unexpected token: {:?}", token))
                    .with_labels(vec![Label::primary(file_id, range).with_message("Unexpected token")])
            }
            _ => Diagnostic::error().with_message(error.to_string()),
        },
        ChenError::Token(tokenizer::TokenError::ParseErrorWithLocation { msg, line }) => {
            let range = get_line_range(code, *line);
            Diagnostic::error()
                .with_message(msg)
                .with_labels(vec![Label::primary(file_id, range).with_message("Token error")])
        }
        ChenError::Parser(parser::ParserError::Token(tokenizer::TokenError::ParseErrorWithLocation { msg, line })) => {
            let range = get_line_range(code, *line);
            Diagnostic::error()
                .with_message(msg)
                .with_labels(vec![Label::primary(file_id, range).with_message("Token error")])
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
        Ok(s) => s,
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
            let end_byte = if len == 0 { start_byte + 1 } else { start_byte + len };
            let end_byte = std::cmp::min(end_byte, code.len());

            return start_byte..end_byte;
        }
        start_byte += line_str.len();
        current_line += 1;
    }

    let len = code.len();
    if len > 0 { len - 1..len } else { 0..0 }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
#[wasm_bindgen]
pub async fn run_wasm(code: String) -> String {
    use std::rc::Rc;

    let result = async {
        let ast = parser::parse_from_source(&code).map_err(ChenError::Parser)?;

        let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

        let output = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedWriter(output.clone());

        {
            let mut vm = vm::VM::with_writer(Box::new(writer));
            // Execute async to handle HTTP requests and other async operations
            let _result = vm.execute_async(Rc::new(program)).await?;
        }

        let output_vec = output.lock().unwrap().clone();
        Ok::<String, ChenError>(String::from_utf8(output_vec)?)
    }
    .await;

    match result {
        Ok(output) => output,
        Err(e) => {
            let error_report = report_error(&code, "<input>", &e);
            format!("Error:\n{}", error_report)
        }
    }
}
