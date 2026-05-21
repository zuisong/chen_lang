use crate::common::run_chen_lang_code;

#[test]
fn test_array_creation() {
    let code = r#"
        let arr = [10, 20, 30]
        console.log(arr[0])
        console.log(arr[1])
        console.log(arr[2])
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
        console.log(arr[0])
        arr[1] = 50
        console.log(arr[1])
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
        console.log(arr[10]) 
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("sparse"));
}

#[test]
fn test_mixed_array() {
    let code = r#"
        let arr = [1, "two", true]
        console.log(arr[0])
        console.log(arr[1])
        console.log(arr[2])
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("1"));
    assert!(output.contains("two"));
    assert!(output.contains("true"));
}
#[test]
fn test_array_push() {
    let code = r#"
        let arr = [10, 20]
        let new_len = arr.push(30)
        console.log(new_len)
        console.log(arr[2])
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
        console.log(val)
        let removed = arr[1] 
        if (removed == null) {
            console.log("Removed")
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
        console.log(arr.length)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("3"));
}

#[test]
fn test_array_type_tag() {
    let code = r#"
        let arr = []
        console.log(arr.__type)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Array"));
}

#[test]
fn test_array_like_object_creation() {
    let code = r#"
        let arr = { 
            0: "first",
            1: "second",
            2: "third"
        }
        console.log(arr[0])
        console.log(arr[1])
        console.log(arr[2])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("first"));
    assert!(output.contains("second"));
    assert!(output.contains("third"));
}

#[test]
fn test_array_like_index_access() {
    let code = r#"
        let arr = { 0: 10, 1: 20, 2: 30 }
        let sum = arr[0] + arr[1] + arr[2]
        console.log(sum)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("60"));
}

#[test]
fn test_array_like_index_assignment() {
    let code = r#"
        let arr = { 0: 1, 1: 2, 2: 3 }
        arr[0] = 100
        arr[1] = 200
        console.log(arr[0])
        console.log(arr[1])
        console.log(arr[2])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
    assert!(output.contains("200"));
    assert!(output.contains("3"));
}

#[test]
fn test_array_like_dynamic_indexing() {
    let code = r#"
        let arr = { 0: "a", 1: "b", 2: "c" }
        let i = 0
        while (i < 3) {
            console.log(arr[i])
            i = i + 1
        }
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("a"));
    assert!(output.contains("b"));
    assert!(output.contains("c"));
}

#[test]
fn test_array_like_sparse_array() {
    let code = r#"
        let sparse = { 0: "start", 100: "end" }
        console.log(sparse[0])
        console.log(sparse[100])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("start"));
    assert!(output.contains("end"));
}

#[test]
fn test_array_like_mixed_keys() {
    let code = r#"
        let mixed = { 
            0: "first element",
            1: "second element",
            name: "my array",
            length: 2
        }
        console.log(mixed[0])
        console.log(mixed[1])
        console.log(mixed.name)
        console.log(mixed.length)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("first element"));
    assert!(output.contains("second element"));
    assert!(output.contains("my array"));
    assert!(output.contains("2"));
}

#[test]
fn test_array_like_nested() {
    let code = r#"
        let matrix = { 
            0: { 0: 1, 1: 2 },
            1: { 0: 3, 1: 4 }
        }
        console.log(matrix[0][0])
        console.log(matrix[0][1])
        console.log(matrix[1][0])
        console.log(matrix[1][1])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("3"));
    assert!(output.contains("4"));
}

#[test]
fn test_array_like_iteration() {
    let code = r#"
        let arr = { 0: 10, 1: 20, 2: 30, 3: 40 }
        let sum = 0
        let i = 0
        while (i < 4) {
            sum = sum + arr[i]
            i = i + 1
        }
        console.log(sum)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
}

#[test]
fn test_array_like_with_strings() {
    let code = r#"
        let names = { 
            0: "Alice",
            1: "Bob",
            2: "Charlie"
        }
        let greeting = "Hello, " + names[0] + "!"
        console.log(greeting)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Hello, Alice!"));
}
