use crate::common::run_chen_lang_code;

#[test]
fn test_minimal_test() {
    let code = r#"
def func(){
    return 123
}
let x = 1
x = func()
println(x)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "123");
}

#[test]
fn test_simple_test() {
    let code = r#"
def test(){
    println("hello")
    return 42
}
let x = 0
x = test()
println("done")
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("hello"));
    assert!(output.contains("done"));
}

#[test]
fn test_fibonacci_example() {
    let code = r#"
def fibonacci(n){
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}
println(fibonacci(1))
println(fibonacci(2))
println(fibonacci(3))
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
        let add_one = def(x) {
            return x + 1
        }
        println(add_one(10))
    "#,
    )
    .expect("failed");
    assert!(output.contains("11"));
}

#[test]
fn test_immediate_invocation() {
    let output = run_chen_lang_code(
        r#"
        let result = def(x, y) {
            return x * y
        } (5, 6)
        println(result)
    "#,
    )
    .expect("failed");
    assert!(output.contains("30"));
}

#[test]
fn test_high_order_function() {
    let output = run_chen_lang_code(
        r#"
        def apply(f, val) {
            return f(val)
        }
        
        let res = apply(def(x){ return x * 2 }, 21)
        println(res)
    "#,
    )
    .expect("failed");
    assert!(output.contains("42"));
}

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
