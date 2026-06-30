use pretty_assertions::assert_matches;

use crate::tokenizer;
use crate::tokenizer::Keyword::{FOR, FUNCTION, LOCAL};
use crate::tokenizer::Operator::Subtract;
use crate::tokenizer::Token::{Identifier, Int, Keyword, Operator, String};

#[test]
#[cfg(feature = "winnow-tokenizer")]
fn test() {
    use crate::tokenizer::winnow::parse_with_winnow;
    assert_matches!(parse_with_winnow("-1"), Ok(("1", Operator(Subtract))));
    assert_matches!(parse_with_winnow("-a"), Ok(("a", Operator(Subtract))));
    assert_matches!(parse_with_winnow("10a"), Ok(("a", Int(10))));
    assert_matches!(parse_with_winnow("\"aaaa\""), Ok(("", String(ref a))) if a == "aaaa");
    assert_matches!(parse_with_winnow("'aaaa'"),Ok(("", String(ref a))) if a == "aaaa");
    assert_matches!(parse_with_winnow("''"), Ok(("", String(ref a))) if a.is_empty());
}

#[test]
fn test_parse_keyword() {
    assert_eq!(
        tokenizer::tokenizer("println".to_string())
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect::<Vec<_>>(),
        vec![Identifier("println".to_string())]
    )
}

#[test]
fn test_parse_for() {
    assert_eq!(
        tokenizer::tokenizer("for".to_string())
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect::<Vec<_>>(),
        vec![Keyword(FOR)]
    )
}

#[test]
fn parse_code() {
    let code = "local i = 0".to_string();
    let tokens: Vec<_> = tokenizer::tokenizer(code)
        .unwrap()
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert!(tokens.contains(&Keyword(LOCAL)), "Should contain LOCAL keyword");
}

#[test]
fn parse_code2() {
    let code = "function aaa(n) return n + 1 end".to_string();
    let tokens: Vec<_> = tokenizer::tokenizer(code)
        .unwrap()
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert!(tokens.contains(&Keyword(FUNCTION)), "Should contain FUNCTION keyword");
}

#[test]
fn test_handwritten_floats() {
    use crate::tokenizer::tokenizer_handwritten;
    let code = "1.25".to_string();
    let tokens = tokenizer_handwritten(code).unwrap();
    assert_matches!(tokens[0].0, crate::tokenizer::Token::Float(_));
}
