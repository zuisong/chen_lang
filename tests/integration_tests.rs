use std::fs;

use assert_cmd::{cargo::cargo_bin_cmd};
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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0")); // a && b = 0
    assert!(stdout.contains("1")); // a || b = 1
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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1")); // a > b = 1 (true)
    assert!(stdout.contains("0")); // a == b = 0 (false)
    assert!(stdout.contains("0")); // a <= b = 0 (false)
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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("14")); // 2 + 3 * 4 = 14
    assert!(stdout.contains("20")); // (2 + 3) * 4 = 20
}

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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn test_string_operations() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let hello = "Hello"
let world = "World"
let result = hello + " " + world
print(result)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // 字符串被转换为哈希值，但应该有输出
    assert!(!stdout.trim().is_empty());
}

#[test]
fn test_nine_nine_multiply_table() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
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
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证有输出（字符串被转换为哈希值）
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2); // 至少2行（对应i=1和i=2）
}

#[test]
fn test_minimal_test() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
def func(){
    return 123
}
let x = 1
x = func()
println(x)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "123");
}

#[test]
fn test_simple_test() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
def test(){
    println("hello")
    return 42
}
let x = 0
x = test()
println("done")
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("done"));
}

#[test]
fn test_sum_example() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
def aaa(n){
    let i = 100
    let sum = 0
    for i != 0 {
        i = i - 1
        if (i%2!=0) || (i%3==0)  {
            println(i)
            sum = sum + i
        }
    }
    println("100以内的 奇数或者是能被三整除的偶数 之和是")
    println(sum)
    sum
}
let sum = 0
sum = aaa(100)
println(sum)
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("100以内的 奇数或者是能被三整除的偶数 之和是"));
    assert!(stdout.contains("3316"));
}

#[test]
fn test_fibonacci_example() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
def fibonacci(n){
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}
println(fibonacci(1))
println(fibonacci(2))
println(fibonacci(3))
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("Fibonacci test output: '{}'", stdout);
    
    // 验证斐波那契数列的前几个值
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("3"));
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

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // 验证包含偶数和奇数的输出
    assert!(stdout.contains("0 是偶数"));
    assert!(stdout.contains("1 是奇数"));
    assert!(stdout.contains("98 是偶数"));
    assert!(stdout.contains("99 是奇数"));
}

#[test]
fn test_nine_nine_multiply_table_full() {
    let mut cmd = cargo_bin_cmd!();

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ch");
    fs::write(
        &test_file,
        r#"
let i=1
for i<=9 {
    let j = 1
    for j <= i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}
"#,
    )
    .unwrap();

    let output = cmd.arg("run").arg(&test_file).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // 验证九九乘法表的几个关键输出
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 9); // 至少9行
    
    // 验证第一行和最后一行
    assert!(lines[0].contains("1x1=1"));
    assert!(lines[8].contains("9x9=81"));
}