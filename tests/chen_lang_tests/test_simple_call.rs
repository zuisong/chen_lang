use chen_lang::run_captured;

#[test]
fn test_simple_method_call() {
    let source = r#"
let io = import "stdlib/io"
let math = import "tests/chen_lang_tests/modules/math_utils.ch"
let result = math.add(10, 20)
io.println(result)
"#;
    let output = run_captured(source.to_string()).unwrap();
    println!("Output: {}", output);
    assert!(output.contains("30"));
}
