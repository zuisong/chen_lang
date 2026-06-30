use crate::common::{run_chen_lang_code, run_chen_lang_code_with_setup};

#[test]
fn test_http_get() {
    let mut server = mockito::Server::new();
    let url = server.url();
    let mock = server
        .mock("GET", "/hello")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("world")
        .create();

    let code = format!(
        r#"
    local http = require("stdlib/http")
    local io = require("stdlib/io")
    local res = http.request("GET", "{}/hello")
    io.write(res.body)
    "#,
        url
    );

    let output = run_chen_lang_code(&code).expect("Execution failed");
    assert_eq!(output, "world");
    mock.assert();
}

#[test]
fn test_http_post() {
    let mut server = mockito::Server::new();
    let url = server.url();
    let mock = server
        .mock("POST", "/echo")
        .match_body("hello")
        .with_status(200)
        .with_body("received")
        .create();

    let code = format!(
        r#"
    local http = require("stdlib/http")
    local io = require("stdlib/io")
    local res = http.request("POST", "{}/echo", "hello")
    io.write(res.body)
    "#,
        url
    );

    let output = run_chen_lang_code(&code).expect("Execution failed");
    assert_eq!(output, "received");
    mock.assert();
}

#[test]
fn test_http_request_method() {
    let mut server = mockito::Server::new();
    let url = server.url();
    let mock = server
        .mock("PUT", "/update")
        .match_body("new_data")
        .with_status(201)
        .with_header("x-custom-header", "custom-value")
        .with_body("updated")
        .create();

    let code = r#"
    local http = require("stdlib/http")
    local io = require("stdlib/io")
    local res = http.request("PUT", url + "/update", "new_data")
    io.write(res.status)
    io.write(res.headers['x-custom-header'])
    io.write(res.body)
    "#;

    let output = run_chen_lang_code_with_setup(code, |vm| {
        vm.add_var_str("url", &url);
    })
    .expect("Execution failed");

    assert!(output.contains("201"));
    assert!(output.contains("custom-value"));
    assert!(output.contains("updated"));
    mock.assert();
}

#[test]
fn test_http_request_with_headers() {
    let mut server = mockito::Server::new();
    let url = server.url();
    let mock = server
        .mock("GET", "/headers")
        .match_header("X-Auth", "secret123")
        .with_status(200)
        .with_body("authorized")
        .create();

    let code = format!(
        r#"
    local http = require("stdlib/http")
    local io = require("stdlib/io")
    local headers = {{}}
    headers["X-Auth"] = "secret123"
    local res = http.request("GET", "{}/headers", nil, headers)
    io.write(res.body)
    "#,
        url
    );

    let output = run_chen_lang_code(&code).expect("Execution failed");
    assert_eq!(output, "authorized");
    mock.assert();
}
