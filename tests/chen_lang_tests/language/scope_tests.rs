use crate::common::run_chen_lang_code;

#[test]
fn test_function_scope_isolation() {
    let code = r#"
function func()
    local local_var = "local_value"
    return "test"
end

local result = "abcd"
func()
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("abcd"));
}

#[test]
fn test_function_variable_not_leaked() {
    let code = r#"
function func()
    local secret = "should_not_be_visible"
    return "done"
end

func()
print(secret)
"#;

    let result = run_chen_lang_code(code);
    assert!(result.is_err());
}

#[test]
fn test_if_statement_scope() {
    let code = r#"
local x = "global"
if true then
    local x = "local"
    println(x)
end
println(x)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("local"));
    assert!(lines[1].contains("global"));
}

#[test]
fn test_for_loop_scope() {
    let code = r#"
local i = 1
while i <= 3 do
    local temp = i
    println(temp)
    i = i + 1
end
println(i)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].trim(), "1");
    assert_eq!(lines[1].trim(), "2");
    assert_eq!(lines[2].trim(), "3");
    assert_eq!(lines[3].trim(), "4");
}

#[test]
fn test_simple_block_assignment() {
    let code = r#"
    local calc = function()
        local a = 10
        local b = 20
        return a + b
    end
    local x = calc()
    if x == 30 then
        println("Test 1 Passed")
    else
        println("Test 1 Failed: Expected 30, got " + x)
    end
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
}

#[test]
fn test_nested_blocks() {
    let code = r#"
    local outer = function()
        local c = 5
        local inner = function()
            local d = 10
            return c + d
        end
        return inner()
    end
    local y = outer()
    if y == 15 then
        println("Test 2 Passed")
    else
        println("Test 2 Failed: Expected 15, got " + y)
    end
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 2 Passed"));
}

#[test]
fn test_block_with_if_else() {
    let code = r#"
    local z = if true then 1 else 0 end
    println("Test 3 (If statement): " + z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 3 (If statement):"));
}

#[test]
fn test_block_ending_with_assignment() {
    let code = r#"
    local get_w = function()
        local f = 1
        f = f + 1
    end
    local w = get_w()
    println("Test 4 (Assignment): " + w)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 4 (Assignment):"));
}

#[test]
fn test_empty_block() {
    let code = r#"
    local empty = function()
    end
    local v = empty()
    println("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 5 (Empty):"));
}

#[test]
fn test_block_value_simple() {
    let code = r#"
    local calc = function()
        return 5 + 10
    end
    local result = calc()
    println(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_block_value_with_variables() {
    let code = r#"
    local calc = function()
        local x = 10
        local y = 20
        return x + y
    end
    local result = calc()
    println(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "30");
}

#[test]
fn test_all_scope_value_tests() {
    let code = r#"
local calc_ab = function()
    local a = 10
    local b = 20
    return a + b
end
local x = calc_ab()
if x == 30 then
    println("Test 1 Passed")
else
    println("Test 1 Failed: Expected 30, got " + x)
end

local outer = function()
    local c = 5
    local inner = function()
        local d = 10
        return c + d
    end
    return inner()
end
local y = outer()
if y == 15 then
    println("Test 2 Passed")
else
    println("Test 2 Failed: Expected 15, got " + y)
end

local z = if true then 1 else 0 end
println("Test 3 (If statement): " + z)

local get_w = function()
    local f = 1
    f = f + 1
end
local w = get_w()
println("Test 4 (Assignment): " + w)

local empty = function()
end
local v = empty()
println("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
    assert!(output.contains("Test 2 Passed"));
    assert!(output.contains("Test 3 (If statement):"));
    assert!(output.contains("Test 4 (Assignment):"));
    assert!(output.contains("Test 5 (Empty):"));
}
