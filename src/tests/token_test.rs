#![rustfmt::skip]
use crate::token;
use crate::token::Keyword::{ELSE, FOR, IF, LET};
use crate::token::Operator::{Assign, Equals, Mod, ADD, LT};
use crate::token::StdFunction::Print;
use crate::token::Token::{
    Identifier, Int, Keyword, LBig, LParen, NewLine, Operator, RBig, RParen, StdFunction, String,
};
use crate::Keyword::DEF;
use crate::Operator::{NotEquals, Or, Subtract};

#[test]
fn test_parse_keyword() {
    assert_eq!(
        token::tokenlizer("println".to_string()).unwrap(),
        vec![StdFunction(Print(true))]
    )
}

#[test]
fn test_parse_for() {
    assert_eq!(
        token::tokenlizer("for".to_string()).unwrap(),
        vec![Keyword(FOR)]
    )
}

#[test]
fn parse_code() {
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

    assert_eq!(
        token::tokenlizer(code).unwrap(),
        vec![
            NewLine,
            Keyword(LET), Identifier("i".to_string()), Operator(Assign), Int(0), NewLine,
            Keyword(FOR), Identifier("i".to_string()), Operator(LT), Int(100), LBig, NewLine,
            NewLine,
            Keyword(IF), Identifier("i".to_string()), Operator(Mod), Int(2), Operator(Equals), Int(0), LBig, NewLine,
            StdFunction(Print(true)), LParen, Identifier("i".to_string()), Operator(ADD), String(" 是偶数".to_string()), RParen, NewLine,
            RBig, Keyword(ELSE), LBig, NewLine,
            StdFunction(Print(true)), LParen, Identifier("i".to_string()), Operator(ADD), String(" 是奇数".to_string()), RParen, NewLine,
            RBig, NewLine,
            Identifier("i".to_string()), Operator(Assign), Identifier("i".to_string()), Operator(ADD), Int(1),
            NewLine,
            RBig, NewLine,
        ]
    );
}

#[test]
fn parse_code2() {
    let code = r#"
# 这里是注释,
# 注释以# 开始, 直到行末
def aaa(n){
    let i = 100
    let sum = 0
    for i!=0 {
        i = i - 1
        # 这里有相对复杂的逻辑运算
        if (i%2!=0) || (i%3==0)  {
            println(i)
            # 打印出来的 i 都是奇数 或者是能被三整除的偶数
            sum = sum + i
        }
    }
    # sum 为 100以为的奇数之和
    println("100以内的 奇数或者是能被三整除的偶数 之和是")
    println(sum)
    sum
}
let sum = 0
sum = aaa(100)
println(sum)
"#
    .to_string();

    assert_eq!(
        token::tokenlizer(code).unwrap(),
        vec![
            NewLine,
            Keyword(DEF), Identifier("aaa".to_string()), LParen, Identifier("n".to_string()), RParen, LBig, NewLine,
            Keyword(LET), Identifier("i".to_string()), Operator(Assign), Int(100), NewLine,
            Keyword(LET), Identifier("sum".to_string()), Operator(Assign), Int(0), NewLine,
            Keyword(FOR), Identifier("i".to_string()), Operator(NotEquals), Int(0), LBig, NewLine,
            Identifier("i".to_string()), Operator(Assign), Identifier("i".to_string()), Operator(Subtract), Int(1), NewLine,
            Keyword(IF), LParen, Identifier("i".to_string()), Operator(Mod), Int(2), Operator(NotEquals), Int(0), RParen, Operator(Or), LParen, Identifier("i".to_string()), Operator(Mod), Int(3), Operator(Equals), Int(0), RParen, LBig, NewLine,
            StdFunction(Print(true)), LParen, Identifier("i".to_string()), RParen, NewLine,
            Identifier("sum".to_string()), Operator(Assign), Identifier("sum".to_string()), Operator(ADD), Identifier("i".to_string()), NewLine,
            RBig, NewLine,
            RBig, NewLine,
            StdFunction(Print(true)), LParen, String("100以内的 奇数或者是能被三整除的偶数 之和是".to_string()), RParen, NewLine,
            StdFunction(Print(true)), LParen, Identifier("sum".to_string()), RParen, NewLine, Identifier("sum".to_string()), NewLine, RBig, NewLine,
            Keyword(LET), Identifier("sum".to_string()), Operator(Assign), Int(0), NewLine,
            Identifier("sum".to_string()), Operator(Assign), Identifier("aaa".to_string()), LParen, Int(100), RParen, NewLine,
            StdFunction(Print(true)), LParen, Identifier("sum".to_string()), RParen, NewLine,
        ],
    );
}
