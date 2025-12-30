use chen_lang::run_captured;

#[test]
fn test_simple_stdlib_import() {
    let source = r#"
let io = import "stdlib/io"
io.println("test")
"#;
    let output = run_captured(source.to_string()).unwrap();
    assert!(output.contains("test"));
}
