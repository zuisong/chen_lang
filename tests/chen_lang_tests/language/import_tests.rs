use chen_lang::run_captured;

#[test]
fn test_import_stdlib_io_json() {
    let source = r#"
        local io = require("stdlib/io")
        local JSON = require("stdlib/json")

        local data = { name = "Chen", version = 0.1 }
        local json_str = JSON.stringify(data)
        io.write("JSON: " + json_str)
    "#
    .to_string();

    let output = run_captured(source).unwrap();
    assert!(output.contains("JSON: {\"name\":\"Chen\",\"version\":0.1}"));
}

#[test]
fn test_no_import_fail() {
    let source = r#"
        -- No import for json
        local data = { name = "Chen" }
        local json_str = JSON.stringify(data)
    "#
    .to_string();

    let result = run_captured(source);
    assert!(result.is_err());
}

#[test]
fn test_import_stdlib_date() {
    let source = r#"
        local io = require("stdlib/io")
        local Date = require("stdlib/date")
        local now = Date:new()
        -- Just check if it's not nil and works
        if now ~= nil then
            io.write("Date ok")
        end
    "#
    .to_string();
    let output = run_captured(source).unwrap();
    assert!(output.contains("Date ok"));
}
