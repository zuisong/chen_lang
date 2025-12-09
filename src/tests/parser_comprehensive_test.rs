use crate::expression::{Expression, Literal, Statement};
use crate::parser;
use crate::token::Operator;
use crate::value::Value;

// Helper to parse a string into statements
fn parse_code(code: &str) -> Result<Vec<Statement>, crate::parser::ParserError> {
    parser::parse_from_source(code)
}

// Helper to parse a single expression string
fn parse_expr_str(code: &str) -> Expression {
    let stmts = parse_code(code).expect("Parsing failed");
    assert_eq!(stmts.len(), 1, "Expected exactly one statement");
    match &stmts[0] {
        Statement::Expression(expr) => expr.clone(),
        _ => panic!("Expected an expression statement"),
    }
}

#[test]
fn test_operator_precedence_mul_add() {
    // 1 + 2 * 3 should be 1 + (2 * 3)
    let expr = parse_expr_str("1 + 2 * 3");

    if let Expression::BinaryOperation(bin_op) = expr {
        assert_eq!(bin_op.operator, Operator::Add);
        match (*bin_op.left, *bin_op.right) {
            (
                Expression::Literal(Literal::Value(Value::Int(1))),
                Expression::BinaryOperation(right_op),
            ) => {
                assert_eq!(right_op.operator, Operator::Multiply);
                assert_eq!(
                    *right_op.left,
                    Expression::Literal(Literal::Value(Value::Int(2)))
                );
                assert_eq!(
                    *right_op.right,
                    Expression::Literal(Literal::Value(Value::Int(3)))
                );
            }
            _ => panic!("AST structure incorrect for 1 + 2 * 3"),
        }
    } else {
        panic!("Expected BinaryOperation");
    }
}

#[test]
fn test_operator_precedence_paren() {
    // (1 + 2) * 3 should be (1 + 2) * 3
    let expr = parse_expr_str("(1 + 2) * 3");

    if let Expression::BinaryOperation(bin_op) = expr {
        assert_eq!(bin_op.operator, Operator::Multiply);
        match (*bin_op.left, *bin_op.right) {
            (
                Expression::BinaryOperation(left_op),
                Expression::Literal(Literal::Value(Value::Int(3))),
            ) => {
                assert_eq!(left_op.operator, Operator::Add);
                assert_eq!(
                    *left_op.left,
                    Expression::Literal(Literal::Value(Value::Int(1)))
                );
                assert_eq!(
                    *left_op.right,
                    Expression::Literal(Literal::Value(Value::Int(2)))
                );
            }
            _ => panic!("AST structure incorrect for (1 + 2) * 3"),
        }
    } else {
        panic!("Expected BinaryOperation");
    }
}

#[test]
fn test_logical_precedence() {
    // true || false && true  =>  true || (false && true)
    let expr = parse_expr_str("true || false && true");

    if let Expression::BinaryOperation(bin_op) = expr {
        assert_eq!(bin_op.operator, Operator::Or);
        // left is true
        if let Expression::Literal(Literal::Value(Value::Bool(true))) = *bin_op.left {
        } else {
            panic!("Left should be true");
        }

        // right is false && true
        if let Expression::BinaryOperation(right_op) = *bin_op.right {
            assert_eq!(right_op.operator, Operator::And);
        } else {
            panic!("Right should be AND operation");
        }
    } else {
        panic!("Expected BinaryOperation OR");
    }
}

#[test]
fn test_if_expression() {
    let expr = parse_expr_str("if true { 1 } else { 0 }");

    if let Expression::If(if_expr) = expr {
        if let Expression::Literal(Literal::Value(Value::Bool(true))) = *if_expr.test {
        } else {
            panic!("Condition failed");
        }
        assert_eq!(if_expr.body.len(), 1);
        assert_eq!(if_expr.else_body.len(), 1);
    } else {
        panic!("Expected If Expression");
    }
}

#[test]
fn test_syntax_error_missing_brace() {
    let code = "if true { 1 "; // Missing closing brace
    let result = parse_code(code);
    assert!(result.is_err());
    let err = result.err().unwrap();
    // We verify it is indeed a parse error, specific message might vary
    println!("Caught expected error: {:?}", err);
}

#[test]
fn test_syntax_error_unexpected_token() {
    let code = "let x = +"; // unexpected operator
    let result = parse_code(code);
    assert!(result.is_err());
}

#[test]
fn test_function_call_no_args() {
    let expr = parse_expr_str("myFunc()");
    if let Expression::FunctionCall(call) = expr {
        if let Expression::Identifier(name) = *call.callee {
            assert_eq!(name, "myFunc");
        } else {
            panic!("Expected Identifier callee");
        }
        assert!(call.arguments.is_empty());
    } else {
        panic!("Expected FunctionCall");
    }
}

#[test]
fn test_function_call_with_args() {
    let expr = parse_expr_str("add(1, 2)");
    if let Expression::FunctionCall(call) = expr {
        if let Expression::Identifier(name) = *call.callee {
            assert_eq!(name, "add");
        } else {
            panic!("Expected Identifier callee");
        }
        assert_eq!(call.arguments.len(), 2);
    } else {
        panic!("Expected FunctionCall");
    }
}

#[test]
fn test_object_literal() {
    // #{ x: 1, y: 2 }
    let expr = parse_expr_str("#{ x: 1, y: 2 }");
    if let Expression::ObjectLiteral(fields) = expr {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "x");
        assert_eq!(fields[1].0, "y");
    } else {
        panic!("Expected ObjectLiteral");
    }
}

#[test]
fn test_get_field() {
    let expr = parse_expr_str("obj.x");
    if let Expression::GetField { object, field } = expr {
        assert_eq!(field, "x");
        if let Expression::Identifier(name) = *object {
            assert_eq!(name, "obj");
        } else {
            panic!("Object base should be identifier");
        }
    } else {
        panic!("Expected GetField");
    }
}

#[test]
fn test_set_field() {
    let stmts = parse_code("obj.x = 1").unwrap();
    if let Statement::SetField { object, field, value: _ } = &stmts[0] {
        if let Expression::Identifier(name) = object {
            assert_eq!(name, "obj");
        } else {
            panic!("Object base should be identifier");
        }
        assert_eq!(field, "x");
    } else {
        panic!("Expected SetField statement");
    }
}

#[test]
fn test_index_access() {
    let expr = parse_expr_str("arr[0]");
    if let Expression::Index { object, index: _ } = expr {
        if let Expression::Identifier(name) = *object {
            assert_eq!(name, "arr");
        }
    } else {
        panic!("Expected Index");
    }
}
