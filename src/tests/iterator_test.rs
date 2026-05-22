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
fn test_sync_array_iterator() {
    let code = r#"
    let arr = [10, 20, 30]
    let sum = 0
    for (let x of arr) {
        sum = sum + x
    }
    return sum
    "#;
    assert_eq!(run_code(code), "60");
}

#[test]
fn test_sync_string_iterator() {
    let code = r#"
    let s = "abc"
    let res = ""
    for (let x of s) {
        res = res + x + "-"
    }
    return res
    "#;
    assert_eq!(run_code(code), "a-b-c-");
}

#[test]
fn test_sync_object_iterator() {
    let code = r#"
    let obj = { a: 1, b: 2, c: 3 }
    let sum = 0
    for (let x of obj) {
        sum = sum + x
    }
    return sum
    "#;
    assert_eq!(run_code(code), "6");
}

#[test]
fn test_custom_sync_iterator() {
    let code = r#"
    let custom = {
        [Symbol.iterator]: function() {
            let i = 0
            return {
                next: function() {
                    if (i < 3) {
                        i = i + 1
                        return { value: i, done: false }
                    }
                    return { value: null, done: true }
                }
            }
        }
    }
    let sum = 0
    for (let x of custom) {
        sum = sum + x
    }
    return sum
    "#;
    assert_eq!(run_code(code), "6");
}

#[test]
fn test_custom_async_iterator() {
    let code = r#"
    let custom_async = {
        [Symbol.asyncIterator]: function() {
            let i = 0
            return {
                next: async function() {
                    if (i < 3) {
                        i = i + 1
                        return { value: i, done: false }
                    }
                    return { value: null, done: true }
                }
            }
        }
    }
    async function run() {
        let sum = 0
        for await (let x of custom_async) {
            sum = sum + x
        }
        return sum
    }
    let p = run()
    return await p
    "#;
    assert_eq!(run_code(code), "6");
}

#[test]
fn test_async_iterator_fallback_to_sync() {
    let code = r#"
    let custom = {
        [Symbol.iterator]: function() {
            let i = 0
            return {
                next: function() {
                    if (i < 3) {
                        i = i + 1
                        return { value: i, done: false }
                    }
                    return { value: null, done: true }
                }
            }
        }
    }
    async function run() {
        let sum = 0
        for await (let x of custom) {
            sum = sum + x
        }
        return sum
    }
    let p = run()
    return await p
    "#;
    assert_eq!(run_code(code), "6");
}
