use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

#[test]
fn test_simple_arithmetic() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let i = 1
let j = 2
let k = i + j
print(k)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn test_modulo_operation() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let a = 10
let b = 3
let result = a % b
print(result)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "1"); // 10 % 3 = 1
}

#[test]
fn test_complex_expression() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let a = 2
let b = 3
let c = 4
let result = a + b * c
print(result)
let result2 = (a + b) * c
print(result2)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("14")); // 2 + 3 * 4 = 14
    assert!(stdout.contains("20")); // (2 + 3) * 4 = 20
}