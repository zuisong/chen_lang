use chen_lang::run_captured as run_captured_orig;

fn run_captured(code: String) -> Result<String, chen_lang::ChenError> {
    let prelude = r#"local io = require("stdlib/io")
local println = io.println
"#;
    run_captured_orig(format!("{}{}", prelude, code))
}

#[test]
fn test_object_keys_basic() {
    let code = r#"
        local obj = { a = 1, b = 2 }
        local keys = obj:keys()
        
        -- Verify length
        println(keys:len())
        
        -- Verify content (order might vary but IndexMap preserves insertion order)
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
        local obj = { x = 10, y = 20, z = 30 }
        local keys = obj:keys()
        local i = 0
        while i < keys:len() do
            local k = keys[i]
            println(k, "=", obj[k])
            i = i + 1
        end
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("x=10"));
    assert!(output.contains("y=20"));
    assert!(output.contains("z=30"));
}

#[test]
fn test_array_keys() {
    let code = r#"
        local arr = [100, 200]
        local keys = arr:keys()
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
        local obj = {}
        local keys = obj:keys()
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
        local s = "hello"
        local k = s.keys()
    "#;

    let result = run_captured(code.to_string());
    // Should fail with TypeMismatch or similar because s.keys is nil, and we try to call it?
    // Wait, if s.keys lookup returns nil (because generic fallback checks Object type),
    // then `local k = s.keys()` tries to call nil.
    // VM should error "Attempt to call non-function value".
    assert!(result.is_err());
}

#[test]
fn test_object_static_keys() {
    let code = r#"
        local obj = { first = 1, second = 2 }
        local keys = Object.keys(obj)
        println(keys[0])
        println(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("first"));
    assert!(output.contains("second"));
}
