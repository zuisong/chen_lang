use super::*;

pub fn create_string_prototype() -> Value {
    use native_string_prototype::*;
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table
        .data
        .insert("__type".to_string(), Value::string("String".to_string()));
    table.data.insert(
        "len".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_len) as Box<NativeFnType>)),
    );
    table.data.insert(
        "trim".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_trim) as Box<NativeFnType>)),
    );
    table.data.insert(
        "upper".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_upper) as Box<NativeFnType>)),
    );
    table.data.insert(
        "lower".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_lower) as Box<NativeFnType>)),
    );

    let table_rc = Rc::new(std::cell::RefCell::new(table));
    let proto_val = Value::Object(table_rc.clone());

    // Set __index = self
    table_rc
        .borrow_mut()
        .data
        .insert("__index".to_string(), proto_val.clone());

    proto_val
}

pub fn native_string_len(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.chars().count() as i32)),
        _ => Err(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: args[0].get_type(),
            operation: "string.len".into(),
        })?,
    }
}

pub fn native_string_trim(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    match args.first() {
        Some(Value::String(s)) => Ok(Value::string(s.trim().to_string())),
        Some(v) => Err(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string.trim".into(),
        }
        .into()),
        None => Err(VMRuntimeError::StackUnderflow("string.trim".into())),
    }
}

pub fn native_string_upper(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    match args.first() {
        Some(Value::String(s)) => Ok(Value::string(s.to_uppercase())),
        Some(v) => Err(crate::vm::VMRuntimeError::ValueError(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string.upper".into(),
        })),
        None => Err(VMRuntimeError::StackUnderflow("string.upper".into())),
    }
}

pub fn native_string_lower(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    match args.first() {
        Some(Value::String(s)) => Ok(Value::string(s.to_lowercase())),
        Some(v) => Err(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string.lower".into(),
        })?,
        None => Err(VMRuntimeError::StackUnderflow("string.lower".into())),
    }
}
