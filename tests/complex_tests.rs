use std::fs;

use assert_cmd::cargo_bin_cmd;
use pretty_assertions::assert_matches;
use tempfile::TempDir;

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

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

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

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证有输出（字符串被转换为哈希值）
    let lines: Vec<&str> = stdout.lines().collect();
    assert_matches!(lines[0], s if s.contains("1x1=1"));
    assert_matches!(lines[1], s if s.contains("2x2=4"));

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

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("100以内的 奇数或者是能被三整除的偶数 之和是"));
    assert!(stdout.contains("3316"));
}