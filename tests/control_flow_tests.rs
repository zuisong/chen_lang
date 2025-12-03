use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

#[test]
fn test_simple_for_loop() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let i = 0
for i <= 2 {
    print(i)
    i = i + 1
}
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
    assert!(stdout.contains("0"));
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
}

#[test]
fn test_simple_if_statement() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let a = 5
let b = 3
if a > b {
    print(1)
}
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
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn test_if_else_example() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let i = 0
for i <= 99 {
    if i%2 == 0 {
        println(i + " 是偶数 ")
    } else {
        println(i + " 是奇数 ")
    }
    i = i + 1
}
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

    // 验证包含偶数和奇数的输出
    assert!(stdout.contains("0 是偶数"));
    assert!(stdout.contains("1 是奇数"));
    assert!(stdout.contains("98 是偶数"));
    assert!(stdout.contains("99 是奇数"));
}
