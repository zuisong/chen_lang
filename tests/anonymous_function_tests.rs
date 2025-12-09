mod common;
use common::run_chen_lang_code;

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
