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
        let new_len = arr:push(30)
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
        let val = arr:pop()
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
        println(arr:len())
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

#[test]
fn test_array_like_object_creation() {
    let code = r#"
        let arr = ${ 
            0: "first",
            1: "second",
            2: "third"
        }
        println(arr[0])
        println(arr[1])
        println(arr[2])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("first"));
    assert!(output.contains("second"));
    assert!(output.contains("third"));
}

#[test]
fn test_array_like_index_access() {
    let code = r#"
        let arr = ${ 0: 10, 1: 20, 2: 30 }
        let sum = arr[0] + arr[1] + arr[2]
        println(sum)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("60"));
}

#[test]
fn test_array_like_index_assignment() {
    let code = r#"
        let arr = ${ 0: 1, 1: 2, 2: 3 }
        arr[0] = 100
        arr[1] = 200
        println(arr[0])
        println(arr[1])
        println(arr[2])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
    assert!(output.contains("200"));
    assert!(output.contains("3"));
}

#[test]
fn test_array_like_dynamic_indexing() {
    let code = r#"
        let arr = ${ 0: "a", 1: "b", 2: "c" }
        let i = 0
        for i < 3 {
            println(arr[i])
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
        # 稀疏数组：只有索引 0 和 100
        let sparse = ${ 0: "start", 100: "end" }
        println(sparse[0])
        println(sparse[100])
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("start"));
    assert!(output.contains("end"));
}

#[test]
fn test_array_like_mixed_keys() {
    let code = r#"
        # 混合使用数字键和字符串键
        let mixed = ${ 
            0: "first element",
            1: "second element",
            name: "my array",
            length: 2
        }
        println(mixed[0])
        println(mixed[1])
        println(mixed.name)
        println(mixed.length)
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
        # 嵌套数组（二维数组）
        let matrix = ${ 
            0: ${ 0: 1, 1: 2 },
            1: ${ 0: 3, 1: 4 }
        }
        println(matrix[0][0])
        println(matrix[0][1])
        println(matrix[1][0])
        println(matrix[1][1])
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
        # 模拟数组迭代
        let arr = ${ 0: 10, 1: 20, 2: 30, 3: 40 }
        let sum = 0
        let i = 0
        for i < 4 {
            sum = sum + arr[i]
            i = i + 1
        }
        println(sum)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("100"));
}

#[test]
fn test_array_like_with_strings() {
    let code = r#"
        let names = ${ 
            0: "Alice",
            1: "Bob",
            2: "Charlie"
        }
        let greeting = "Hello, " + names[0] + "!"
        println(greeting)
    "#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Hello, Alice!"));
}
