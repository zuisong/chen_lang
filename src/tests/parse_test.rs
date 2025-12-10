use crate::{
    parser,
};

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