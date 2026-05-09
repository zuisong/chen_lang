use crate::expression::{Statement, TypeAnnotation};
use crate::parser;

#[test]
fn parse() {
    let code: String = r#"
 let i = 0
 for i<100{

     if i%2 == 0{
         println(i + " 是偶数")
     }else{
         println(i + " 是奇数")
     }
     i = i+1
 }
 "#
    .to_string();

    let res = parser::parse_from_source(&code);

    dbg!(&res);

    assert!(res.is_ok());
    let statements = res.unwrap();
    assert!(!statements.is_empty());

    // Detailed AST structure verification is covered by parser_comprehensive_test.rs
    // This test ensures that a larger block of code parses without error.
}

#[test]
fn parse_optional_type_annotations() {
    let code = "let x: int = 10\ndef add(a: int, b: int) -> int { return a + b }";
    let statements = parser::parse_from_source(code).unwrap();

    match &statements[0] {
        Statement::Local(local) => assert_eq!(local.type_annotation, Some(TypeAnnotation::Int)),
        other => panic!("expected local declaration, got {other:?}"),
    }

    match &statements[1] {
        Statement::FunctionDeclaration(function) => {
            assert_eq!(function.return_type, Some(TypeAnnotation::Int));
            assert_eq!(function.parameters[0].type_annotation, Some(TypeAnnotation::Int));
            assert_eq!(function.parameters[1].type_annotation, Some(TypeAnnotation::Int));
        }
        other => panic!("expected function declaration, got {other:?}"),
    }
}

#[test]
fn parse_unannotated_code_still_works() {
    let code = "let x = 10\ndef id(a) { return a }";
    let statements = parser::parse_from_source(code).unwrap();

    match &statements[0] {
        Statement::Local(local) => assert_eq!(local.type_annotation, None),
        other => panic!("expected local declaration, got {other:?}"),
    }

    match &statements[1] {
        Statement::FunctionDeclaration(function) => {
            assert_eq!(function.return_type, None);
            assert_eq!(function.parameters[0].type_annotation, None);
        }
        other => panic!("expected function declaration, got {other:?}"),
    }
}
