use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_custom_module_simple() {
    let source = r#"
        let mod = Chen.load("tests/fixtures/temp_module.chen.js")
        console.log(mod.name)
        console.log(mod.greet("World"))
    "#;

    let output = run_captured(source).unwrap();

    assert!(output.contains("Module"));
    assert!(output.contains("Hello, World from Module"));
}

#[test]
fn test_import_custom_module_relative_path() {
    // Note: Paths are currently relative to CWD (project root during cargo test)
    let source = r#"
        let math = Chen.load("tests/fixtures/math_utils.chen.js")
        console.print(math.add(10, 20))
    "#;

    let output = run_captured(source).unwrap();

    assert!(output.contains("30"));
}

#[test]
fn test_import_custom_module_caching() {
    let source = r#"
        let m1 = Chen.load("tests/fixtures/cached_module.chen.js")
        let m2 = Chen.load("tests/fixtures/cached_module.chen.js")
    "#;

    let output = run_captured(source).unwrap();

    // "Module Loaded" should appear only once if caching works
    let matches: Vec<_> = output.matches("Module Loaded").collect();
    assert_eq!(matches.len(), 1, "Module should be loaded exactly once due to caching");
}

#[test]
fn test_call_imported_function() {
    let source = r#"
        let math = Chen.load("tests/fixtures/math_utils.chen.js")
        console.log("Before call")
        let result = math.add(10, 20)
        console.log("After call")
        console.log(result)
    "#;
    let output = run_captured(source).unwrap();
    assert!(output.contains("Before call"));
    assert!(output.contains("After call"));
    assert!(output.contains("30"));
}
