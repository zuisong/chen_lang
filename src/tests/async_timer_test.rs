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

    let code_fixed = r#"
    let timer = Chen.timer
    let date = Chen.date
    let sleep = timer.sleep
    let now = date.now
    let start = now()
    await sleep(100)
    let end = now()
    let diff = end - start

    // We can't assert exact time, but it should be > 50ms and < 2000ms
    if (diff >= 50) {
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
    let timer = Chen.timer
    await timer.sleep(10)
    await timer.sleep(10)
    return "Done"
    "#;
    assert_eq!(run_code(code), "Done");
}

#[test]
fn test_spawn_closure_with_sleep() {
    let code = r#"
    let timer = Chen.timer
    async function run() {
        await timer.sleep(50)
        return "WakeUp"
    }
    let p = run()
    return await p
    "#;

    assert_eq!(run_code(code), "WakeUp");
}

#[test]
fn test_spawn_closure_captures_and_sleep() {
    let code = r#"
    let timer = Chen.timer
    let msg = "Capturing"
    async function run() {
        await timer.sleep(10)
        return msg + " Works"
    }
    let p = run()
    return await p
    "#;

    assert_eq!(run_code(code), "Capturing Works");
}

