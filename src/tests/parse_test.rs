use crate::{
    expression::{
        Assign, BinaryOperation, Expression, FunctionCall, If, Literal, Local, Loop, Statement,
    },
    parser,
    token::Operator,
    value::Value,
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
                    left: Expression::Identifier("i".to_string()).into(),
                    right: Expression::Literal(Literal::Value(Value::Int(100,),),).into(),
                },),
                body: vec![
                    Statement::Expression(Expression::If(If {
                        test: Expression::BinaryOperation(BinaryOperation {
                            operator: Operator::Equals,
                            left: Expression::BinaryOperation(BinaryOperation {
                                operator: Operator::Mod,
                                left: Expression::Identifier("i".to_string()).into(),
                                right: Expression::Literal(Literal::Value(Value::Int(2,),),).into(),
                            },)
                            .into(),
                            right: Expression::Literal(Literal::Value(Value::Int(0,),),).into(),
                        },)
                        .into(),
                        body: vec![Statement::Expression(Expression::FunctionCall(
                            FunctionCall {
                                callee: Box::new(Expression::Identifier("println".to_string())),
                                arguments: vec![Expression::BinaryOperation(BinaryOperation {
                                    operator: Operator::Add,
                                    left: Expression::Identifier("i".to_string()).into(),
                                    right: Expression::Literal(Literal::Value(Value::string(
                                        " 是偶数".to_string(),
                                    ),),)
                                    .into(),
                                },),],
                            },
                        ),),],
                        else_body: vec![Statement::Expression(Expression::FunctionCall(
                            FunctionCall {
                                callee: Box::new(Expression::Identifier("println".to_string())),
                                arguments: vec![Expression::BinaryOperation(BinaryOperation {
                                    operator: Operator::Add,
                                    left: Expression::Identifier("i".to_string()).into(),
                                    right: Expression::Literal(Literal::Value(Value::string(
                                        " 是奇数".to_string(),
                                    ),),)
                                    .into(),
                                },),],
                            },
                        ),),],
                    },)),
                    Statement::Assign(Assign {
                        name: "i".to_string(),
                        expr: Expression::BinaryOperation(BinaryOperation {
                            operator: Operator::Add,
                            left: Expression::Identifier("i".to_string()).into(),
                            right: Expression::Literal(Literal::Value(Value::Int(1,),),).into(),
                        },)
                        .into(),
                    },),
                ],
            },),
        ],
    )
}
