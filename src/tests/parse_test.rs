use std::boxed;

use crate::expression::{Assign, FunctionCall, If};
use crate::token::Operator;
use crate::{
    expression::{BinaryOperation, Expression, Literal, Local, Loop, Statement, Value},
    parser, token,
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

    let res = parser(token::tokenlizer(code).unwrap());

    dbg!(&res);

    assert_eq!(
        res.unwrap(),
        vec![
            Statement::Local(Local {
                name: "i".to_string(),
                expression: Expression::Literal(Literal::Value(Value::Int(0,),),),
            },),
            Statement::Loop(Loop {
                test: Expression::BinaryOperation(BinaryOperation {
                    operator: Operator::LT,
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
                                    operator: Operator::ADD,
                                    left: Expression::Literal(
                                        Literal::Identifier("i".to_string(),),
                                    )
                                    .into(),
                                    right: Expression::Literal(Literal::Value(Value::Str(
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
                                    operator: Operator::ADD,
                                    left: Expression::Literal(
                                        Literal::Identifier("i".to_string(),),
                                    )
                                    .into(),
                                    right: Expression::Literal(Literal::Value(Value::Str(
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
                            operator: Operator::ADD,
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
