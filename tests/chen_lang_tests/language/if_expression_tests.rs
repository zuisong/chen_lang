use crate::common::run_chen_lang_code;

#[test]
fn test_if_expression_true_branch() {
    let code = r#"
    local a = if true then 10 else 20 end
    print("a should be 10: " + a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("a should be 10: 10"));
}

#[test]
fn test_if_expression_false_branch() {
    let code = r#"
    local b = if false then 10 else 20 end
    print("b should be 20: " + b)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("b should be 20: 20"));
}

#[test]
fn test_nested_if_expression() {
    let code = r#"
    local c = if true then
        if false then 100 else 200 end
    else
        300
    end
    print("c should be 200: " + c)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("c should be 200: 200"));
}

#[test]
fn test_if_expression_without_else() {
    let code = r#"
    local d = if false then 10 end
    print("d should be nil: " + d)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("d should be nil: nil"));
}

#[test]
fn test_if_expression_with_block_logic() {
    let code = r#"
    local e = if true then
        local x = 5
        x * 2
    else
        0
    end
    print("e should be 10: " + e)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("e should be 10: 10"));
}

#[test]
fn test_if_expression_in_binary_operation() {
    let code = r#"
    local f = 10 + if true then 5 else 0 end
    print("f should be 15: " + f)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("f should be 15: 15"));
}

#[test]
fn test_if_expression_as_function_argument() {
    let code = r#"
    function check(val)
        print("val is: " + val)
    end
    check(if true then "yes" else "no" end)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("val is: yes"));
}

#[test]
fn test_all_if_expressions() {
    // Run the complete test file
    let code = r#"
-- Test if as an expression
local a = if true then 10 else 20 end
print("a should be 10: " + a)

local b = if false then 10 else 20 end
print("b should be 20: " + b)

-- Test nested if expression
local c = if true then
    if false then 100 else 200 end
else
    300
end
print("c should be 200: " + c)

-- Test if expression without else (should return nil)
local d = if false then 10 end
print("d should be nil: " + d)

-- Test if expression with block logic
local e = if true then
    local x = 5
    x * 2
else
    0
end
print("e should be 10: " + e)

-- Test if expression in binary operation
local f = 10 + if true then 5 else 0 end
print("f should be 15: " + f)

-- Test if expression as function argument
function check(val)
    print("val is: " + val)
end
check(if true then "yes" else "no" end)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("a should be 10: 10"));
    assert!(output.contains("b should be 20: 20"));
    assert!(output.contains("c should be 200: 200"));
    assert!(output.contains("d should be nil: nil"));
    assert!(output.contains("e should be 10: 10"));
    assert!(output.contains("f should be 15: 15"));
    assert!(output.contains("val is: yes"));
}

#[test]
fn test_if_else_if_expression() {
    let code = r#"
    local x = 15
    local result = if x < 10 then
        "small"
    else
        if x < 20 then
            "medium"
        else
            "large"
        end
    end
    print("result should be medium: " + result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("result should be medium: medium"));
}
