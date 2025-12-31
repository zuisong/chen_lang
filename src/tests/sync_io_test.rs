use crate::*;

#[test]
fn test_fs_read_write() {
    let code = r#"
        let fs = import "stdlib/fs"
        let io = import "stdlib/io"
        let path = "test_file.txt"
        fs.write_file(path, "Hello Chen Lang")
        let content = fs.read_file(path)
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
        let fs = import "stdlib/fs"
        let process = import "stdlib/process"
        let io = import "stdlib/io"
        let dir = "test_dir"
        process.exec("mkdir " + dir)
        fs.write_file(dir + "/f1.txt", "1")
        fs.write_file(dir + "/f2.txt", "2")
        let entries = fs.read_dir(dir)
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
        let fs = import "stdlib/fs"
        let io = import "stdlib/io"
        let path = "test_exists.txt"
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
        let http = import "stdlib/http"
        let io = import "stdlib/io"
        let resp = http.request("GET", "https://httpbin.org/get")
        io.print("Success")
    "#;
    let result = run_captured(code.to_string());
    dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_process_exec() {
    let code = r#"
        let process = import "stdlib/process"
        let io = import "stdlib/io"
        let res = process.exec("echo hello")
        io.print(res.stdout:trim())
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello");
}
