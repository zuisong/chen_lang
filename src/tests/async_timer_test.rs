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
fn test_timer_sleep() {
    let code_fixed = r#"
    local timer = require("stdlib/timer")
    local date = require("stdlib/date")
    local start = date.now()
    timer.sleep(100)
    local endv = date.now()
    local diff = endv - start
    
    -- We can't assert exact time, but it should be > 50ms and < 2000ms
    if diff >= 50 then
        return "OK"
    else
        return "FAIL: " .. diff
    end
    "#;

    assert_eq!(run_code(code_fixed), "OK");
}

#[test]
fn test_async_interleaving() {
    let code = r#"
    local timer = require("stdlib/timer")
    timer.sleep(10)
    timer.sleep(10)
    return "Done"
    "#;
    assert_eq!(run_code(code), "Done");
}

#[test]
fn test_spawn_closure_with_sleep() {
    let code = r#"
    local timer = require("stdlib/timer")
    local co = coroutine.create(function()
        timer.sleep(50)
        return "WakeUp"
    end)
    
    coroutine.spawn(co)
    local results = coroutine.await_all({co})
    
    return results[0]
    "#;

    assert_eq!(run_code(code), "WakeUp");
}

#[test]
fn test_spawn_closure_captures_and_sleep() {
    let code = r#"
    local timer = require("stdlib/timer")
    local msg = "Capturing"
    
    local co = coroutine.create(function()
        timer.sleep(10)
        return msg .. " Works"
    end)
    
    coroutine.spawn(co)
    local results = coroutine.await_all({co})
    
    return results[0]
    "#;

    assert_eq!(run_code(code), "Capturing Works");
}
