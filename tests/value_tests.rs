// Value system tests - testing the new unified value system
// including float operations, string operations, and type conversions
use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

#[test]
fn test_integer_arithmetic() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, "let x = 5\nlet y = 3\nprint(x + y)\n").unwrap();

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "8");
}

#[test]
fn test_float_arithmetic() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, "let x = 3.14\nlet y = 2.0\nprint(x * y)\n").unwrap();

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "6.28");
}

#[test]
fn test_string_concatenation() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let hello = \"Hello\"\nlet world = \" World\"\nprint(hello + world)\n",
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
    assert_eq!(stdout.trim(), "Hello World");
}

#[test]
fn test_mixed_type_arithmetic() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let int_val = 5\nlet float_val = 2.5\nlet result = int_val + float_val\nprint(result)\n",
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
    assert_eq!(stdout.trim(), "7.5");
}

#[test]
fn test_float_division() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let x = 7.0\nlet y = 2.0\nlet result = x / y\nprint(result)\n",
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
    assert_eq!(stdout.trim(), "3.5");
}

#[test]
fn test_negative_float() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let x = -3.14\nlet y = 2.0\nlet result = x * y\nprint(result)\n",
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
    assert_eq!(stdout.trim(), "-6.28");
}

#[test]
fn test_variable_assignment_with_float() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let pi = 3.14159\nlet radius = 2.0\nlet area = pi * radius * radius\nprint(area)\n",
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
    assert_eq!(stdout.trim(), "12.56636");
}

#[test]
fn test_string_with_numbers() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let prefix = \"Result: \"\nlet number = 42\nlet message = prefix + \"42\"\nprint(message)\n",
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
    assert_eq!(stdout.trim(), "Result: 42");
}

#[test]
fn test_complex_float_expression() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let a = 1.5\nlet b = 2.0\nlet c = 3.0\nlet result = a + b * c - 0.5\nprint(result)\n",
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
    // 1.5 + 2.0 * 3.0 - 0.5 = 1.5 + 6.0 - 0.5 = 7.0
    assert_eq!(stdout.trim(), "7");
}

#[test]
fn test_zero_float() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let x = 0.0\nlet y = 5.0\nlet result = x + y\nprint(result)\n",
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
    assert_eq!(stdout.trim(), "5");
}

#[test]
fn test_float_comparison() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let a = 3.14\nlet b = 3.14\nlet equal = a == b\nprint(equal)\n",
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
    assert_eq!(stdout.trim(), "true");
}

#[test]
fn test_mixed_comparison() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        "let int_val = 5\nlet float_val = 5.0\nlet equal = int_val == float_val\nprint(equal)\n",
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
    assert_eq!(stdout.trim(), "true");
}
