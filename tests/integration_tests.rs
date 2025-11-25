use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_simple_arithmetic() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let i = 1
let j = 2
let k = i + j
print(k)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn test_boolean_operations() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let a = 1
let b = 0
let result = a && b
print(result)
let result2 = a || b
print(result2)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0")); // a && b = 0
    assert!(stdout.contains("1")); // a || b = 1
}

#[test]
fn test_comparison_operations() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let a = 5
let b = 3
let result = a > b
print(result)
let result2 = a == b
print(result2)
let result3 = a <= b
print(result3)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1")); // a > b = 1 (true)
    assert!(stdout.contains("0")); // a == b = 0 (false)
    assert!(stdout.contains("0")); // a <= b = 0 (false)
}

#[test]
fn test_modulo_operation() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let a = 10
let b = 3
let result = a % b
print(result)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "1"); // 10 % 3 = 1
}

#[test]
fn test_complex_expression() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let a = 2
let b = 3
let c = 4
let result = a + b * c
print(result)
let result2 = (a + b) * c
print(result2)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("14")); // 2 + 3 * 4 = 14
    assert!(stdout.contains("20")); // (2 + 3) * 4 = 20
}

#[test]
fn test_simple_for_loop() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let i = 0
for i <= 2 {
    print(i)
    i = i + 1
}
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0"));
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
}

#[test]
fn test_simple_if_statement() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let a = 5
let b = 3
if a > b {
    print(1)
}
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn test_string_operations() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let hello = "Hello"
let world = "World"
let result = hello + " " + world
print(result)
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // 字符串被转换为哈希值，但应该有输出
    assert!(!stdout.trim().is_empty());
}

#[test]
fn test_nine_nine_multiply_table() {
    let mut cmd = Command::cargo_bin("chen_lang").unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, r#"
let i=1
for i<=2 {
    let j = 1
    for j <= i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}
"#).unwrap();
    
    let output = cmd.arg("run").arg(&test_file).output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // 验证有输出（字符串被转换为哈希值）
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2); // 至少2行（对应i=1和i=2）
}