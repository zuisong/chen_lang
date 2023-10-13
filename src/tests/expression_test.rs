// use pretty_assertions::assert_eq;
//
// use crate::expression::BinaryStatement;
// use crate::expression::Element::Value;
// use crate::expression::Value::{Bool, Int, Str};
// use crate::expression::{Expression, NotStatement};
// use crate::token::Operator;
// use crate::Context;
// #[test]
// #[should_panic]
// fn test_not_int2() {
//     test_not_int(10);
// }
//
// fn test_not_int(i: i32) {
//     let expr = NotStatement {
//         expr: Box::new(Value(Int(i))),
//     };
//
//     let res = expr.evaluate(&mut Context::default());
//     assert_eq!(res.unwrap(), Bool(true));
// }
//
// #[test]
// fn test_not_false() {
//     let expr = NotStatement {
//         expr: Box::new(Value(Bool(false))),
//     };
//
//     let res = expr.evaluate(&mut Context::default());
//     assert_eq!(res.unwrap(), Bool(true));
// }
//
// #[test]
// fn test_sub_int_int() {
//     let mut ctx = Context::default();
//     let opt = Box::new(BinaryStatement {
//         operator: Operator::Subtract,
//         left: Box::new(Value(Int(1))),
//         right: Box::new(Value(Int(1))),
//     });
//     assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(0));
// }
//
// #[should_panic]
// #[test]
// fn test_sub_bool_int() {
//     let mut ctx = Context::default();
//     let opt: Box<dyn Expression> = Box::new(BinaryStatement {
//         operator: Operator::ADD,
//         left: Box::new(Value(Bool(false))),
//         right: Box::new(Value(Int(1))),
//     });
//     opt.evaluate(&mut ctx).unwrap();
// }
//
// #[test]
// fn test_add_int_int() {
//     let mut ctx = Context::default();
//     let opt = BinaryStatement {
//         operator: Operator::ADD,
//         left: Box::new(Value(Int(1))),
//         right: Box::new(Value(Int(1))),
//     };
//     assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(2));
// }
//
// #[test]
// fn test_add_str_int() {
//     let mut ctx = Context::default();
//     let opt = BinaryStatement {
//         operator: Operator::ADD,
//         left: Box::new(Value(Str("hello".to_string()))),
//         right: Box::new(Value(Int(1))),
//     };
//     assert_eq!(opt.evaluate(&mut ctx).unwrap(), Str("hello1".to_string()));
// }
//
// #[should_panic]
// #[test]
// fn test_add_bool_int() {
//     let mut ctx = Context::default();
//     let opt = BinaryStatement {
//         operator: Operator::ADD,
//         left: Box::new(Value(Bool(false))),
//         right: Box::new(Value(Int(1))),
//     };
//     opt.evaluate(&mut ctx).unwrap();
// }
