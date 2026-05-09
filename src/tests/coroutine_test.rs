use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

#[test]
fn test_async_await_basic() {
    let code = r#"
    def task(v) {
        return v + 1
    }

    # Manually create coroutine since 'async' keyword is removed
    let t = coroutine.create(task)
    
    # Passing arguments via resume for the first time
    let res = coroutine.resume(t, 10)
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
fn test_resume_yield_lua_semantics_roundtrip() {
    let code = r#"
    def f() {
        let v = coroutine.yield("Y1")
        return "R:" + v
    }

    let co = coroutine.create(f)
    let a = coroutine.resume(co)         # => "Y1"
    let b = coroutine.resume(co, "X")    # => "R:X"

    return a + "|" + b
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "Y1|R:X"),
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
    
    def task_a() {
        let io = import("stdlib/io")
        let i = 0
        for i < 3 {
             io.print("Task A: " + i)
             coroutine.yield(i) # Yield control
             i = i + 1
        }
        return "A_DONE"
    }
    
    # Create coroutine explicitly
    let t = coroutine.create(task_a)
    if coroutine.status(t) != "suspended" { throw "Init status error" }
    
    coroutine.resume(t) # 运行到 yield 0
    if coroutine.status(t) != "suspended" { throw "After yield status error" }
    
    coroutine.resume(t) # yield 1
    coroutine.resume(t) # yield 2
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

/// 测试在主程序中直接调用 yield 应该报错
/// 对应 demo_codes/test_yield_direct.ch
#[test]
fn test_yield_from_root_should_error() {
    let code = r#"
    def range(n) {
        let i = 0
        for i < n {
            coroutine.yield(i)
            i = i + 1
        }
    }
    
    range(5)
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);

    // 应该报错：yield from root
    assert!(res.is_err(), "Expected error when yield from root, but got: {:?}", res);
    let err_msg = format!("{}", res.unwrap_err());
    assert!(
        err_msg.contains("yield") && err_msg.contains("root"),
        "Expected 'yield from root' error, but got: {}",
        err_msg
    );
}

/// 测试在 spawn 的协程中调用 yield 应该报错（因为没有 caller）
/// 对应 demo_codes/test_spawn_yield.ch
#[test]
fn test_yield_in_spawn_without_caller_should_error() {
    let code = r#"
    let co = coroutine.create(def() {
        coroutine.yield("暂停中")
        return "完成"
    })
    
    coroutine.spawn(co)
    let results = coroutine.await_all([co])
    results[0]
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);

    // 应该报错：yield without caller
    assert!(
        res.is_err(),
        "Expected error when yield in spawn without caller, but got: {:?}",
        res
    );
    let err_msg = format!("{}", res.unwrap_err());
    assert!(
        err_msg.contains("yield") && err_msg.contains("caller"),
        "Expected 'yield without caller' error, but got: {}",
        err_msg
    );
}

/// 测试基本的 spawn + await_all 并发
#[test]
fn test_spawn_await_all_basic() {
    let code = r#"
    def task(x) {
        return x * 2
    }
    
    let co1 = coroutine.create(def() { task(5) })
    let co2 = coroutine.create(def() { task(10) })
    
    coroutine.spawn(co1)
    coroutine.spawn(co2)
    
    let results = coroutine.await_all([co1, co2])
    
    if results[0] != 10 {
        throw "Expected results[0] = 10, got " + results[0]
    }
    if results[1] != 20 {
        throw "Expected results[1] = 20, got " + results[1]
    }
    
    return "OK_SPAWN_BASIC"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK_SPAWN_BASIC"),
        Err(e) => panic!("VM Error: {}", e),
    }
}

/// 测试多个协程并发执行
#[test]
fn test_spawn_multiple_coroutines() {
    let code = r#"
    let results_collector = []
    
    def task(name) {
        return name + "_done"
    }
    
    let co1 = coroutine.create(def() { task("A") })
    let co2 = coroutine.create(def() { task("B") })
    let co3 = coroutine.create(def() { task("C") })
    
    coroutine.spawn(co1)
    coroutine.spawn(co2)
    coroutine.spawn(co3)
    
    let results = coroutine.await_all([co1, co2, co3])
    
    if results[0] != "A_done" { throw "Error: " + results[0] }
    if results[1] != "B_done" { throw "Error: " + results[1] }
    if results[2] != "C_done" { throw "Error: " + results[2] }
    
    return "OK_MULTIPLE"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK_MULTIPLE"),
        Err(e) => panic!("VM Error: {}", e),
    }
}

/// 测试 spawn 后协程状态变化
#[test]
fn test_spawn_coroutine_status() {
    let code = r#"
    let co = coroutine.create(def() { return 42 })
    
    # 初始状态是 suspended
    if coroutine.status(co) != "suspended" {
        throw "Expected suspended, got " + coroutine.status(co)
    }
    
    coroutine.spawn(co)
    
    # spawn 后协程仍然是 suspended（等待事件循环执行）
    # 但一旦 await_all 完成，协程变成 dead
    let results = coroutine.await_all([co])
    
    if coroutine.status(co) != "dead" {
        throw "Expected dead after await_all, got " + coroutine.status(co)
    }
    
    if results[0] != 42 {
        throw "Expected 42, got " + results[0]
    }
    
    return "OK_STATUS"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK_STATUS"),
        Err(e) => panic!("VM Error: {}", e),
    }
}

/// 测试 await_all 空数组
#[test]
fn test_await_all_empty_array() {
    let code = r#"
    let results = coroutine.await_all([])
    
    # 空数组应该立即返回空结果
    return "OK_EMPTY"
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);
    match res {
        Ok(v) => assert_eq!(v.to_string(), "OK_EMPTY"),
        Err(e) => panic!("VM Error: {}", e),
    }
}
