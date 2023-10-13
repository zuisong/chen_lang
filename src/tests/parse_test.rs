use crate::parse::parse_block;
use crate::{parser, token};
use crate::token::Keyword::*;
use crate::token::Operator::*;
use crate::token::Token::*;

#[test]
fn parse(){

    let code: std::string::String = r#"
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


    let res =   parser(
        token::tokenlizer(code).unwrap(),


    );

    dbg!(&res);
}
