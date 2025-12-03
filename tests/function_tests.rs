use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

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

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

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

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("done"));
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

    let output = cmd
        .arg("run")
        .arg(&test_file)
        .env("RUST_LOG", "off")
        .output()
        .unwrap();

    if !output.status.success() {
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证斐波那契数列的前几个值
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
    // assert!(stdout.contains("3")); // fib(3) is 2
}
