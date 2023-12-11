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
                expression: Literal(Value(Value::Int(0,),),),
            },),
            Loop(Loop {
                test: BinaryOperation(BinaryOperation {
                    operator: LT,
                    left: Literal(Identifier("i",),),
                    right: Literal(Value(Int(100,),),),
                },),
                body: [
                    If(If {
                        test: BinaryOperation(BinaryOperation {
                            operator: Equals,
                            left: BinaryOperation(BinaryOperation {
                                operator: Mod,
                                left: Literal(Identifier("i",),),
                                right: Literal(Value(Int(2,),),),
                            },),
                            right: Literal(Value(Int(0,),),),
                        },),
                        body: [Expression(FunctionCall(FunctionCall {
                            name: "println",
                            arguments: [BinaryOperation(BinaryOperation {
                                operator: ADD,
                                left: Literal(Identifier("i",),),
                                right: Literal(Value(Str(" 是偶数",),),),
                            },),],
                        },),),],
                        else_body: [Expression(FunctionCall(FunctionCall {
                            name: "println",
                            arguments: [BinaryOperation(BinaryOperation {
                                operator: ADD,
                                left: Literal(Identifier("i",),),
                                right: Literal(Value(Str(" 是奇数",),),),
                            },),],
                        },),),],
                    },),
                    Assign(Assign {
                        name: "i",
                        expr: BinaryOperation(BinaryOperation {
                            operator: ADD,
                            left: Literal(Identifier("i",),),
                            right: Literal(Value(Int(1,),),),
                        },),
                    },),
                ],
            },),
        ],
    )
}
