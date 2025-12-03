mod common;
use common::run_chen_lang_code;

#[test]
fn test_simple_for_loop() {
    let code = r#"
let i = 0
for i <= 2 {
    print(i)
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_simple_if_statement() {
    let code = r#"
let a = 5
let b = 3
if a > b {
    print(1)
}
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_if_else_example() {
    let code = r#"
let i = 0
for i <= 99 {
    if i%2 == 0 {
        println(i + " 是偶数 ")
    } else {
        println(i + " 是奇数 ")
    }
    i = i + 1
}
"#;

    let output = run_chen_lang_code(code).unwrap();

    // 验证包含偶数和奇数的输出
    assert!(output.contains("0 是偶数"));
    assert!(output.contains("1 是奇数"));
    assert!(output.contains("98 是偶数"));
    assert!(output.contains("99 是奇数"));
}
