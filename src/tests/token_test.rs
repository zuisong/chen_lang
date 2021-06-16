use crate::token;
use crate::token::{Keyword, StdFunction, Token};

#[test]
fn test_parse_keyword() {
    assert_eq!(
        token::tokenlizer("println".to_string()).unwrap(),
        vec![Token::StdFunction(StdFunction::Print(true))]
    )
}

#[test]
fn test_parse_for() {
    assert_eq!(
        token::tokenlizer("for".to_string()).unwrap(),
        vec![Token::Keyword(Keyword::FOR)]
    )
}
