use crate::common::run_chen_lang_code;

#[test]
fn test_basic_closure_capture() {
    let code = r#"
    function make_adder(x)
        return function(y)
            return x + y
        end
    end
    local add5 = make_adder(5)
    println(add5(10))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_closure_multiple_upvalues() {
    let code = r#"
    function make_sandwich(bread)
        local cheese = "cheddar"
        return function(meat)
            return bread + " with " + meat + " and " + cheese
        end
    end
    local s = make_sandwich("rye")
    println(s("turkey"))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "rye with turkey and cheddar");
}

#[test]
fn test_nested_closures() {
    let code = r#"
    function outer(a)
        return function(b)
            return function(c)
                return a + b + c
            end
        end
    end
    local f1 = outer(100)
    local f2 = f1(20)
    println(f2(3))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "123");
}

#[test]
fn test_closure_mutation() {
    let code = r#"
    function make_counter()
        local count = 0
        return function()
            count = count + 1
            return count
        end
    end
    local counter = make_counter()
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
    local captured = "success"
    local f = function()
        return captured
    end
    println(f())
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "success");
}

#[test]
fn test_closure_in_loop() {
    let code = r#"
    local funcs = []
    local i = 0
    while i < 3 do
        local val = i
        funcs:push(function() return val end)
        i = i + 1
    end
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
    function a(x)
        return function() return x end
    end
    function b(f)
        return f()
    end
    local c = a("hello")
    println(b(c))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "hello");
}
