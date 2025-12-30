use crate::common::run_chen_lang_code;

#[test]
fn test_date() {
    let code = r#"
    let Date = import "stdlib/date"
    let io = import "stdlib/io"
    let d = Date:new()
    io.println(d.__type)
    io.println(d:format('%Y'))
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Date"));
    assert!(output.contains("20"));
}

#[test]
fn test_json() {
    let code = r#"
    let JSON = import "stdlib/json"
    let io = import "stdlib/io"
    let obj = JSON.parse('{"a": 1}')
    io.println(obj.a)
    let s = JSON.stringify(obj)
    io.println(s)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("1"));
    assert!(output.contains("{\"a\":1}"));
}

#[test]
fn test_json_float_precision() {
    let code = r#"
    let JSON = import "stdlib/json"
    let io = import "stdlib/io"
    let data = #{
        simple_add: 0.1 + 2,
        decimal_add: 0.1 + 0.2,
        int_float: 1 + 0.5,
        multiply: 3.14159 * 2
    }
    let json_str = JSON.stringify(data)
    io.println(json_str)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");

    // Verify that floats are serialized with correct precision
    // Should be 2.1, not 2.0999999...
    assert!(output.contains("2.1"), "Expected '2.1' in output, got: {}", output);

    // 0.1 + 0.2 should be 0.3
    assert!(output.contains("0.3"), "Expected '0.3' in output, got: {}", output);

    // 1 + 0.5 should be 1.5
    assert!(output.contains("1.5"), "Expected '1.5' in output, got: {}", output);

    // 3.14159 * 2 should be 6.28318
    assert!(
        output.contains("6.28318"),
        "Expected '6.28318' in output, got: {}",
        output
    );
}

#[test]
fn test_json_roundtrip_precision() {
    let code = r#"
    let JSON = import "stdlib/json"
    let io = import "stdlib/io"
    let original = #{ value: 0.1 + 2 }
    let json_str = JSON.stringify(original)
    let parsed = JSON.parse(json_str)
    io.println(parsed.value)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");

    // After round-trip, should still be 2.1
    assert!(
        output.contains("2.1"),
        "Expected '2.1' after round-trip, got: {}",
        output
    );
}

#[test]
fn test_json_nested_floats() {
    let code = r#"
    let JSON = import "stdlib/json"
    let io = import "stdlib/io"
    let data = #{
        nested: #{
            a: 0.1,
            b: 0.2,
            sum: 0.1 + 0.2
        },
        array: [0.1, 0.2, 0.3]
    }
    let json_str = JSON.stringify(data)
    io.println(json_str)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");

    // Verify nested object floats
    assert!(output.contains("0.1"), "Expected '0.1' in output");
    assert!(output.contains("0.2"), "Expected '0.2' in output");
    assert!(output.contains("0.3"), "Expected '0.3' in output");
}

#[test]
fn test_json_large_precision() {
    let code = r#"
    let JSON = import "stdlib/json"
    let io = import "stdlib/io"
    let data = #{
        pi: 3.141592653589793,
        e: 2.718281828459045,
        small: 0.000000001
    }
    let json_str = JSON.stringify(data)
    io.println(json_str)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");

    // Verify high precision numbers are preserved
    assert!(output.contains("3.141592653589793"), "Expected full precision for pi");
    assert!(output.contains("2.718281828459045"), "Expected full precision for e");
    // Very small numbers may be represented in scientific notation (1e-9)
    assert!(
        output.contains("0.000000001") || output.contains("1e-9") || output.contains("1e-09"),
        "Expected small number in decimal or scientific notation, got: {}",
        output
    );
}

#[test]
fn test_simple_stdlib_import() {
    let source = r#"
        let io = import "stdlib/io"
        io.println("test")
    "#;
    let output = run_chen_lang_code(source).unwrap();
    assert!(output.contains("test"));
}
