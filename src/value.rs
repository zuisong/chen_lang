use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use indexmap::IndexMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use thiserror::Error;

use crate::vm::VMRuntimeError;

/// Table 结构，用于实现对象和 Map
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub data: IndexMap<String, Value>,
    pub metatable: Option<Rc<RefCell<Table>>>,
}

/// 原生函数类型
pub type NativeFnType = dyn Fn(Vec<Value>) -> Result<Value, VMRuntimeError> + 'static;

/// 运行时值类型 - 统一表示所有数据类型
#[derive(Clone)]
pub enum Value {
    Int(i32),
    Float(Decimal),
    Bool(bool),
    String(Rc<String>),
    /// 对象类型 (Table)
    Object(Rc<RefCell<Table>>),
    /// 函数引用 (函数名 - Chen 语言定义的函数)
    Function(String),
    /// 原生函数 (Rust 定义的函数)
    NativeFunction(Rc<Box<NativeFnType>>),
    Null,
}

impl Value {
    /// 创建整数值
    pub fn int(n: i32) -> Self {
        Value::Int(n)
    }

    /// 创建浮点数值
    pub fn float(f: Decimal) -> Self {
        Value::Float(f)
    }

    /// 创建布尔值
    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// 创建字符串值
    pub fn string(s: String) -> Self {
        Value::String(Rc::new(s))
    }

    /// 创建空对象
    pub fn object() -> Self {
        Value::Object(Rc::new(RefCell::new(Table {
            data: IndexMap::new(),
            metatable: None,
        })))
    }

    /// 创建空值
    pub fn null() -> Self {
        Value::Null
    }

    /// 获取整数值
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// 获取浮点数值
    pub fn as_float(&self) -> Option<Decimal> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 获取字符串值
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 检查是否为真值（用于条件判断）
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Null => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == Decimal::ZERO => false,
            Value::String(s) if s.is_empty() => false,
            _ => true,
        }
    }

    /// 类型检查
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
            Value::String(_) => ValueType::String,
            Value::Object(_) => ValueType::Object,
            Value::Function(_) => ValueType::Function,
            Value::NativeFunction(_) => ValueType::Function,
            Value::Null => ValueType::Null,
        }
    }

    /// 数值类型转换
    pub fn to_float(&self) -> Option<Decimal> {
        match self {
            Value::Int(n) => Some(Decimal::from(*n)),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 转换为整数（截断）
    pub fn to_int(&self) -> Option<i32> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => f.to_i32(),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // 对象比较：引用比较
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
            // 函数比较：名称比较
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Rc::ptr_eq(a, b),
            // 混合类型比较：int和float可以比较
            (Value::Int(a), Value::Float(b)) => Decimal::from(*a) == *b,
            (Value::Float(a), Value::Int(b)) => *a == Decimal::from(*b),
            _ => false,
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{{")?;
        let mut first = true;
        for (k, v) in &self.data {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", k, v)?;
            first = false;
        }
        write!(f, "}}}}")
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl.normalize()),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Object(obj) => write!(f, "{}", obj.borrow()),
            Value::Function(name) => write!(f, "<function {}", name),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::Null => write!(f, "null"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "Int({})", n),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::String(s) => write!(f, "String(\"{}\")", s),
            Value::Object(obj) => write!(f, "Object({:.?})", obj.borrow()),
            Value::Function(name) => write!(f, "Function({})", name),
            Value::NativeFunction(_) => write!(f, "NativeFunction(<native fn>)"),
            Value::Null => write!(f, "Null"),
        }
    }
}

/// 值类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    String,
    Object,
    Function,
    Null,
}

impl ValueType {
    /// 检查是否为数值类型
    pub fn is_numeric(&self) -> bool {
        matches!(self, ValueType::Int | ValueType::Float)
    }

    /// 获取两个类型的公共类型（用于类型提升）
    pub fn get_common_type(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (ValueType::Int, ValueType::Float) | (ValueType::Float, ValueType::Int) => Some(ValueType::Float),
            (a, b) if a == b => Some(*a),
            _ => None,
        }
    }
}

/// 运算错误类型
#[derive(Error, Debug, Clone)]
pub enum ValueError {
    #[error("Type mismatch in {{operation}}: expected {{expected:?}}, found {{found:?}}")]
    TypeMismatch {
        expected: ValueType,
        found: ValueType,
        operation: String,
    },
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid operation: {{left_type:?}} {{operator}} {{right_type:?}}")]
    InvalidOperation {
        operator: String,
        left_type: ValueType,
        right_type: ValueType,
    },
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Undefined field: {{0}}")]
    UndefinedField(String),
    #[error("Attempt to call non-function value: {{0:?}}")]
    CallNonFunction(ValueType),
}

/// 表示算术操作的结果，可以是直接的 Value，也可以是需要调用元方法的指示
#[derive(Debug, Clone)]
pub enum OpResult {
    Value(Value),
    MetamethodCall(MetamethodCallInfo),
}

/// 包含元方法调用所需的信息
#[derive(Debug, Clone)]
pub struct MetamethodCallInfo {
    pub metamethod: Value,
    pub left: Value,
    pub right: Value,
}

impl OpResult {
    /// Helper for tests that expect a direct Value result. Panics if it's a MetamethodCall.
    pub fn unwrap_value(self) -> Value {
        match self {
            OpResult::Value(val) => val,
            OpResult::MetamethodCall(_) => {
                panic!("OpResult::unwrap_value() called on a MetamethodCall variant")
            }
        }
    }
}

/// 算术运算实现
impl Value {
    pub fn add(&self, other: &Value) -> Result<OpResult, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(OpResult::Value(Value::Int(a + b))),
            (Value::Float(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(a + b))),
            (Value::Int(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(Decimal::from(*a) + b))),
            (Value::Float(a), Value::Int(b)) => Ok(OpResult::Value(Value::Float(a + Decimal::from(*b)))),
            (Value::String(a), Value::String(b)) => {
                let mut result = String::as_str(a).to_string();
                result.push_str(String::as_str(b));
                Ok(OpResult::Value(Value::string(result)))
            }
            // 字符串连接：任何类型都可以与字符串相加
            (Value::String(a), _) => {
                let mut result = String::as_str(a).to_string();
                result.push_str(&other.to_string());
                Ok(OpResult::Value(Value::string(result)))
            }
            (_, Value::String(b)) => {
                let mut result = self.to_string();
                result.push_str(String::as_str(b));
                Ok(OpResult::Value(Value::string(result)))
            }
            // Metamethod lookup for __add
            (left_val, right_val) => {
                let metamethod = left_val
                    .get_metamethod_from_object("__add")
                    .or_else(|| right_val.get_metamethod_from_object("__add"));

                if let Some(metamethod_func) = metamethod {
                    Ok(OpResult::MetamethodCall(MetamethodCallInfo {
                        metamethod: metamethod_func,
                        left: left_val.clone(),
                        right: right_val.clone(),
                    }))
                } else {
                    Err(ValueError::InvalidOperation {
                        operator: "+".to_string(),
                        left_type: self.get_type(),
                        right_type: other.get_type(),
                    })
                }
            }
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<OpResult, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(OpResult::Value(Value::Int(a - b))),
            (Value::Float(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(a - b))),
            (Value::Int(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(Decimal::from(*a) - b))),
            (Value::Float(a), Value::Int(b)) => Ok(OpResult::Value(Value::Float(a - Decimal::from(*b)))),
            // Metamethod lookup for __sub
            (left_val, right_val) => {
                let metamethod = left_val
                    .get_metamethod_from_object("__sub")
                    .or_else(|| right_val.get_metamethod_from_object("__sub"));

                if let Some(metamethod_func) = metamethod {
                    Ok(OpResult::MetamethodCall(MetamethodCallInfo {
                        metamethod: metamethod_func,
                        left: left_val.clone(),
                        right: right_val.clone(),
                    }))
                } else {
                    Err(ValueError::InvalidOperation {
                        operator: "-".to_string(),
                        left_type: self.get_type(),
                        right_type: other.get_type(),
                    })
                }
            }
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<OpResult, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(OpResult::Value(Value::Int(a * b))),
            (Value::Float(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(a * b))),
            (Value::Int(a), Value::Float(b)) => Ok(OpResult::Value(Value::Float(Decimal::from(*a) * b))),
            (Value::Float(a), Value::Int(b)) => Ok(OpResult::Value(Value::Float(a * Decimal::from(*b)))),
            // Metamethod lookup for __mul
            (left_val, right_val) => {
                let metamethod = left_val
                    .get_metamethod_from_object("__mul")
                    .or_else(|| right_val.get_metamethod_from_object("__mul"));

                if let Some(metamethod_func) = metamethod {
                    Ok(OpResult::MetamethodCall(MetamethodCallInfo {
                        metamethod: metamethod_func,
                        left: left_val.clone(),
                        right: right_val.clone(),
                    }))
                } else {
                    Err(ValueError::InvalidOperation {
                        operator: "*".to_string(),
                        left_type: self.get_type(),
                        right_type: other.get_type(),
                    })
                }
            }
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == Decimal::ZERO {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == Decimal::ZERO {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(Decimal::from(*a) / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / Decimal::from(*b)))
                }
            }
            _ => Err(ValueError::InvalidOperation {
                operator: "/".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == Decimal::ZERO {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == Decimal::ZERO {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(Decimal::from(*a) % b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % Decimal::from(*b)))
                }
            }
            _ => Err(ValueError::InvalidOperation {
                operator: "%".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }
}

/// 比较运算实现
impl Value {
    pub fn equal(&self, other: &Value) -> Value {
        Value::bool(self == other)
    }

    pub fn not_equal(&self, other: &Value) -> Value {
        Value::bool(self != other)
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool(Decimal::from(*a) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a < Decimal::from(*b))),
            (Value::String(a), Value::String(b)) => Ok(Value::bool(String::as_str(a) < String::as_str(b))),
            _ => Err(ValueError::InvalidOperation {
                operator: "<".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool(Decimal::from(*a) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a <= Decimal::from(*b))),
            (Value::String(a), Value::String(b)) => Ok(Value::bool(String::as_str(a) <= String::as_str(b))),
            _ => Err(ValueError::InvalidOperation {
                operator: "<=".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool(Decimal::from(*a) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a > Decimal::from(*b))),
            (Value::String(a), Value::String(b)) => Ok(Value::bool(String::as_str(a) > String::as_str(b))),
            _ => Err(ValueError::InvalidOperation {
                operator: ">".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn greater_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool(Decimal::from(*a) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a >= Decimal::from(*b))),
            (Value::String(a), Value::String(b)) => Ok(Value::bool(String::as_str(a) >= String::as_str(b))),
            _ => Err(ValueError::InvalidOperation {
                operator: ">=".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }
}

/// 逻辑运算实现
impl Value {
    pub fn and(&self, other: &Value) -> Value {
        Value::bool(self.is_truthy() && other.is_truthy())
    }

    pub fn or(&self, other: &Value) -> Value {
        Value::bool(self.is_truthy() || other.is_truthy())
    }

    pub fn not(&self) -> Value {
        Value::bool(!self.is_truthy())
    }
}

/// Metatable support methods
impl Value {
    /// Recursively search for a metamethod in the object's metatable chain.
    /// Returns the metamethod Value if found, otherwise None.
    pub fn get_metamethod_from_object(&self, metamethod_name: &str) -> Option<Value> {
        let mut current_obj_owned = self.clone(); // Own the Value directly
        loop {
            match current_obj_owned {
                Value::Object(table_ref) => {
                    let table = table_ref.borrow();
                    if let Some(metatable_ref) = &table.metatable {
                        let metatable = metatable_ref.borrow();
                        if let Some(metamethod_val) = metatable.data.get(metamethod_name) {
                            return match metamethod_val {
                                Value::Function(_) | Value::NativeFunction(_) => Some(metamethod_val.clone()),
                                _ => {
                                    // Found a value, but it's not a function, treat as not found
                                    None
                                }
                            };
                        }
                        // Continue search in the metatable's metatable (Lua's behavior for __index chain)
                        current_obj_owned = Value::Object(metatable_ref.clone());
                    } else {
                        return None; // No metatable, end of chain
                    }
                }
                _ => return None, // Not an object, cannot have a metatable
            }
        }
    }
    /// Get field with metatable support (__index metamethod)
    pub fn get_field_with_meta(&self, key: &str) -> Value {
        match self {
            Value::Object(table_ref) => {
                let table = table_ref.borrow();

                // 1. Try direct lookup first
                if let Some(value) = table.data.get(key) {
                    return value.clone();
                }

                // 2. Check for metatable
                if let Some(metatable_ref) = &table.metatable {
                    let metatable = metatable_ref.borrow();

                    // 3. Look for __index metamethod
                    if let Some(index_method) = metatable.data.get("__index") {
                        match index_method {
                            // If __index is a table, recursively look up the key
                            Value::Object(index_table_ref) => {
                                return Value::Object(index_table_ref.clone()).get_field_with_meta(key);
                            }
                            Value::Function(_) => {
                                // TODO: If __index is a function, call it (future feature)
                            }
                            _ => {}
                        }
                    }
                }

                // Not found anywhere
                Value::null()
            }
            _ => Value::null(),
        }
    }

    /// Set field with metatable support (__newindex metamethod placeholder)
    pub fn set_field_with_meta(&self, key: String, value: Value) -> Result<(), ValueError> {
        match self {
            Value::Object(table_ref) => {
                let mut table = table_ref.borrow_mut();

                // If key already exists, always update directly
                if table.data.contains_key(&key) {
                    table.data.insert(key, value);
                    return Ok(());
                }

                // Check for __newindex metamethod (placeholder for future)
                if let Some(metatable_ref) = &table.metatable {
                    let metatable = metatable_ref.borrow();
                    if metatable.data.contains_key("__newindex") {
                        // TODO: Call __newindex if it's a function (future feature)
                        // For now, just insert directly
                    }
                }

                // No __newindex or it's not callable, insert directly
                table.data.insert(key, value);
                Ok(())
            }
            _ => Err(ValueError::InvalidOperation {
                operator: "set_field".to_string(),
                left_type: self.get_type(),
                right_type: ValueType::Null,
            }),
        }
    }

    /// Set metatable for an object
    pub fn set_metatable(&self, metatable: Value) -> Result<(), ValueError> {
        let meta_type = metatable.get_type(); // Get type before consuming
        match (self, metatable) {
            (Value::Object(obj_ref), Value::Object(meta_ref)) => {
                obj_ref.borrow_mut().metatable = Some(meta_ref);
                Ok(())
            }
            (Value::Object(obj_ref), Value::Null) => {
                // Allow setting metatable to null (removes metatable)
                obj_ref.borrow_mut().metatable = None;
                Ok(())
            }
            _ => Err(ValueError::InvalidOperation {
                operator: "set_metatable".to_string(),
                left_type: self.get_type(),
                right_type: meta_type,
            }),
        }
    }

    /// Get metatable for an object
    pub fn get_metatable(&self) -> Value {
        match self {
            Value::Object(obj_ref) => {
                if let Some(meta_ref) = &obj_ref.borrow().metatable {
                    Value::Object(meta_ref.clone())
                } else {
                    Value::null()
                }
            }
            _ => Value::null(),
        }
    }
}
