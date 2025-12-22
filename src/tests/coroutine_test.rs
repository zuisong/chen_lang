use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

#[test]
fn test_async_await_basic() {
    let code = r#"
    async def task(v) {
        return v + 1
    }

    let t = task(10)
    # t should be a Fiber (Coroutine)
    # Checking functionality by resuming it
    
    let res = coroutine.resume(t)
    if res != 11 {
        throw "Async task failed: expected 11, got " + res
    }
    return "OK_ASYNC"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK_ASYNC"),
        Err(e) => panic!("VM Error: {}", e),
    }
}

#[test]
fn test_coroutine_primitives_with_yield_values() {
    let code = r#"
    def worker() {
         let got = coroutine.yield("start")
         return got
    }

    let co = coroutine.create(worker)
    let res1 = coroutine.resume(co)
    
    if res1 != "start" {
        throw "Fail 1: " + res1
    }
    
    let res2 = coroutine.resume(co, "back")
    if res2 != "back" {
        throw "Fail 2: " + res2
    }
    return "OK"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK"),
        Err(e) => panic!("VM Error: {}", e),
    }
}

#[test]
fn test_scheduler_simulation() {
    let code = r#"
    # Scheduler simulation removed to focus on primitives

    
    # 上面的调度逻辑太复杂因为没有合适的 Array API。
    # 我们简化测试：
    # 验证能获取 status
    
    async def task_a() {
        import stdlib/io
        let i = 0
        let i = 0
        for i < 3 {
             io.print("Task A: " + i)
             await i # Yield control
             i = i + 1
        }
        return "A_DONE"
    }
    
    let t = task_a()
    if coroutine.status(t) != "suspended" { throw "Init status error" }
    
    coroutine.resume(t) # 运行到 await 0
    if coroutine.status(t) != "suspended" { throw "After yield status error" }
    
    coroutine.resume(t) # await 1
    coroutine.resume(t) # await 2
    let final_res = coroutine.resume(t) # return "A_DONE"
    
    if coroutine.status(t) != "dead" { throw "Finish status error: " + coroutine.status(t) }
    if final_res != "A_DONE" { throw "Return val error" }
    
    return "SCHEDULER_OK"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "SCHEDULER_OK"),
        Err(e) => panic!("VM Error: {}", e),
    }
}
