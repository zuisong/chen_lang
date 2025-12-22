use crate::common::run_chen_lang_code;

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
#[test]
fn test_array_push() {
    // Requires method call optimization because push is native method on proto
    let code = r#"
        let arr = [10, 20]
        let new_len = arr.push(30)
        println(new_len)
        println(arr[2])
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("3"));
    assert!(output.contains("30"));
}

#[test]
fn test_array_pop() {
    let code = r#"
        let arr = [10, 20]
        let val = arr.pop()
        println(val)
        let removed = arr[1] 
        # Accessing "1" should be null.
        if removed == null {
            println("Removed")
        }
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("20"));
    assert!(output.contains("Removed"));
}

#[test]
fn test_array_len() {
    let code = r#"
        let arr = [1, 2, 300]
        println(arr.len())
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("3"));
}

#[test]
fn test_array_type_tag() {
    let code = r#"
        let arr = []
        println(arr.__type)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Array"));
}
