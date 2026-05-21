use crate::expression::{Statement, TypeAnnotation};
use crate::parser;

#[test]
fn parse() {
    let code: String = r#"
 let i = 0
 for (let i of [1,2,3]) {
     if (i % 2 == 0) {
         console.log(i + " 是偶数")
     } else {
         console.log(i + " 是奇数")
     }
 }
 "#
    .to_string();

    let res = parser::parse_from_source(&code);

    dbg!(&res);

    assert!(res.is_ok());
    let statements = res.unwrap();
    assert!(!statements.is_empty());
}

#[test]
fn parse_optional_type_annotations() {
    let code = "let x: int = 10\nfunction add(a: int, b: int) -> int { return a + b }";
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
    let code = "let x = 10\nfunction id(a) { return a }";
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

#[test]
fn parse_phase2_type_annotations() {
    let code = r#"
let arr: Array<int> = [1, 2, 3]
let opt: Option<string> = null
function process(val: int | float) -> int | float { return val }
"#;
    let statements = parser::parse_from_source(code).unwrap();

    match &statements[0] {
        Statement::Local(local) => assert_eq!(
            local.type_annotation,
            Some(TypeAnnotation::Generic {
                name: "Array".to_string(),
                arguments: vec![TypeAnnotation::Int],
            })
        ),
        other => panic!("expected local declaration, got {other:?}"),
    }

    match &statements[2] {
        Statement::FunctionDeclaration(function) => {
            assert_eq!(
                function.parameters[0].type_annotation,
                Some(TypeAnnotation::Union(vec![TypeAnnotation::Int, TypeAnnotation::Float]))
            );
            assert_eq!(
                function.return_type,
                Some(TypeAnnotation::Union(vec![TypeAnnotation::Int, TypeAnnotation::Float]))
            );
        }
        other => panic!("expected function declaration, got {other:?}"),
    }
}

#[test]
fn parse_comment_slash_slash() {
    let code = r#"
        // This is a comment
        let x = 10 // Another comment
    "#;
    let ast = parser::parse_from_source(code);
    assert!(ast.is_ok());
    let statements = ast.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn parse_comment_hash_error() {
    let code = r#"
        # Hash comment should be rejected
        let x = 10
    "#;
    let ast = parser::parse_from_source(code);
    assert!(ast.is_err());
}
