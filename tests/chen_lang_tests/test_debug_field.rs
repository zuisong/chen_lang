use chen_lang::run_captured;

#[test]
fn test_access_imported_field() {
    let source = r#"
let io = import "stdlib/io"
let math = import "tests/chen_lang_tests/modules/math_utils.ch"
io.println("math object:")
io.println(math)
io.println("math.add:")
io.println(math.add)
"#;
    let output = run_captured(source.to_string()).unwrap();
    println!("{}", output);
    assert!(output.contains("math object:"));
}
