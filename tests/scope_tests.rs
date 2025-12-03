mod common;
use common::run_chen_lang_code;

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
print(secret)  # 这应该报错：未定义变量
"#;

    let result = run_chen_lang_code(code);
    assert!(result.is_err());
}

#[test]
fn test_if_statement_scope() {
    let code = r#"
let x = "global"
if true {
    let x = "local"
    println(x)
}
println(x)
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
let i = 1
for i <= 3 {
    let temp = i
    println(temp)
    i = i + 1
}
println(i)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].trim(), "1");
    assert_eq!(lines[1].trim(), "2");
    assert_eq!(lines[2].trim(), "3");
    assert_eq!(lines[3].trim(), "4");
}
