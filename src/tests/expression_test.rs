use crate::expression::Value::{Bool, Int, Str};
use crate::expression::{BinaryOperator, Expression, Element, Value};
use crate::token::Operator;
use crate::Context;

#[test]
fn test_sub_int_int() {
    let mut ctx = Context::default();
    let opt = box BinaryOperator {
        operator: Operator::Subtract,
        left: box Element::Value(Int(1)),
        right: box Element::Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(0));
}

#[should_panic]
#[test]
fn test_sub_bool_int() {
    let mut ctx = Context::default();
    let opt: Box<dyn Expression> = box BinaryOperator {
        operator: Operator::ADD,
        left: box Element::Value(Bool(false)),
        right: box Element::Value(Int(1)),
    };
    opt.evaluate(&mut ctx).unwrap();
}

#[test]
fn test_add_int_int() {
    let mut ctx = Context::default();
    let opt = BinaryOperator {
        operator: Operator::ADD,
        left: box Element::Value(Int(1)),
        right: box Element::Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(2));
}

#[test]
fn test_add_str_int() {
    let mut ctx = Context::default();
    let opt = BinaryOperator {
        operator: Operator::ADD,
        left: box Element::Value(Str("hello".to_string())),
        right: box Element::Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Str("hello1".to_string()));
}

#[should_panic]
#[test]
fn test_add_bool_int() {
    let mut ctx = Context::default();
    let opt = BinaryOperator {
        operator: Operator::ADD,
        left: box Element::Value(Bool(false)),
        right: box Element::Value(Int(1)),
    };
    opt.evaluate(&mut ctx).unwrap();
}