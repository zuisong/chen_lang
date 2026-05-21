use crate::common::run_chen_lang_code;

#[test]
fn test_function_scope_isolation() {
    let code = r#"
function func() {
    let local_var = "local_value"
    return "test"
}

let result = "abcd"
func()
console.log(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("abcd"));
}

#[test]
fn test_function_variable_not_leaked() {
    let code = r#"
function func() {
    let secret = "should_not_be_visible"
    return "done"
}

func()
console.log(secret)
"#;

    let result = run_chen_lang_code(code);
    assert!(result.is_err());
}

#[test]
fn test_if_statement_scope() {
    let code = r#"
let x = "global"
if (true) {
    let x = "local"
    console.log(x)
}
console.log(x)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("local"));
    assert!(lines[1].contains("global"));
}

#[test]
fn test_for_loop_scope() {
    let code = r#"
let i = 1
while (i <= 3) {
    let temp = i
    console.log(temp)
    i = i + 1
}
console.log(i)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].trim(), "1");
    assert_eq!(lines[1].trim(), "2");
    assert_eq!(lines[2].trim(), "3");
    assert_eq!(lines[3].trim(), "4");
}

#[test]
fn test_simple_block_assignment() {
    let code = r#"
    let x = function() {
        let a = 10
        let b = 20
        return a + b
    }()
    if (x == 30) {
        console.log("Test 1 Passed")
    } else {
        console.log("Test 1 Failed: Expected 30, got " + x)
    }
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
}

#[test]
fn test_nested_blocks() {
    let code = r#"
    let y = function() {
        let c = 5
        return function() {
            let d = 10
            return c + d
        }()
    }()
    if (y == 15) {
        console.log("Test 2 Passed")
    } else {
        console.log("Test 2 Failed: Expected 15, got " + y)
    }
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 2 Passed"));
}

#[test]
fn test_block_with_if_else() {
    let code = r#"
    let z = function() {
        let e = 100
        return if (e > 50) {
            1
        } else {
            0
        }
    }()
    console.log("Test 3 (If statement): " + z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 3 (If statement):"));
}

#[test]
fn test_block_ending_with_assignment() {
    let code = r#"
    let w = function() {
        let f = 1
        f = f + 1
    }()
    console.log("Test 4 (Assignment): " + w)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 4 (Assignment):"));
}

#[test]
fn test_empty_block() {
    let code = r#"
    let v = {}
    console.log("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 5 (Empty):"));
}

#[test]
fn test_block_value_simple() {
    let code = r#"
    let result = function() {
        return 5 + 10
    }()
    console.log(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_block_value_with_variables() {
    let code = r#"
    let result = function() {
        let x = 10
        let y = 20
        return x + y
    }()
    console.log(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "30");
}

#[test]
fn test_all_scope_value_tests() {
    let code = r#"
let x = function() {
    let a = 10
    let b = 20
    return a + b
}()
if (x == 30) {
    console.log("Test 1 Passed")
} else {
    console.log("Test 1 Failed: Expected 30, got " + x)
}

let y = function() {
    let c = 5
    return function() {
        let d = 10
        return c + d
    }()
}()
if (y == 15) {
    console.log("Test 2 Passed")
} else {
    console.log("Test 2 Failed: Expected 15, got " + y)
}

let z = function() {
    let e = 100
    return if (e > 50) {
        1
    } else {
        0
    }
}()
console.log("Test 3 (If statement): " + z) 

let w = function() {
    let f = 1
    f = f + 1
}()
console.log("Test 4 (Assignment): " + w)

let v = {}
console.log("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
    assert!(output.contains("Test 2 Passed"));
    assert!(output.contains("Test 3 (If statement):"));
    assert!(output.contains("Test 4 (Assignment):"));
    assert!(output.contains("Test 5 (Empty):"));
}
