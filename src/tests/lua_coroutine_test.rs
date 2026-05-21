use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

fn run_code(code: &str) -> String {
    let ast = parse_from_source(code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);
    let mut vm = VM::new();
    match vm.execute(&program) {
        Ok(v) => v.to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

#[test]
fn test_create_with_args() {
    // 1. Chen.coroutine.create(f, arg) -> resume()
    let code = r#"
    function f(x) {
        return "Arg: " + x
    }
    let co = Chen.coroutine.create(f, 100)
    let res = Chen.coroutine.resume(co)
    return res
    "#;
    assert_eq!(run_code(code), "Arg: 100");
}

#[test]
fn test_resume_with_args_start() {
    // 2. Chen.coroutine.create(f) -> resume(co, arg)
    let code = r#"
    function f(x) {
        return "Arg: " + x
    }
    let co = Chen.coroutine.create(f)
    let res = Chen.coroutine.resume(co, 200)
    return res
    "#;
    assert_eq!(run_code(code), "Arg: 200");
}

#[test]
fn test_resume_pass_data() {
    // 3. resume(co, val) -> yield returns val
    let code = r#"
    function f() {
        let val = Chen.coroutine.yield("start")
        return "Got: " + val
    }
    let co = Chen.coroutine.create(f)
    Chen.coroutine.resume(co) // Start, returns "start"
    let res = Chen.coroutine.resume(co, "World")
    return res
    "#;
    assert_eq!(run_code(code), "Got: World");
}

#[test]
fn test_yield_pass_data() {
    // 4. yield(val) -> resume returns val
    let code = r#"
    function f() {
        Chen.coroutine.yield("from_yield")
        return 0
    }
    let co = Chen.coroutine.create(f)
    let res = Chen.coroutine.resume(co)
    return res
    "#;
    assert_eq!(run_code(code), "from_yield");
}

#[test]
fn test_resume_no_args_is_null() {
    // 5. resume(co) -> yield returns null
    let code = r#"
    function f() {
        let val = Chen.coroutine.yield(1)
        if (val == null) { return "Was Null" }
        return "Not Null"
    }
    let co = Chen.coroutine.create(f)
    Chen.coroutine.resume(co)
    let res = Chen.coroutine.resume(co) // No args
    return res
    "#;
    assert_eq!(run_code(code), "Was Null");
}
