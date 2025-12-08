use chen_lang::expression::*;
use chen_lang::parse_pest;

#[test]
fn test_pest_basic() {
    let code = r#"
        let x = 1 + 2 * 3
        print(x)
    "#;

    let ast = parse_pest::parse(code).expect("Pest parsing failed");

    // Check structure
    assert!(!ast.is_empty());

    // Check first statement: let x = 1 + 2 * 3
    if let Statement::Local(local) = &ast[0] {
        assert_eq!(local.name, "x");
        // We could deeply check the expression tree, but let's trust it for now
    } else {
        panic!("First statement should be a declaration");
    }
}

#[test]
fn test_pest_function() {
    let code = r#"
        def add(a, b) {
            return a + b
        }
        print(add(1, 2))
    "#;

    let ast = parse_pest::parse(code).expect("Pest parsing failed");
    assert_eq!(ast.len(), 2);

    if let Statement::FunctionDeclaration(fd) = &ast[0] {
        assert_eq!(fd.name, "add");
        assert_eq!(fd.parameters, vec!["a", "b"]);
    } else {
        panic!("First statement should be function declaration");
    }
}
