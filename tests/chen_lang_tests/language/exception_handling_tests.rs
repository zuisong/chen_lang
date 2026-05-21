use crate::common::run_chen_lang_code;

#[test]
fn test_try_catch_basic() {
    let code = r#"
    try {
        throw "Something went wrong!"
    } catch (error) {
        console.log("Caught error: " + error)
    }
    console.log("Program continues...")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught error: Something went wrong!"));
    assert!(output.contains("Program continues..."));
}

#[test]
fn test_try_catch_with_finally() {
    let code = r#"
    let cleanup_called = false
    
    try {
        console.log("In try block")
        throw "Error occurred"
    } catch (error) {
        console.log("In catch block: " + error)
    } finally {
        console.log("In finally block")
        cleanup_called = true
    }
    
    console.log("Cleanup called: " + cleanup_called)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("In try block"));
    assert!(output.contains("In catch block: Error occurred"));
    assert!(output.contains("In finally block"));
    assert!(output.contains("Cleanup called: true"));
}

#[test]
fn test_try_catch_in_function() {
    let code = r#"
    function divide(a, b) {
        if (b == 0) {
            throw "Division by zero"
        }
        a / b
    }
    
    try {
        let result = divide(10, 2)
        console.log("Result: " + result)
        
        let bad_result = divide(10, 0)
        console.log("This should not print")
    } catch (error) {
        console.log("Caught: " + error)
    }
    
    console.log("Program completed")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Result: 5"));
    assert!(output.contains("Caught: Division by zero"));
    assert!(output.contains("Program completed"));
    assert!(!output.contains("This should not print"));
}

#[test]
fn test_nested_try_catch() {
    let code = r#"
    try {
        console.log("Outer try")
        
        try {
            console.log("Inner try")
            throw "Inner error"
        } catch (inner_error) {
            console.log("Inner catch: " + inner_error)
            throw "Outer error"
        }
        
        console.log("This should not print")
    } catch (outer_error) {
        console.log("Outer catch: " + outer_error)
    }
    
    console.log("Done")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Outer try"));
    assert!(output.contains("Inner try"));
    assert!(output.contains("Inner catch: Inner error"));
    assert!(output.contains("Outer catch: Outer error"));
    assert!(output.contains("Done"));
    assert!(!output.contains("This should not print"));
}

#[test]
fn test_try_catch_without_error_variable() {
    let code = r#"
    try {
        throw "Some error"
    } catch {
        console.log("Error caught (no variable)")
    }
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Error caught (no variable)"));
}

#[test]
fn test_throw_string() {
    let code = r#"
    try {
        throw "Error message"
    } catch (e) {
        console.log("Caught: " + e)
    }
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught: Error message"));
}

#[test]
fn test_throw_number() {
    let code = r#"
    try {
        throw 42
    } catch (e) {
        console.log("Caught: " + e)
    }
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught: 42"));
}

#[test]
fn test_finally_executes_on_success() {
    let code = r#"
    let finally_ran = false
    
    try {
        console.log("Try block")
    } catch (e) {
        console.log("This should not run")
    } finally {
        console.log("Finally block")
        finally_ran = true
    }
    
    console.log("Finally ran: " + finally_ran)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Try block"));
    assert!(output.contains("Finally block"));
    assert!(output.contains("Finally ran: true"));
    assert!(!output.contains("This should not run"));
}

#[test]
fn test_multiple_throws_in_sequence() {
    let code = r#"
    let count = 0
    
    try {
        throw "First"
    } catch (e) {
        console.log("Caught first: " + e)
        count = count + 1
    }
    
    try {
        throw "Second"
    } catch (e) {
        console.log("Caught second: " + e)
        count = count + 1
    }
    
    console.log("Count: " + count)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught first: First"));
    assert!(output.contains("Caught second: Second"));
    assert!(output.contains("Count: 2"));
}
