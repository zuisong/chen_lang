use rust_decimal::{Decimal, dec};

use crate::value::Value;

#[test]
fn test_arithmetic_operations() {
    let a = Value::int(5);
    let b = Value::int(3);

    assert_eq!(a.add(&b).unwrap().unwrap_value(), Value::int(8));
    assert_eq!(a.subtract(&b).unwrap().unwrap_value(), Value::int(2));
    assert_eq!(a.multiply(&b).unwrap().unwrap_value(), Value::int(15));
    assert_eq!(a.divide(&b).unwrap(), Value::int(1));
    assert_eq!(a.modulo(&b).unwrap(), Value::int(2));
}

#[test]
fn test_float_operations() {
    let a = Value::float(dec!(5.5));
    let b = Value::int(2);

    assert_eq!(a.add(&b).unwrap().unwrap_value(), Value::float(dec!(7.5)));
    assert_eq!(a.subtract(&b).unwrap().unwrap_value(), Value::float(dec!(3.5)));
    assert_eq!(a.multiply(&b).unwrap().unwrap_value(), Value::float(dec!(11.0)));
    assert_eq!(a.divide(&b).unwrap(), Value::float(dec!(2.75)));
}

#[test]
fn test_string_operations() {
    let a = Value::string("Hello".to_string());
    let b = Value::string(" World".to_string());

    assert_eq!(
        a.add(&b).unwrap().unwrap_value(),
        Value::string("Hello World".to_string())
    );
}

#[test]
fn test_comparison_operations() {
    let a = Value::int(5);
    let b = Value::int(3);

    assert_eq!(a.less_than(&b).unwrap(), Value::bool(false));
    assert_eq!(a.greater_than(&b).unwrap(), Value::bool(true));
    assert_eq!(a.equal(&b), Value::bool(false));
    assert_eq!(a.not_equal(&b), Value::bool(true));
}

#[test]
fn test_logical_operations() {
    let a = Value::bool(true);
    let b = Value::bool(false);

    assert_eq!(a.and(&b), Value::bool(false));
    assert_eq!(a.or(&b), Value::bool(true));
    assert_eq!(a.not(), Value::bool(false));
}

#[test]
fn test_type_conversions() {
    let int_val = Value::int(42);
    let float_val = Value::float(dec!(3.14));

    assert_eq!(int_val.to_float(), Some(dec!(42)));
    assert_eq!(float_val.to_int(), Some(3));
    assert_eq!(int_val.to_int(), Some(42));
    assert_eq!(float_val.to_float(), Some(dec!(3.14)));
}

#[test]
fn test_truthy_values() {
    assert!(Value::bool(true).is_truthy());
    assert!(!Value::bool(false).is_truthy());
    assert!(Value::int(1).is_truthy());
    assert!(!Value::int(0).is_truthy());
    assert!(Value::float(Decimal::from(1)).is_truthy());
    assert!(!Value::float(Decimal::from(0)).is_truthy());
    assert!(Value::string("test".to_string()).is_truthy());
    assert!(!Value::string("".to_string()).is_truthy());
    assert!(!Value::null().is_truthy());
}
