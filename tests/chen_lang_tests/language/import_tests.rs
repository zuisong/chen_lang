use chen_lang::run_captured;

#[test]
fn test_import_stdlib_io_json() {
    let source = r#"
        let io = import("stdlib/io")
        let JSON = import("stdlib/json")

        let data = ${ name: "Chen", version: 0.1 }
        let json_str = JSON.stringify(data)
        io.println("JSON: " + json_str)
    "#
    .to_string();

    let output = run_captured(source).unwrap();
    assert!(output.contains("JSON: {\"name\":\"Chen\",\"version\":0.1}"));
}

#[test]
fn test_no_import_fail() {
    let source = r#"
        # No import for json
        let data = ${ name: "Chen" }
        let json_str = JSON.stringify(data)
    "#
    .to_string();

    let result = run_captured(source);
    assert!(result.is_err());
}

#[test]
fn test_import_stdlib_date() {
    let source = r#"
        let io = import("stdlib/io")
        let Date = import("stdlib/date")
        let now = Date:new()
        # Just check if it's not null and works
        if now != null {
            io.print("Date ok")
        }
    "#
    .to_string();
    let output = run_captured(source).unwrap();
    assert!(output.contains("Date ok"));
}
