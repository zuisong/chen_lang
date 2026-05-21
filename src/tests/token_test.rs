use pretty_assertions::assert_eq;
use pretty_assertions::assert_matches;

use crate::tokenizer;
use crate::tokenizer::Keyword::{ELSE, FOR, IF, LET};
use crate::tokenizer::Operator::{Add, Assign, Equals, Lt, Mod};
use crate::tokenizer::Operator::{NotEquals, Or, Subtract};
use crate::tokenizer::Token::{Identifier, Int, Keyword, LBig, LParen, NewLine, Operator, RBig, RParen, String};

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
    let code: std::string::String = r#"
let i = 0
while (i < 100) {
    if (i % 2 == 0) {
        console.log(i + " 是偶数")
    } else {
        console.log(i + " 是奇数")
    }
    i = i + 1
}
"#
    .to_string();
    #[rustfmt::skip]
    assert_eq!(
        tokenizer::tokenizer(code).unwrap().into_iter().map(|(t, _)| t).collect::<Vec<_>>(),
        vec![
            NewLine,
            Keyword(LET), Identifier("i".to_string()), Operator(Assign), Int(0), NewLine,
            Keyword(crate::tokenizer::Keyword::WHILE), LParen, Identifier("i".to_string()), Operator(Lt), Int(100), RParen, LBig, NewLine,
            Keyword(IF), LParen, Identifier("i".to_string()), Operator(Mod), Int(2), Operator(Equals), Int(0), RParen, LBig, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, Identifier("i".to_string()), Operator(Add), String(" 是偶数".to_string()), RParen, NewLine,
            RBig, Keyword(ELSE), LBig, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, Identifier("i".to_string()), Operator(Add), String(" 是奇数".to_string()), RParen, NewLine,
            RBig, NewLine,
            Identifier("i".to_string()), Operator(Assign), Identifier("i".to_string()), Operator(Add), Int(1),
            NewLine,
            RBig, NewLine,
        ]
    );
}

#[test]
fn parse_code2() {
    let code = r#"
// 这里是注释,
// 注释以// 开始, 直到行末
function aaa(n) {
    let i = 100
    let sum = 0
    while (i != 0) {
        i = i - 1
        // 这里有相对复杂的逻辑运算
        if ((i % 2 != 0) || (i % 3 == 0)) {
            console.log(i)
            // 打印出来的 i 都是奇数 或者是能被三整除的偶数
            sum = sum + i
        }
    }
    // sum 为 100以为 of 奇数之和
    console.log("100以内的 奇数或者是能被三整除的偶数 之和是")
    console.log(sum)
    return sum
}
let sum = 0
sum = aaa(100)
console.log(sum)
"#
    .to_string();

    #[rustfmt::skip]
    assert_eq!(
        tokenizer::tokenizer(code).unwrap().into_iter().map(|(t, _)| t).collect::<Vec<_>>(),
        vec![
            NewLine,NewLine,NewLine,
            Keyword(crate::tokenizer::Keyword::FUNCTION), Identifier("aaa".to_string()), LParen, Identifier("n".to_string()), RParen, LBig, NewLine,
            Keyword(LET), Identifier("i".to_string()), Operator(Assign), Int(100), NewLine,
            Keyword(LET), Identifier("sum".to_string()), Operator(Assign), Int(0), NewLine,
            Keyword(crate::tokenizer::Keyword::WHILE), LParen, Identifier("i".to_string()), Operator(NotEquals), Int(0), RParen, LBig, NewLine,
            Identifier("i".to_string()), Operator(Assign), Identifier("i".to_string()), Operator(Subtract), Int(1), NewLine,
            NewLine,
            Keyword(IF), LParen, LParen, Identifier("i".to_string()), Operator(Mod), Int(2), Operator(NotEquals), Int(0), RParen, Operator(Or), LParen, Identifier("i".to_string()), Operator(Mod), Int(3), Operator(Equals), Int(0), RParen, RParen, LBig, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, Identifier("i".to_string()), RParen, NewLine,
            NewLine,
            Identifier("sum".to_string()), Operator(Assign), Identifier("sum".to_string()), Operator(Add), Identifier("i".to_string()), NewLine,
            RBig, NewLine,
            RBig, NewLine, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, String("100以内的 奇数或者是能被三整除的偶数 之和是".to_string()), RParen, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, Identifier("sum".to_string()), RParen, NewLine, Keyword(crate::tokenizer::Keyword::RETURN), Identifier("sum".to_string()), NewLine, RBig, NewLine,
            Keyword(LET), Identifier("sum".to_string()), Operator(Assign), Int(0), NewLine,
            Identifier("sum".to_string()), Operator(Assign), Identifier("aaa".to_string()), LParen, Int(100), RParen, NewLine,
            Identifier("console".to_string()), crate::tokenizer::Token::Dot, Identifier("log".to_string()), LParen, Identifier("sum".to_string()), RParen, NewLine,
        ],
    );
}
