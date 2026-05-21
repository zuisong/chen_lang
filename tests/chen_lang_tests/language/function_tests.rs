use crate::common::run_chen_lang_code;

#[test]
fn test_minimal_test() {
    let code = r#"
function func(){
    return 123
}
let x = 1
x = func()
console.log(x)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "123");
}

#[test]
fn test_simple_test() {
    let code = r#"
function test(){
    console.log("hello")
    return 42
}
let x = 0
x = test()
console.log("done")
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("hello"));
    assert!(output.contains("done"));
}

#[test]
fn test_fibonacci_example() {
    let code = r#"
function fibonacci(n){
    if (n <= 1) {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}
console.log(fibonacci(1))
console.log(fibonacci(2))
console.log(fibonacci(3))
"#;

    let output = run_chen_lang_code(code).unwrap();

    // 验证斐波那契数列的前几个值
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    // assert!(stdout.contains("3")); // fib(3) is 2
}

#[test]
fn test_anonymous_function_variable() {
    let output = run_chen_lang_code(
        r#"
        let add_one = function(x) {
            return x + 1
        }
        console.log(add_one(10))
    "#,
    )
    .expect("failed");
    assert!(output.contains("11"));
}

#[test]
fn test_immediate_invocation() {
    let output = run_chen_lang_code(
        r#"
        let result = function(x, y) {
            return x * y
        } (5, 6)
        console.log(result)
    "#,
    )
    .expect("failed");
    assert!(output.contains("30"));
}

#[test]
fn test_high_order_function() {
    let output = run_chen_lang_code(
        r#"
        function apply(f, val) {
            return f(val)
        }
        
        let res = apply(function(x){ return x * 2 }, 21)
        console.log(res)
    "#,
    )
    .expect("failed");
    assert!(output.contains("42"));
}

#[test]
fn test_implicit_return_add() {
    let code = r#"
    function add(a, b) {
        a + b
    }
    console.log(add(1, 2))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_explicit_return() {
    let code = r#"
    function explicit_return(a) {
        return a * 2
    }
    console.log(explicit_return(10))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn test_empty_function_returns_null() {
    let code = r#"
    function empty() {
    }
    console.log(empty())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "null");
}

#[test]
fn test_statement_end_returns_null() {
    let code = r#"
    function statement_end() {
        let x = 1
    }
    console.log(statement_end())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "null");
}

#[test]
fn test_all_implicit_returns() {
    // Run the complete test file
    let code = r#"
function add(a, b) {
    a + b
}
console.log(add(1, 2))

function explicit_return(a) {
    return a * 2
}
console.log(explicit_return(10))

function empty() {
}
console.log(empty())

function statement_end() {
    let x = 1
}
console.log(statement_end())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].trim(), "3");
    assert_eq!(lines[1].trim(), "20");
    assert_eq!(lines[2].trim(), "null");
    assert_eq!(lines[3].trim(), "null");
}
