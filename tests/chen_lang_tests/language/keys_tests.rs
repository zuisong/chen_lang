use chen_lang::run_captured;

#[test]
fn test_object_keys_basic() {
    let code = r#"
        let obj = { a: 1, b: 2 }
        console.log("obj: " + obj)
        console.log("Object: " + Object)
        console.log("keys_fn: " + Object.keys)
        let keys = Object.keys(obj)
        console.log("keys: " + keys)
        console.log("keys_len: " + keys.length)
        console.log(keys.length)
        console.log(keys[0])
        console.log(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    println!("DEBUG OUTPUT:\n{}", output);
    assert!(output.contains("2"));
    assert!(output.contains("a"));
    assert!(output.contains("b"));
}

#[test]
fn test_object_keys_iteration() {
    let code = r#"
        let obj = { x: 10, y: 20, z: 30 }
        let keys = Object.keys(obj)
        let i = 0
        while (i < keys.length) {
            let k = keys[i]
            console.log(k + "=" + obj[k])
            i = i + 1
        }
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("x=10"));
    assert!(output.contains("y=20"));
    assert!(output.contains("z=30"));
}

#[test]
fn test_array_keys() {
    let code = r#"
        let arr = [100, 200]
        let keys = Object.keys(arr)
        console.log(keys.length)
        console.log(keys[0])
        console.log(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("2"));
    assert!(output.contains("0"));
    assert!(output.contains("1"));
}

#[test]
fn test_empty_object_keys() {
    let code = r#"
        let obj = {}
        let keys = Object.keys(obj)
        console.log(keys.length)
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("0"));
}

#[test]
fn test_keys_on_non_object() {
    let code = r#"
        let s = "hello"
        let k = Object.keys(s)
    "#;

    let result = run_captured(code.to_string());
    assert!(result.is_err());
}

#[test]
fn test_object_static_keys() {
    let code = r#"
        let obj = { first: 1, second: 2 }
        let keys = Object.keys(obj)
        console.log(keys[0])
        console.log(keys[1])
    "#;

    let output = run_captured(code.to_string()).unwrap();
    assert!(output.contains("first"));
    assert!(output.contains("second"));
}
