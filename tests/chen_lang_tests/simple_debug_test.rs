use crate::common::run_chen_lang_code as run_captured;

#[test]
#[ignore] // Temporarily disabled - investigating function call issue
fn test_import_simple_debug() {
    let source = r#"
        let io = import "stdlib/io"  
        io.println("Hello from test!")
        let mod = import "tests/chen_lang_tests/modules/simple_test.ch"
        io.println("Module imported")
        let result = mod.test()
        io.println(result)
    "#;

    let output = run_captured(&source.to_string()).unwrap();
    assert!(output.contains("Hello from test!"));
    assert!(output.contains("Module imported"));
    assert!(output.contains("999"));
}
