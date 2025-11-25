use crate::{
    Token, expression::{Assign, BinaryOperation, Expression, FunctionCall, If, Literal, Local, Loop, Statement}, parse, token::{self, Operator}, value::Value
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

    let tokens = token::tokenlizer(code).unwrap();
    let mut lines: Vec<Box<[Token]>> = vec![];
    let mut temp = vec![];
    for x in tokens {
        if let Token::NewLine = x {
            if !temp.is_empty() {
                lines.push(temp.into_boxed_slice());
                temp = vec![];
            }
        } else {
            temp.push(x)
        }
    }
    let res = parse::parse_block(lines.as_slice(), 0);

    dbg!(&lines);
    dbg!(&res);

    assert!(res.is_ok());
    let (_end_line, statements) = res.unwrap();
    // Just check that parsing works without specific line count
    assert!(!statements.is_empty());


    assert_eq!(
        statements,
        vec![
            Statement::Local(Local {
                name: "i".to_string(),
                expression: Expression::Literal(Literal::Value(Value::Int(0,),),),
            },),
            Statement::Loop(Loop {
                test: Expression::BinaryOperation(BinaryOperation {
                    operator: Operator::Lt,
                    left: Expression::Literal(Literal::Identifier("i".to_string(),),).into(),
                    right: Expression::Literal(Literal::Value(Value::Int(100,),),).into(),
                },),
                body: vec![
                    Statement::If(If {
                        test: Expression::BinaryOperation(BinaryOperation {
                            operator: Operator::Equals,
                            left: Expression::BinaryOperation(BinaryOperation {
                                operator: Operator::Mod,
                                left: Expression::Literal(Literal::Identifier("i".to_string(),),)
                                    .into(),
                                right: Expression::Literal(Literal::Value(Value::Int(2,),),).into(),
                            },)
                            .into(),
                            right: Expression::Literal(Literal::Value(Value::Int(0,),),).into(),
                        },),
                        body: vec![Statement::Expression(Expression::FunctionCall(
                            FunctionCall {
                                name: "println".to_string(),
                                arguments: vec![Expression::BinaryOperation(BinaryOperation {
                                    operator: Operator::Add,
                                    left: Expression::Literal(
                                        Literal::Identifier("i".to_string(),),
                                    )
                                    .into(),
                                    right: Expression::Literal(Literal::Value(Value::string(
                                        " 是偶数".to_string(),
                                    ),),)
                                    .into(),
                                },),],
                            },
                        ),),],
                        else_body: vec![Statement::Expression(Expression::FunctionCall(
                            FunctionCall {
                                name: "println".to_string(),
                                arguments: vec![Expression::BinaryOperation(BinaryOperation {
                                    operator: Operator::Add,
                                    left: Expression::Literal(
                                        Literal::Identifier("i".to_string(),),
                                    )
                                    .into(),
                                    right: Expression::Literal(Literal::Value(Value::string(
                                        " 是奇数".to_string(),
                                    ),),)
                                    .into(),
                                },),],
                            },
                        ),),],
                    },),
                    Statement::Assign(Assign {
                        name: "i".to_string(),
                        expr: Expression::BinaryOperation(BinaryOperation {
                            operator: Operator::Add,
                            left: Expression::Literal(Literal::Identifier("i".to_string(),),)
                                .into(),
                            right: Expression::Literal(Literal::Value(Value::Int(1,),),).into(),
                        },)
                        .into(),
                    },),
                ],
            },),
        ],
    )
}