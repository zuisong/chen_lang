use crate::*;

#[test]
fn test_fs_read_write() {
    let code = r#"
        let fs = Chen.fs
        let path = "test_file.txt"
        fs.writeTextFile(path, "Hello Chen Lang")
        let content = fs.readTextFile(path)
        console.print(content)
        fs.remove(path)
    "#;

    let result = run_captured(code.to_string());
    assert!(result.is_ok(), "FS operations should work: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "Hello Chen Lang");
}

#[test]
fn test_fs_read_dir() {
    let code = r#"
        let fs = Chen.fs
        let process = Chen.process
        let dir = "test_dir"
        process.exec("mkdir " + dir)
        fs.writeTextFile(dir + "/f1.txt", "1")
        fs.writeTextFile(dir + "/f2.txt", "2")
        let entries = fs.readDir(dir)
        console.log(entries.length)
        fs.remove(dir)
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}

#[test]
fn test_fs_exists() {
    let code = r#"
        let fs = Chen.fs
        let path = "test_exists.txt"
        console.log(fs.exists(path))
        fs.writeTextFile(path, "exists")
        console.log(fs.exists(path))
        fs.remove(path)
        console.log(fs.exists(path))
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
        let request = Chen.http.request
        let resp = request("GET", "https://httpbin.org/get")
        console.print("Success")
    "#;
    let result = run_captured(code.to_string());
    dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_process_exec() {
    let code = r#"
        let process = Chen.process
        let res = process.exec("echo hello")
        console.print(res.stdout.trim())
    "#;
    let result = run_captured(code.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello");
}
