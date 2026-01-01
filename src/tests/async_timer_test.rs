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
    // Tests that we can sleep for a duration
    // And that the VM waits for it.

    // NOTE: stdlib.date.now() returns milliseconds.
    // importing stdlib/date implicitly? No, `stdlib.date` is not standard.
    // We need `import "stdlib/date"`.
    // But `import` returns the module.
    // `native_date` has `now`.

    let code_fixed = r#"
    let timer = import "stdlib/timer"
    let date = import "stdlib/date"
    let start = date.now()
    timer.sleep(100)
    let end = date.now()
    let diff = end - start
    
    # We can't assert exact time, but it should be > 50ms and < 2000ms
    if diff >= 50 {
        return "OK"
    } else {
        return "FAIL: " + diff
    }
    "#;

    assert_eq!(run_code(code_fixed), "OK");
}

#[test]
fn test_async_interleaving() {
    // Determine if we can run two timers?
    // This requires `spawn`. We don't have `spawn` exposed yet.
    // But we can check if `sleep` works in a loop (sequential).

    let code = r#"
    let timer = import "stdlib/timer"
    timer.sleep(10)
    timer.sleep(10)
    return "Done"
    "#;
    assert_eq!(run_code(code), "Done");
}

#[test]
fn test_spawn_closure_with_sleep() {
    let code = r#"
    let timer = import "stdlib/timer"
    let co = coroutine.create(def() {
        # 匿名函数直接调用 native async，这会触发 Yield
        timer.sleep(50)
        return "WakeUp"
    })
    
    coroutine.spawn(co)
    let results = coroutine.await_all([co])
    
    return results[0]
    "#;

    assert_eq!(run_code(code), "WakeUp");
}

#[test]
fn test_spawn_closure_captures_and_sleep() {
    let code = r#"
    let timer = import "stdlib/timer"
    let msg = "Capturing"
    
    let co = coroutine.create(def() {
        timer.sleep(10)
        return msg + " Works"
    })
    
    coroutine.spawn(co)
    let results = coroutine.await_all([co])
    
    return results[0]
    "#;

    assert_eq!(run_code(code), "Capturing Works");
}
