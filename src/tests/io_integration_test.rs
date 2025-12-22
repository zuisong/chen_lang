use crate::*;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_io_integration() {
    // 使用 tempfile 创建临时文件
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Hello from Chen VM!").unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("IO token created")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to run IO code: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("IO token created"), "Expected output not found: {}", output);
}

#[test]
fn test_io_token_helpers() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Test content").unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("Testing io_token helpers")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to run IO token helpers code: {:?}", result.err());
}

#[test]
fn test_io_objects_exist() {
    let code = r#"
        print(io)
        print(io_token)
    "#;
    
    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "Failed to check IO objects: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("{{"), "Expected object output");
}

#[test]
fn test_io_read_file_basic() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Chen language is awesome!").unwrap();
    temp_file.flush().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("Token created")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to create read token: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("Token created"), "Expected token creation message");
}

#[test]
fn test_io_write_file_basic() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.write_file("{}", "Hello Chen!")
        print("Write token created")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to create write token: {:?}", result.err());
}

#[test]
fn test_io_token_is_completed() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "test").unwrap();
    temp_file.flush().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        let completed = io_token.is_completed(token)
        print("Completed: ")
        print(completed)
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to check completion: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("Completed:"), "Expected completion status");
}

#[test]
fn test_multiple_io_operations() {
    let mut temp_file1 = NamedTempFile::new().unwrap();
    let mut temp_file2 = NamedTempFile::new().unwrap();
    write!(temp_file1, "File 1 content").unwrap();
    write!(temp_file2, "File 2 content").unwrap();
    temp_file1.flush().unwrap();
    temp_file2.flush().unwrap();
    
    let path1 = temp_file1.path().to_str().unwrap();
    let path2 = temp_file2.path().to_str().unwrap();
    
    let code = format!(r#"
        let token1 = io.read_file("{}")
        let token2 = io.read_file("{}")
        print("Created 2 tokens")
    "#, path1, path2);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to create multiple tokens: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("Created 2 tokens"), "Expected success message");
}

#[test]
fn test_io_with_coroutine() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Coroutine test").unwrap();
    temp_file.flush().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        def read_async(path) {{
            let token = io.read_file(path)
            coroutine.yield(token)
            return "done"
        }}
        
        let co = coroutine.create(read_async, "{}")
        let result = coroutine.resume(co)
        print("Coroutine result: ")
        print(result)
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to use IO with coroutine: {:?}", result.err());
}

#[test]
fn test_scheduler_object_exists() {
    let code = r#"
        print(scheduler)
        print("Scheduler exists")
    "#;
    
    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "Failed to check scheduler object: {:?}", result.err());
    
    let output = result.unwrap();
    assert!(output.contains("Scheduler exists"), "Expected scheduler confirmation");
}

#[test]
fn test_io_error_handling() {
    // 尝试读取不存在的文件
    let code = r#"
        let token = io.read_file("/nonexistent/path/file.txt")
        print("Token created for nonexistent file")
    "#;
    
    let result = run_captured(code.to_string());
    // 应该能创建 token，但稍后会失败
    assert!(result.is_ok(), "Should be able to create token for nonexistent file");
}

#[test]
fn test_io_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("Reading empty file")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to read empty file: {:?}", result.err());
}

#[test]
fn test_io_large_content() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let large_content = "x".repeat(10000);
    write!(temp_file, "{}", large_content).unwrap();
    temp_file.flush().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("Reading large file")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to read large file: {:?}", result.err());
}

#[test]
fn test_io_special_characters() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "Special chars: 你好世界 🎉 \n\t\r").unwrap();
    temp_file.flush().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    let code = format!(r#"
        let token = io.read_file("{}")
        print("Reading file with special chars")
    "#, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed to read file with special chars: {:?}", result.err());
}

#[test]
fn test_io_write_then_read() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap().to_string();
    
    let code = format!(r#"
        let write_token = io.write_file("{}", "Test content")
        print("Write initiated")
        
        let read_token = io.read_file("{}")
        print("Read initiated")
    "#, temp_path, temp_path);
    
    let result = run_captured(code);
    assert!(result.is_ok(), "Failed write-then-read: {:?}", result.err());
}

#[test]
fn test_coroutine_with_multiple_yields() {
    let code = r#"
        def multi_yield() {
            coroutine.yield(1)
            coroutine.yield(2)
            coroutine.yield(3)
            return 4
        }
        
        let co = coroutine.create(multi_yield)
        let r1 = coroutine.resume(co)
        let r2 = coroutine.resume(co)
        let r3 = coroutine.resume(co)
        let r4 = coroutine.resume(co)
        
        print("Results: ")
        print(r1)
        print(r2)
        print(r3)
        print(r4)
    "#;
    
    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "Failed multi-yield test: {:?}", result.err());
}

#[test]
fn test_nested_coroutines() {
    let code = r#"
        def inner() {
            coroutine.yield("inner")
            return "inner done"
        }
        
        def outer() {
            let co = coroutine.create(inner)
            let result = coroutine.resume(co)
            coroutine.yield(result)
            return "outer done"
        }
        
        let co = coroutine.create(outer)
        let r1 = coroutine.resume(co)
        print("Nested result: ")
        print(r1)
    "#;
    
    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "Failed nested coroutines: {:?}", result.err());
}
