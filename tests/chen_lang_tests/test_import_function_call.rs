use chen_lang::run_captured;

#[test]
fn test_call_imported_function() {
    let source = r#"
let io = import "stdlib/io"
let math = import "tests/chen_lang_tests/modules/math_utils.ch"
io.println("Before call")
let result = math.add(10, 20)
io.println("After call")
io.println(result)
"#;
    let output = run_captured(source.to_string()).unwrap();
    assert!(output.contains("Before call"));
    assert!(output.contains("After call"));
    assert!(output.contains("30"));
}
