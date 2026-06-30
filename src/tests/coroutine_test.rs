use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

#[test]
fn test_async_await_basic() {
    let code = r#"
    function task(v)
        return v + 1
    end

    local t = coroutine.create(task)
    
    local res = coroutine.resume(t, 10)
    if res ~= 11 then
        error("Async task failed: expected 11, got " .. res)
    end
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
    function worker()
         local got = coroutine.yield("start")
         return got
    end

    local co = coroutine.create(worker)
    local res1 = coroutine.resume(co)
    
    if res1 ~= "start" then
        error("Fail 1: " .. res1)
    end
    
    local res2 = coroutine.resume(co, "back")
    if res2 ~= "back" then
        error("Fail 2: " .. res2)
    end
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
    function f()
        local v = coroutine.yield("Y1")
        return "R:" .. v
    end

    local co = coroutine.create(f)
    local a = coroutine.resume(co)         -- => "Y1"
    local b = coroutine.resume(co, "X")    -- => "R:X"

    return a .. "|" .. b
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
    function task_a()
        local io = require("stdlib/io")
        local i = 0
        while i < 3 do
             io.print("Task A: " .. i)
             coroutine.yield(i)
             i = i + 1
        end
        return "A_DONE"
    end
    
    local t = coroutine.create(task_a)
    if coroutine.status(t) ~= "suspended" then error("Init status error") end
    
    coroutine.resume(t)
    if coroutine.status(t) ~= "suspended" then error("After yield status error") end
    
    coroutine.resume(t)
    coroutine.resume(t)
    local final_res = coroutine.resume(t)
    
    if coroutine.status(t) ~= "dead" then error("Finish status error: " .. coroutine.status(t)) end
    if final_res ~= "A_DONE" then error("Return val error") end
    
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

#[test]
fn test_yield_from_root_should_error() {
    let code = r#"
    function range(n)
        local i = 0
        while i < n do
            coroutine.yield(i)
            i = i + 1
        end
    end
    
    range(5)
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);

    assert!(res.is_err(), "Expected error when yield from root, but got: {:?}", res);
    let err_msg = format!("{}", res.unwrap_err());
    assert!(
        err_msg.contains("yield") && err_msg.contains("root"),
        "Expected 'yield from root' error, but got: {}",
        err_msg
    );
}

#[test]
fn test_yield_in_spawn_without_caller_should_error() {
    let code = r#"
    local co = coroutine.create(function()
        coroutine.yield("paused")
        return "done"
    end)
    
    coroutine.spawn(co)
    local results = coroutine.await_all({co})
    results[0]
    "#;

    let ast = parse_from_source(&code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);

    let mut vm = VM::new();
    let res = vm.execute(&program);

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

#[test]
fn test_spawn_await_all_basic() {
    let code = r#"
    function task(x)
        return x * 2
    end
    
    local co1 = coroutine.create(function() return task(5) end)
    local co2 = coroutine.create(function() return task(10) end)
    
    coroutine.spawn(co1)
    coroutine.spawn(co2)
    
    local results = coroutine.await_all({co1, co2})
    
    if results[0] ~= 10 then
        error("Expected results[0] = 10, got " .. results[0])
    end
    if results[1] ~= 20 then
        error("Expected results[1] = 20, got " .. results[1])
    end
    
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

#[test]
fn test_spawn_multiple_coroutines() {
    let code = r#"
    local results_collector = {}
    
    function task(name)
        return name .. "_done"
    end
    
    local co1 = coroutine.create(function() return task("A") end)
    local co2 = coroutine.create(function() return task("B") end)
    local co3 = coroutine.create(function() return task("C") end)
    
    coroutine.spawn(co1)
    coroutine.spawn(co2)
    coroutine.spawn(co3)
    
    local results = coroutine.await_all({co1, co2, co3})
    
    if results[0] ~= "A_done" then error("Error: " .. results[0]) end
    if results[1] ~= "B_done" then error("Error: " .. results[1]) end
    if results[2] ~= "C_done" then error("Error: " .. results[2]) end
    
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

#[test]
fn test_spawn_coroutine_status() {
    let code = r#"
    local co = coroutine.create(function() return 42 end)
    
    if coroutine.status(co) ~= "suspended" then
        error("Expected suspended, got " .. coroutine.status(co))
    end
    
    coroutine.spawn(co)
    
    local results = coroutine.await_all({co})
    
    if coroutine.status(co) ~= "dead" then
        error("Expected dead after await_all, got " .. coroutine.status(co))
    end
    
    if results[0] ~= 42 then
        error("Expected 42, got " .. results[0])
    end
    
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

#[test]
fn test_await_all_empty_array() {
    let code = r#"
    local results = coroutine.await_all({})
    
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
