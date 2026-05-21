use crate::common::run_chen_lang_code;

#[test]
fn test_basic_closure_capture() {
    let code = r#"
    function make_adder(x) {
        return function(y) {
            return x + y
        }
    }
    let add5 = make_adder(5)
    console.log(add5(10))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_closure_multiple_upvalues() {
    let code = r#"
    function make_sandwich(bread) {
        let cheese = "cheddar"
        return function(meat) {
            return bread + " with " + meat + " and " + cheese
        }
    }
    let s = make_sandwich("rye")
    console.log(s("turkey"))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "rye with turkey and cheddar");
}

#[test]
fn test_nested_closures() {
    let code = r#"
    function outer(a) {
        return function(b) {
            return function(c) {
                return a + b + c
            }
        }
    }
    let f1 = outer(100)
    let f2 = f1(20)
    console.log(f2(3))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "123");
}

#[test]
fn test_closure_mutation() {
    let code = r#"
    function make_counter() {
        let count = 0
        return function() {
            count = count + 1
            return count
        }
    }
    let counter = make_counter()
    console.log(counter())
    console.log(counter())
    console.log(counter())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "1");
    assert_eq!(lines[1], "2");
    assert_eq!(lines[2], "3");
}

#[test]
fn test_global_closure_assignment() {
    let code = r#"
    let captured = "success"
    let f = function() {
        return captured
    }
    console.log(f())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "success");
}

#[test]
fn test_closure_in_loop() {
    let code = r#"
    let funcs = []
    let i = 0
    while (i < 3) {
        let val = i
        funcs.push(function() { return val })
        i = i + 1
    }
    console.log(funcs[0]())
    console.log(funcs[1]())
    console.log(funcs[2]())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "0");
    assert_eq!(lines[1], "1");
    assert_eq!(lines[2], "2");
}

#[test]
fn test_closure_across_files_simulated() {
    let code = r#"
    function a(x) {
        return function() { return x }
    }
    function b(f) {
        return f()
    }
    let c = a("hello")
    console.log(b(c))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "hello");
}
