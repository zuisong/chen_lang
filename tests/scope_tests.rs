use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;
use std::fs;

/// 创建临时文件并运行chen_lang
fn run_chen_lang_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!();
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, code)?;
    
    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output()?;
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn test_function_scope_isolation() {
    let code = r#"
def func() {
    let local_var = "local_value"
    return "test"
}

let result = "abcd"
func()
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("abcd"));

}

#[test]
fn test_function_variable_not_leaked() {
    let code = r#"
def func() {
    let secret = "should_not_be_visible"
    return "done"
}

func()
print(secret)  // 这应该报错：未定义变量
"#;

    let output = run_chen_lang_code(code).unwrap();
    // The following assertion checks that the variable 'secret' was NOT leaked.
    // Given the current implementation, this test will now FAIL, which is the correct behavior for a test case designed to catch this bug.
    assert!(!output.contains("should_not_be_visible"), "FAILURE: Variable 'secret' was leaked into the global scope and printed.");
}

#[test]
fn test_if_statement_scope() {
    let code = r#"
let x = "global"
if true {
    let x = "local"
    print(x)      // 应该输出"local"
}
print(x)          // 应该输出"global"
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("local"));
    assert!(lines[1].contains("global"));
}

#[test]
fn test_for_loop_scope() {
    let code = r#"
let i = 100
for i <= 3 {
    let temp = i
    print(temp)   // 应该输出1, 2, 3
}
print(i)          // 应该输出100
print(temp)       // 应该报错：未定义变量
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    
    // 当前实现：i会被覆盖为3，temp仍然可见
    // 正确实现：前3行输出1,2,3，第4行输出100，第5行报错
    assert!(lines[0].trim() == "1");
    assert!(lines[1].trim() == "2"); 
    assert!(lines[2].trim() == "3");
    assert!(lines[3].trim() == "100");
}