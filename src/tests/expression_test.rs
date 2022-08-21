use crate::expression::Element::Value;
use crate::expression::Value::{Bool, Int, Str};
use crate::expression::{BinaryStatement};
use crate::expression::{Expression, NotStatement};
use crate::token::Operator;
use crate::Context;

quickcheck! {
    #[should_panic]
    fn test_not_int2(i:i32) -> bool {
        test_not_int(i);
        println!("{}",i);
        false
    }
}

fn test_not_int(i: i32) {
    let expr = NotStatement {
        expr: Box::new(Value(Int(i))),
    };

    let res = expr.evaluate(&mut Context::default());
    assert_eq!(res.unwrap(), Bool(true));
}

#[test]
fn test_not_false() {
    let expr = NotStatement {
        expr: Box::new(Value(Bool(false))),
    };

    let res = expr.evaluate(&mut Context::default());
    assert_eq!(res.unwrap(), Bool(true));
}

#[test]
fn test_sub_int_int() {
    let mut ctx = Context::default();
    let opt = box BinaryStatement {
        operator: Operator::Subtract,
        left: box Value(Int(1)),
        right: box Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(0));
}

#[should_panic]
#[test]
fn test_sub_bool_int() {
    let mut ctx = Context::default();
    let opt: Box<dyn Expression> = box BinaryStatement {
        operator: Operator::ADD,
        left: box Value(Bool(false)),
        right: box Value(Int(1)),
    };
    opt.evaluate(&mut ctx).unwrap();
}

#[test]
fn test_add_int_int() {
    let mut ctx = Context::default();
    let opt = BinaryStatement {
        operator: Operator::ADD,
        left: box Value(Int(1)),
        right: box Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Int(2));
}

#[test]
fn test_add_str_int() {
    let mut ctx = Context::default();
    let opt = BinaryStatement {
        operator: Operator::ADD,
        left: box Value(Str("hello".to_string())),
        right: box Value(Int(1)),
    };
    assert_eq!(opt.evaluate(&mut ctx).unwrap(), Str("hello1".to_string()));
}

#[should_panic]
#[test]
fn test_add_bool_int() {
    let mut ctx = Context::default();
    let opt = BinaryStatement {
        operator: Operator::ADD,
        left: box Value(Bool(false)),
        right: box Value(Int(1)),
    };
    opt.evaluate(&mut ctx).unwrap();
}
