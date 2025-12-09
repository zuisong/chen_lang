use std::cell::RefCell;
use indexmap::IndexMap;
use std::fmt;
use std::rc::Rc;

/// Table 结构，用于实现对象和 Map
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub data: IndexMap<String, Value>,
    pub metatable: Option<Rc<RefCell<Table>>>,
}

/// 运行时值类型 - 统一表示所有数据类型
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(Rc<String>),
    /// 对象类型 (Table)
    Object(Rc<RefCell<Table>>),
    /// 函数引用 (函数名)
    Function(String),
    Null,
}

impl Value {
    /// 创建整数值
    pub fn int(n: i32) -> Self {
        Value::Int(n)
    }

    /// 创建浮点数值
    pub fn float(f: f32) -> Self {
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
    pub fn as_float(&self) -> Option<f32> {
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
            Value::Float(f) if *f == 0.0 => false,
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
            Value::Null => ValueType::Null,
        }
    }

    /// 数值类型转换
    pub fn to_float(&self) -> Option<f32> {
        match self {
            Value::Int(n) => Some(*n as f32),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 转换为整数（截断）
    pub fn to_int(&self) -> Option<i32> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => Some(*f as i32),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f32::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // 对象比较：引用比较
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
            // 函数比较：名称比较
            (Value::Function(a), Value::Function(b)) => a == b,
            // 混合类型比较：int和float可以比较
            (Value::Int(a), Value::Float(b)) => (*a as f32 - b).abs() < f32::EPSILON,
            (Value::Float(a), Value::Int(b)) => (a - *b as f32).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (k, v) in &self.data {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", k, v)?;
            first = false;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Object(obj) => write!(f, "{}", obj.borrow()),
            Value::Function(name) => write!(f, "<function {}>", name),
            Value::Null => write!(f, "null"),
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
            (ValueType::Int, ValueType::Float) | (ValueType::Float, ValueType::Int) => {
                Some(ValueType::Float)
            }
            (a, b) if a == b => Some(*a),
            _ => None,
        }
    }
}

/// 运算错误类型
#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeMismatch {
        expected: ValueType,
        found: ValueType,
        operation: String,
    },
    DivisionByZero,
    InvalidOperation {
        operator: String,
        left_type: ValueType,
        right_type: ValueType,
    },
    IndexOutOfBounds,
    UndefinedVariable(String),
    UndefinedField(String),
    CallNonFunction(ValueType),
    StackUnderflow(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::TypeMismatch {
                expected,
                found,
                operation,
            } => {
                write!(
                    f,
                    "Type mismatch in {}: expected {:?}, found {:?}",
                    operation, expected, found
                )
            }
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::InvalidOperation {
                operator,
                left_type,
                right_type,
            } => {
                write!(
                    f,
                    "Invalid operation: {:?} {} {:?}",
                    left_type, operator, right_type
                )
            }
            RuntimeError::StackUnderflow(msg) => write!(f, "Stack underflow: {}", msg),
            RuntimeError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::UndefinedField(name) => write!(f, "Undefined field: {}", name),
            RuntimeError::CallNonFunction(t) => {
                write!(f, "Attempt to call non-function value: {:?}", t)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// 算术运算实现
impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f32 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f32)),
            (Value::String(a), Value::String(b)) => {
                let mut result = String::as_str(a).to_string();
                result.push_str(String::as_str(b));
                Ok(Value::string(result))
            }
            // 字符串连接：任何类型都可以与字符串相加
            (Value::String(a), _) => {
                let mut result = String::as_str(a).to_string();
                result.push_str(&other.to_string());
                Ok(Value::string(result))
            }
            (_, Value::String(b)) => {
                let mut result = self.to_string();
                result.push_str(String::as_str(b));
                Ok(Value::string(result))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operator: "+".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f32 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f32)),
            _ => Err(RuntimeError::InvalidOperation {
                operator: "-".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f32 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f32)),
            _ => Err(RuntimeError::InvalidOperation {
                operator: "*".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f32 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / *b as f32))
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                operator: "/".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f32 % b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % *b as f32))
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
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

    pub fn less_than(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool((*a as f32) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a < (*b as f32))),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::bool(String::as_str(a) < String::as_str(b)))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operator: "<".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool((*a as f32) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a <= (*b as f32))),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::bool(String::as_str(a) <= String::as_str(b)))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operator: "<=".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool((*a as f32) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a > (*b as f32))),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::bool(String::as_str(a) > String::as_str(b)))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operator: ">".to_string(),
                left_type: self.get_type(),
                right_type: other.get_type(),
            }),
        }
    }

    pub fn greater_equal(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::bool((*a as f32) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::bool(*a >= (*b as f32))),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::bool(String::as_str(a) >= String::as_str(b)))
            }
            _ => Err(RuntimeError::InvalidOperation {
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
                            // TODO: If __index is a function, call it (future feature)
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
    pub fn set_field_with_meta(&self, key: String, value: Value) -> Result<(), RuntimeError> {
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
            _ => Err(RuntimeError::InvalidOperation {
                operator: "set_field".to_string(),
                left_type: self.get_type(),
                right_type: crate::value::ValueType::Null,
            }),
        }
    }

    /// Set metatable for an object
    pub fn set_metatable(&self, metatable: Value) -> Result<(), RuntimeError> {
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
            _ => Err(RuntimeError::InvalidOperation {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        let a = Value::int(5);
        let b = Value::int(3);

        assert_eq!(a.add(&b).unwrap(), Value::int(8));
        assert_eq!(a.subtract(&b).unwrap(), Value::int(2));
        assert_eq!(a.multiply(&b).unwrap(), Value::int(15));
        assert_eq!(a.divide(&b).unwrap(), Value::int(1));
        assert_eq!(a.modulo(&b).unwrap(), Value::int(2));
    }

    #[test]
    fn test_float_operations() {
        let a = Value::float(5.5);
        let b = Value::int(2);

        assert_eq!(a.add(&b).unwrap(), Value::float(7.5));
        assert_eq!(a.subtract(&b).unwrap(), Value::float(3.5));
        assert_eq!(a.multiply(&b).unwrap(), Value::float(11.0));
        assert_eq!(a.divide(&b).unwrap(), Value::float(2.75));
    }

    #[test]
    fn test_string_operations() {
        let a = Value::string("Hello".to_string());
        let b = Value::string(" World".to_string());

        assert_eq!(a.add(&b).unwrap(), Value::string("Hello World".to_string()));
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
        let float_val = Value::float(3.14);

        assert_eq!(int_val.to_float(), Some(42.0));
        assert_eq!(float_val.to_int(), Some(3));
        assert_eq!(int_val.to_int(), Some(42));
        assert_eq!(float_val.to_float(), Some(3.14));
    }

    #[test]
    fn test_truthy_values() {
        assert!(Value::bool(true).is_truthy());
        assert!(!Value::bool(false).is_truthy());
        assert!(Value::int(1).is_truthy());
        assert!(!Value::int(0).is_truthy());
        assert!(Value::float(1.0).is_truthy());
        assert!(!Value::float(0.0).is_truthy());
        assert!(Value::string("test".to_string()).is_truthy());
        assert!(!Value::string("".to_string()).is_truthy());
        assert!(!Value::null().is_truthy());
    }
}
