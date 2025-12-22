use chen_lang::run_captured as run_captured_orig;

fn run_captured(code: String) -> Result<String, chen_lang::ChenError> {
    let prelude = "import stdlib/io\nlet println = io.println\n";
    run_captured_orig(format!("{}{}", prelude, code))
}

#[test]
fn test_object_keys_basic() {
    let code = r#"
        let obj = #{ a: 1, b: 2 }
        let keys = obj:keys()
        
        # Verify length
        println(keys:len())
        
        # Verify content (order might vary but IndexMap preserves insertion order)
        println(keys[0])
        println(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("2"));
    assert!(output.contains("a"));
    assert!(output.contains("b"));
}

#[test]
fn test_object_keys_iteration() {
    let code = r#"
        let obj = #{ x: 10, y: 20, z: 30 }
        let keys = obj:keys()
        let i = 0
        for i < keys:len() {
            let k = keys[i]
            println(k, "=", obj[k])
            i = i + 1
        }
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("x=10"));
    assert!(output.contains("y=20"));
    assert!(output.contains("z=30"));
}

#[test]
fn test_array_keys() {
    let code = r#"
        let arr = [100, 200]
        let keys = arr:keys()
        println(keys:len())
        println(keys[0])
        println(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("2"));
    assert!(output.contains("0"));
    assert!(output.contains("1"));
}

#[test]
fn test_empty_object_keys() {
    let code = r#"
        let obj = #{}
        let keys = obj:keys()
        println(keys:len())
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("0"));
}

#[test]
fn test_keys_on_non_object() {
    // String has method len(), but not keys() currently unless we added it (we didn't).
    // Actually, string_prototype uses same GetField logic, so if we implemented it in GetField/GetMethod generic fallback
    // it depends on how we implemented it.
    // In vm.rs, we checked: `if let Value::Object(_) = obj`.
    // So strings should NOT have keys().
    let code = r#"
        let s = "hello"
        let k = s.keys()
    "#;

    let result = run_captured(code.to_string());
    // Should fail with TypeMismatch or similar because s.keys is null, and we try to call it?
    // Wait, if s.keys lookup returns Null (because generic fallback checks Object type),
    // then `let k = s.keys()` tries to call Null.
    // VM should error "Attempt to call non-function value".
    assert!(result.is_err());
}
