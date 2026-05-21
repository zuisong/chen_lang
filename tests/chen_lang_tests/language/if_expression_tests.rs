use crate::common::run_chen_lang_code;

#[test]
fn test_if_expression_true_branch() {
    let code = r#"
    let a = if (true) { 10 } else { 20 }
    console.log("a should be 10: " + a)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("a should be 10: 10"));
}

#[test]
fn test_if_expression_false_branch() {
    let code = r#"
    let b = if (false) { 10 } else { 20 }
    console.log("b should be 20: " + b)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("b should be 20: 20"));
}

#[test]
fn test_nested_if_expression() {
    let code = r#"
    let c = if (true) {
        if (false) { 100 } else { 200 }
    } else {
        300
    }
    console.log("c should be 200: " + c)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("c should be 200: 200"));
}

#[test]
fn test_if_expression_without_else() {
    let code = r#"
    let d = if (false) { 10 }
    console.log("d should be null: " + d)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("d should be null: null"));
}

#[test]
fn test_if_expression_with_block_logic() {
    let code = r#"
    let e = if (true) {
        let x = 5
        x * 2
    } else {
        0
    }
    console.log("e should be 10: " + e)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("e should be 10: 10"));
}

#[test]
fn test_if_expression_in_binary_operation() {
    let code = r#"
    let f = 10 + if (true) { 5 } else { 0 }
    console.log("f should be 15: " + f)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("f should be 15: 15"));
}

#[test]
fn test_if_expression_as_function_argument() {
    let code = r#"
    function check(val) {
        console.log("val is: " + val)
    }
    check(if (true) { "yes" } else { "no" })
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("val is: yes"));
}

#[test]
fn test_all_if_expressions() {
    // Run the complete test file
    let code = r#"
let a = if (true) { 10 } else { 20 }
console.log("a should be 10: " + a)

let b = if (false) { 10 } else { 20 }
console.log("b should be 20: " + b)

let c = if (true) {
    if (false) { 100 } else { 200 }
} else {
    300
}
console.log("c should be 200: " + c)

let d = if (false) { 10 }
console.log("d should be null: " + d)

let e = if (true) {
    let x = 5
    x * 2
} else {
    0
}
console.log("e should be 10: " + e)

let f = 10 + if (true) { 5 } else { 0 }
console.log("f should be 15: " + f)

function check(val) {
    console.log("val is: " + val)
}
check(if (true) { "yes" } else { "no" })
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("a should be 10: 10"));
    assert!(output.contains("b should be 20: 20"));
    assert!(output.contains("c should be 200: 200"));
    assert!(output.contains("d should be null: null"));
    assert!(output.contains("e should be 10: 10"));
    assert!(output.contains("f should be 15: 15"));
    assert!(output.contains("val is: yes"));
}

#[test]
fn test_if_else_if_expression() {
    let code = r#"
    let x = 15
    let result = if (x < 10) {
        "small"
    } else if (x < 20) {
        "medium"
    } else {
        "large"
    }
    console.log("result should be medium: " + result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("result should be medium: medium"));
}
