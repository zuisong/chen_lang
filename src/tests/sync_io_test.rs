use crate::*;

#[test]
fn test_fs_read_write() {
    let code = r#"
        let path = "test_file.txt"
        fs.write_file(path, "Hello Chen Lang")
        let content = fs.read_file(path)
        print(content)
        fs.remove(path)
    "#;

    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "FS operations should work: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "Hello Chen Lang");
}

#[test]
fn test_fs_read_dir() {
    let code = r#"
        let dir = "test_dir"
        process.exec("mkdir " + dir)
        fs.write_file(dir + "/f1.txt", "1")
        fs.write_file(dir + "/f2.txt", "2")
        let entries = fs.read_dir(dir)
        println(entries.len())
        fs.remove(dir)
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}

#[test]
fn test_fs_exists() {
    let code = r#"
        let path = "test_exists.txt"
        println(fs.exists(path))
        fs.write_file(path, "exists")
        println(fs.exists(path))
        fs.remove(path)
        println(fs.exists(path))
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
#[cfg(feature = "http")]
#[ignore] // Requires internet
fn test_http_get() {
    let code = r#"
        let resp = http.get("https://httpbin.org/get")
        print("Success")
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
}

#[test]
fn test_process_exec() {
    let code = r#"
        let res = process.exec("echo hello")
        print(res.stdout.trim())
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello");
}
