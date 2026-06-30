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
    let code = r#"
    function f(x)
        return "Arg: " .. x
    end
    local co = coroutine.create(f, 100)
    local res = coroutine.resume(co)
    return res
    "#;
    assert_eq!(run_code(code), "Arg: 100");
}

#[test]
fn test_resume_with_args_start() {
    let code = r#"
    function f(x)
        return "Arg: " .. x
    end
    local co = coroutine.create(f)
    local res = coroutine.resume(co, 200)
    return res
    "#;
    assert_eq!(run_code(code), "Arg: 200");
}

#[test]
fn test_resume_pass_data() {
    let code = r#"
    function f()
        local val = coroutine.yield("start")
        return "Got: " .. val
    end
    local co = coroutine.create(f)
    coroutine.resume(co) -- Start, returns "start"
    local res = coroutine.resume(co, "World")
    return res
    "#;
    assert_eq!(run_code(code), "Got: World");
}

#[test]
fn test_yield_pass_data() {
    let code = r#"
    function f()
        coroutine.yield("from_yield")
        return 0
    end
    local co = coroutine.create(f)
    local res = coroutine.resume(co)
    return res
    "#;
    assert_eq!(run_code(code), "from_yield");
}

#[test]
fn test_resume_no_args_is_nil() {
    let code = r#"
    function f()
        local val = coroutine.yield(1)
        if val == nil then return "Was Nil" end
        return "Not Nil"
    end
    local co = coroutine.create(f)
    coroutine.resume(co)
    local res = coroutine.resume(co) -- No args
    return res
    "#;
    assert_eq!(run_code(code), "Was Nil");
}
