mod common;
use common::run_chen_lang_code;

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
