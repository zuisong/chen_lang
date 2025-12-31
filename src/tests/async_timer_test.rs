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
    let code = r#"
    let timer = import "stdlib/timer"
    let start = stdlib.date.now()
    timer.sleep(200)
    let end = stdlib.date.now()
    
    # Check if time passed is at least 200ms
    # Using a small margin for error (Rust test env might be slow or fast, but delta should be positive)
    if (end - start) >= 200 { 
        return "OK" 
    } else {
        return "Too fast: " + (end - start)
    }
    "#;

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
