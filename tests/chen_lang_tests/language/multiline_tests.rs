use crate::common::run_chen_lang_code;

#[test]
fn test_multiline_simple_addition() {
    let code = r#"
    let x = 1 + 
        2
    println(x)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_multiline_block_expression() {
    let code = r#"
    let y = {
        let a = 10
        a * 
        2
    }
    println(y)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn test_multiline_complex_expression() {
    let code = r#"
    let z = 1 + 2 * 3 +
            4
    println(z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "11");
}

#[test]
fn test_all_multiline_expressions() {
    // Run the complete test file
    let code = r#"
let x = 1 + 
    2
println(x)

let y = {
    let a = 10
    a * 
    2
}
println(y)

let z = 1 + 2 * 3 +
        4
println(z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].trim(), "3");
    assert_eq!(lines[1].trim(), "20");
    assert_eq!(lines[2].trim(), "11");
}
