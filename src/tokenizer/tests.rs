#![cfg(test)]
#![cfg(feature = "winnow-tokenizer")]

use pretty_assertions::assert_eq;

use crate::tokenizer::{Token, tokenizer_handwritten, winnow::tokenizer as tokenizer_winnow};

#[test]
fn test_handwritten_parity_basic() {
    let code = "let x = 1 + 2";
    let winnow_tokens = tokenizer_winnow(code.to_string()).unwrap();
    let handwritten_tokens = tokenizer_handwritten(code.to_string()).unwrap();
    assert_eq!(winnow_tokens, handwritten_tokens, "Basic assignment parity failed");
}

#[test]
fn test_handwritten_parity_complex() {
    let code = r#"
    def foo(n) {
        if n > 0 {
            return n * foo(n - 1)
        } else {
            return 1
        }
    }
    "#;
    let winnow_tokens = tokenizer_winnow(code.to_string()).unwrap();
    let handwritten_tokens = tokenizer_handwritten(code.to_string()).unwrap();

    // Note: handwritten might handle newlines slightly differently if logic isn't identical,
    // but our updated logic tries to match.
    // Let's verify token types match mostly.
    assert_eq!(winnow_tokens.len(), handwritten_tokens.len());
    for (i, ((wt, wl), (ht, hl))) in winnow_tokens.iter().zip(handwritten_tokens.iter()).enumerate() {
        assert_eq!(wt, ht, "Token mismatch at index {}", i);
        assert_eq!(wl, hl, "Line number mismatch at index {} for token {:?}", i, wt);
    }
}

#[test]
fn test_handwritten_floats() {
    let code = "1.23 0.5 10.0";
    let tokens = tokenizer_handwritten(code.to_string()).unwrap();
    assert_eq!(tokens.len(), 3); // float, float, float (spaces skipped)

    let values: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
    // Check types
    if let Token::Float(_) = values[0] {
    } else {
        panic!("Expected float, got {:?}", values[0]);
    }
    if let Token::Float(_) = values[1] {
    } else {
        panic!("Expected float, got {:?}", values[1]);
    }
    if let Token::Float(_) = values[2] {
    } else {
        panic!("Expected float, got {:?}", values[2]);
    }
}

#[test]
fn test_handwritten_strings() {
    let code = r#""hello" 'world'"#;
    let tokens = tokenizer_handwritten(code.to_string()).unwrap();
    let values: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
    assert_eq!(values[0], Token::String("hello".to_string()));
    assert_eq!(values[1], Token::String("world".to_string()));
}

#[test]
fn test_handwritten_comments() {
    let code = r#"
    # line 1
    let x = 1 # line 2
    "#;
    let tokens = tokenizer_handwritten(code.to_string()).unwrap();
    // Should contain NewLines and logic tokens, but no Comments
    // NewLine, Let, x, Assign, 1, NewLine
    // Depending on how many newlines are in the source string.

    let winnow_tokens = tokenizer_winnow(code.to_string()).unwrap();
    assert_eq!(tokens, winnow_tokens);
}
