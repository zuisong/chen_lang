use crate::common::run_chen_lang_code;

#[test]
fn test_basic_closure_capture() {
    let code = r#"
    def make_adder(x) {
        return def(y) {
            return x + y
        }
    }
    let add5 = make_adder(5)
    println(add5(10))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_closure_multiple_upvalues() {
    let code = r#"
    def make_sandwich(bread) {
        let cheese = "cheddar"
        return def(meat) {
            return bread + " with " + meat + " and " + cheese
        }
    }
    let s = make_sandwich("rye")
    println(s("turkey"))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "rye with turkey and cheddar");
}

#[test]
fn test_nested_closures() {
    let code = r#"
    def outer(a) {
        return def(b) {
            return def(c) {
                return a + b + c
            }
        }
    }
    let f1 = outer(100)
    let f2 = f1(20)
    println(f2(3))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "123");
}

#[test]
fn test_closure_mutation() {
    let code = r#"
    def make_counter() {
        let count = 0
        return def() {
            count = count + 1
            return count
        }
    }
    let counter = make_counter()
    println(counter())
    println(counter())
    println(counter())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "1");
    assert_eq!(lines[1], "2");
    assert_eq!(lines[2], "3");
}

#[test]
fn test_global_closure_assignment() {
    // This previously failed with "Invalid operation: Null get_upvalue Null"
    let code = r#"
    let captured = "success"
    let f = def() {
        return captured
    }
    println(f())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "success");
}

#[test]
fn test_closure_in_loop() {
    let code = r#"
    let funcs = []
    let i = 0
    for i < 3 {
        let val = i
        funcs:push(def() { return val })
        i = i + 1
    }
    println(funcs[0]())
    println(funcs[1]())
    println(funcs[2]())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "0");
    assert_eq!(lines[1], "1");
    assert_eq!(lines[2], "2");
}

#[test]
fn test_closure_across_files_simulated() {
    // Basic test to ensure current_closure is restored correctly after multiple calls
    let code = r#"
    def a(x) {
        return def() { return x }
    }
    def b(f) {
        return f()
    }
    let c = a("hello")
    println(b(c))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "hello");
}
