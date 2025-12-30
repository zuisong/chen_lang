use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_custom_module_simple() {
    let source = r#"
        let io = import "stdlib/io"
        let mod = import "tests/fixtures/temp_module.ch"
        io.println(mod.name)
        io.println(mod.greet("World"))
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    assert!(output.contains("Module"));
    assert!(output.contains("Hello, World from Module"));
}

#[test]
fn test_import_custom_module_relative_path() {
    // Note: Paths are currently relative to CWD (project root during cargo test)
    let source = r#"
        let io = import "stdlib/io"
        let math = import "tests/fixtures/math_utils.ch"
        io.print(math.add(10, 20))
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    assert!(output.contains("30"));
}

#[test]
fn test_import_custom_module_caching() {
    let source = r#"
        let m1 = import "tests/fixtures/cached_module.ch"
        let m2 = import "tests/fixtures/cached_module.ch"
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    // "Module Loaded" should appear only once if caching works
    let matches: Vec<_> = output.matches("Module Loaded").collect();
    assert_eq!(matches.len(), 1, "Module should be loaded exactly once due to caching");
}

#[test]
fn test_call_imported_function() {
    let source = r#"
        let io = import "stdlib/io"
        let math = import "tests/fixtures/math_utils.ch"
        io.println("Before call")
        let result = math.add(10, 20)
        io.println("After call")
        io.println(result)
    "#;
    let output = run_captured(&source.to_string()).unwrap();
    assert!(output.contains("Before call"));
    assert!(output.contains("After call"));
    assert!(output.contains("30"));
}
