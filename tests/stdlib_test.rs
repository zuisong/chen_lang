mod common;
use common::run_chen_lang_code;

#[test]
fn test_date() {
    let code = r#"
    let d = Date.new()
    println(d.__type)
    println(d.format('%Y'))
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Date"));
    assert!(output.contains("20"));
}

#[test]
fn test_json() {
    let code = r#"
    let obj = JSON.parse('{"a": 1}')
    println(obj.a)
    let s = JSON.stringify(obj)
    println(s)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("1"));
    assert!(output.contains("{\"a\":1}"));
}
