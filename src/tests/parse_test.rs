use crate::parser;

#[test]
fn parse() {
    let code: String = r#"
 local i = 0
 while i < 100 do

     if i % 2 == 0 then
         println(i .. " is even")
     else
         println(i .. " is odd")
     end
     i = i + 1
 end
 "#
    .to_string();

    let res = parser::parse_from_source(&code);

    dbg!(&res);

    assert!(res.is_ok());
    let statements = res.unwrap();
    assert!(!statements.is_empty());
}
