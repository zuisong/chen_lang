mod common;
use common::run_chen_lang_code;

#[test]
fn test_implicit_return_add() {
    let code = r#"
    def add(a, b) {
        a + b
    }
    println(add(1, 2))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_explicit_return() {
    let code = r#"
    def explicit_return(a) {
        return a * 2
    }
    println(explicit_return(10))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn test_empty_function_returns_null() {
    let code = r#"
    def empty() {
    }
    println(empty())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "null");
}

#[test]
fn test_statement_end_returns_null() {
    let code = r#"
    def statement_end() {
        let x = 1
    }
    println(statement_end())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "null");
}

#[test]
fn test_all_implicit_returns() {
    // Run the complete test file
    let code = r#"
def add(a, b) {
    a + b
}
println(add(1, 2))

def explicit_return(a) {
    return a * 2
}
println(explicit_return(10))

def empty() {
}
println(empty())

def statement_end() {
    let x = 1
}
println(statement_end())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].trim(), "3");
    assert_eq!(lines[1].trim(), "20");
    assert_eq!(lines[2].trim(), "null");
    assert_eq!(lines[3].trim(), "null");
}
