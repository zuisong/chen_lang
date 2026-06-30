use crate::common::run_chen_lang_code;

#[test]
fn test_multiline_simple_addition() {
    let code = r#"
    local x = 1 + 
        2
    print(x)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_multiline_block_expression() {
    let code = r#"
    local calc = function()
        local a = 10
        return a * 2
    end
    local y = calc()
    print(y)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn test_multiline_complex_expression() {
    let code = r#"
    local z = 1 + 2 * 3 +
            4
    print(z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "11");
}

#[test]
fn test_all_multiline_expressions() {
    let code = r#"
local x = 1 + 
    2
print(x)

local calc = function()
    local a = 10
    return a * 2
end
local y = calc()
print(y)

local z = 1 + 2 * 3 +
        4
print(z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].trim(), "3");
    assert_eq!(lines[1].trim(), "20");
    assert_eq!(lines[2].trim(), "11");
}
