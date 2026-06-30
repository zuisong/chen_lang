use crate::common::run_chen_lang_code;

#[test]
fn test_try_catch_basic() {
    let code = r#"
    try
        error("Something went wrong!")
    catch e
        print("Caught error: " + e)
    end
    print("Program continues...")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught error: Something went wrong!"));
    assert!(output.contains("Program continues..."));
}

#[test]
fn test_try_catch_with_finally() {
    let code = r#"
    local cleanup_called = false
    
    try
        print("In try block")
        error("Error occurred")
    catch e
        print("In catch block: " + e)
    finally
        print("In finally block")
        cleanup_called = true
    end
    
    print("Cleanup called: " + cleanup_called)
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
    function divide(a, b)
        if b == 0 then
            error("Division by zero")
        end
        return a / b
    end
    
    try
        local result = divide(10, 2)
        print("Result: " + result)
        
        local bad_result = divide(10, 0)
        print("This should not print")
    catch e
        print("Caught: " + e)
    end
    
    print("Program completed")
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
    try
        print("Outer try")
        
        try
            print("Inner try")
            error("Inner error")
        catch inner_error
            print("Inner catch: " + inner_error)
            error("Outer error")
        end
        
        print("This should not print")
    catch outer_error
        print("Outer catch: " + outer_error)
    end
    
    print("Done")
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
    try
        error("Some error")
    catch
        print("Error caught (no variable)")
    end
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Error caught (no variable)"));
}

#[test]
fn test_throw_string() {
    let code = r#"
    try
        error("Error message")
    catch e
        print("Caught: " + e)
    end
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught: Error message"));
}

#[test]
fn test_throw_number() {
    let code = r#"
    try
        error(42)
    catch e
        print("Caught: " + e)
    end
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught: 42"));
}

#[test]
fn test_finally_executes_on_success() {
    let code = r#"
    local finally_ran = false
    
    try
        print("Try block")
    catch e
        print("This should not run")
    finally
        print("Finally block")
        finally_ran = true
    end
    
    print("Finally ran: " + finally_ran)
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
    local count = 0
    
    try
        error("First")
    catch e
        print("Caught first: " + e)
        count = count + 1
    end
    
    try
        error("Second")
    catch e
        print("Caught second: " + e)
        count = count + 1
    end
    
    print("Count: " + count)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("Caught first: First"));
    assert!(output.contains("Caught second: Second"));
    assert!(output.contains("Count: 2"));
}
