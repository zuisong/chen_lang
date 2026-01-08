use mockito::Server;

use crate::compiler::compile;
use crate::parser::parse_from_source;
use crate::vm::VM;

fn run_code(code: &str) -> String {
    let ast = parse_from_source(code).unwrap();
    let program = compile(&code.chars().collect::<Vec<_>>(), ast);
    let mut vm = VM::new();
    match vm.execute(&program) {
        Ok(v) => v.to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

#[test]
fn test_http_get_async() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/hello")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("world")
        .create();

    let url = server.url();
    let code = format!(
        r#"
    let http = import "stdlib/http"
    let url = "{}/hello"
    let resp = http.request("GET", url)
    return resp.body
    "#,
        url
    );

    let result = run_code(&code);
    assert_eq!(result, "world");
    mock.assert();
}

#[test]
fn test_http_get_json_async() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/data")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "ok"}"#)
        .create();

    let url = server.url();
    let code = format!(
        r#"
    let http = import "stdlib/http"
    let json = import "stdlib/json"
    let url = "{}/data"
    let resp = http.request("GET", url)
    let data = json.parse(resp.body)
    return data.status
    "#,
        url
    );

    let result = run_code(&code);
    assert_eq!(result, "ok");
    mock.assert();
}

#[test]
fn test_http_request_async_error_propagates() {
    let code = r#"
    let http = import "stdlib/http"
    try {
        http.request("BAD METHOD", "http://example.com")
        return "NO_ERROR"
    } catch err {
        return "CAUGHT: " + err
    }
    "#;

    let result = run_code(code);
    assert!(result.contains("CAUGHT:"));
    assert!(result.contains("HTTP invalid method"));
}
