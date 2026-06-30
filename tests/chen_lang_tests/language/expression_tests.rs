use crate::common::run_chen_lang_code;

#[test]
fn test_if_expression() {
    let code = r#"
    local a = if true then 10 else 20 end
    println(a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "10");
}

#[test]
fn test_if_expression_else() {
    let code = r#"
    local a = if false then 10 else 20 end
    println(a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn test_if_expression_nested() {
    let code = r#"
    local a = if true then
        if false then 1 else 2 end
    else
        3
    end
    println(a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "2");
}

#[test]
fn test_if_expression_in_math() {
    let code = r#"
    local a = 5 + if true then 10 else 0 end
    println(a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_function_implicit_return() {
    let code = r#"
    function add(a, b)
        a + b
    end
    println(add(10, 20))
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "30");
}

#[test]
fn test_function_block_complex() {
    let code = r#"
    function complex_logic(x)
        local y = x * 2
        if y > 10 then
            return y - 5
        else
            return y + 5
        end
    end
    println(complex_logic(4))  -- 4*2=8, 8+5=13
    println(complex_logic(6))  -- 6*2=12, 12-5=7
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0].trim(), "13");
    assert_eq!(lines[1].trim(), "7");
}
