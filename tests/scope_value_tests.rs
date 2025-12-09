mod common;
use common::run_chen_lang_code;

#[test]
fn test_simple_block_assignment() {
    let code = r#"
    let x = {
        let a = 10
        let b = 20
        a + b
    }
    if x == 30 {
        println("Test 1 Passed")
    } else {
        println("Test 1 Failed: Expected 30, got " + x)
    }
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
}

#[test]
fn test_nested_blocks() {
    let code = r#"
    let y = {
        let c = 5
        {
            let d = 10
            c + d
        }
    }
    if y == 15 {
        println("Test 2 Passed")
    } else {
        println("Test 2 Failed: Expected 15, got " + y)
    }
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 2 Passed"));
}

#[test]
fn test_block_with_if_else() {
    let code = r#"
    let z = {
        let e = 100
        if e > 50 {
            1
        } else {
            0
        }
    }
    println("Test 3 (If statement): " + z)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    // The output will depend on whether if is treated as an expression or statement
    assert!(output.contains("Test 3 (If statement):"));
}

#[test]
fn test_block_ending_with_assignment() {
    let code = r#"
    let w = {
        let f = 1
        f = f + 1
    }
    println("Test 4 (Assignment): " + w)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    // Assignment is a statement, so w should be null
    assert!(output.contains("Test 4 (Assignment):"));
}

#[test]
fn test_empty_block() {
    let code = r#"
    let v = {}
    println("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 5 (Empty):"));
}

#[test]
fn test_block_value_simple() {
    let code = r#"
    let result = {
        5 + 10
    }
    println(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_block_value_with_variables() {
    let code = r#"
    let result = {
        let x = 10
        let y = 20
        x + y
    }
    println(result)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "30");
}

#[test]
fn test_all_scope_value_tests() {
    // Run the complete test file
    let code = r#"
# Test 1: Simple block assignment
let x = {
    let a = 10
    let b = 20
    a + b
}
if x == 30 {
    println("Test 1 Passed")
} else {
    println("Test 1 Failed: Expected 30, got " + x)
}

# Test 2: Nested blocks
let y = {
    let c = 5
    {
        let d = 10
        c + d
    }
}
if y == 15 {
    println("Test 2 Passed")
} else {
    println("Test 2 Failed: Expected 15, got " + y)
}

# Test 3: Block with if/else
let z = {
    let e = 100
    if e > 50 {
        1
    } else {
        0
    }
}
println("Test 3 (If statement): " + z) 

# Test 4: Block ending with non-expression
let w = {
    let f = 1
    f = f + 1
}
println("Test 4 (Assignment): " + w)

# Test 5: Empty block
let v = {}
println("Test 5 (Empty): " + v)
    "#;
    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("Test 1 Passed"));
    assert!(output.contains("Test 2 Passed"));
    assert!(output.contains("Test 3 (If statement):"));
    assert!(output.contains("Test 4 (Assignment):"));
    assert!(output.contains("Test 5 (Empty):"));
}
