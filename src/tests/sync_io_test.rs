use crate::*;

#[test]
fn test_fs_read_write() {
    let code = r#"
        local fs = require("stdlib/fs")
        local io = require("stdlib/io")
        local path = "test_file.txt"
        fs.write_file(path, "Hello Chen Lang")
        local content = fs.read_file(path)
        io.print(content)
        fs.remove(path)
    "#;

    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "FS operations should work: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "Hello Chen Lang");
}

#[test]
fn test_fs_read_dir() {
    let code = r#"
        local fs = require("stdlib/fs")
        local process = require("stdlib/process")
        local io = require("stdlib/io")
        local dir = "test_dir"
        process.exec("mkdir " .. dir)
        fs.write_file(dir .. "/f1.txt", "1")
        fs.write_file(dir .. "/f2.txt", "2")
        local entries = fs.read_dir(dir)
        io.println(entries:len())
        fs.remove(dir)
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}

#[test]
fn test_fs_exists() {
    let code = r#"
        local fs = require("stdlib/fs")
        local io = require("stdlib/io")
        local path = "test_exists.txt"
        io.println(fs.exists(path))
        fs.write_file(path, "exists")
        io.println(fs.exists(path))
        fs.remove(path)
        io.println(fs.exists(path))
    "#;

    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    let output = result.unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "false");
    assert_eq!(lines[1], "true");
    assert_eq!(lines[2], "false");
}

#[test]
#[ignore = "require network"]
#[cfg(feature = "http")]
fn test_http_get() {
    let code = r#"
        local http = require("stdlib/http")
        local io = require("stdlib/io")
        local resp = http.request("GET", "https://httpbin.org/get")
        io.print("Success")
    "#;
    let result = run_captured(code.to_string());
    dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_process_exec() {
    let code = r#"
        local process = require("stdlib/process")
        local io = require("stdlib/io")
        local res = process.exec("echo hello")
        io.print(res.stdout:trim())
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello");
}
