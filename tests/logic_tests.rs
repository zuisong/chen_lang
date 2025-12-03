use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

#[test]
fn test_boolean_operations() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let a = 1
let b = 0
let result = a && b
print(result)
let result2 = a || b
print(result2)
"#,
    )
    .unwrap();

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("false")); // a && b = false
    assert!(stdout.contains("true")); // a || b = true
}

#[test]
fn test_comparison_operations() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let a = 5
let b = 3
let result = a > b
print(result)
let result2 = a == b
print(result2)
let result3 = a <= b
print(result3)
"#,
    )
    .unwrap();

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("true")); // a > b = true
    assert!(stdout.contains("false")); // a == b = false
    assert!(stdout.contains("false")); // a <= b = false
}
