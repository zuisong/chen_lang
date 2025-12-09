mod common;
use common::run_chen_lang_code;

#[test]
fn test_array_creation() {
    let code = r#"
        let arr = [10, 20, 30]
        println(arr[0])
        println(arr[1])
        println(arr[2])
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
}

#[test]
fn test_array_indexing() {
    let code = r#"
        let arr = [10, 20]
        println(arr[0])
        arr[1] = 50
        println(arr[1])
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("10"));
    assert!(output.contains("50"));
}

#[test]
fn test_sparse_array() {
    let code = r#"
        let arr = [1]
        arr[10] = "sparse"
        println(arr[10]) 
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("sparse"));
}

#[test]
fn test_mixed_array() {
    let code = r#"
        let arr = [1, "two", true]
        println(arr[0])
        println(arr[1])
        println(arr[2])
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("1"));
    assert!(output.contains("two"));
    assert!(output.contains("true"));
}
