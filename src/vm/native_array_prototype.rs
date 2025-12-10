use super::*;

pub fn create_array_prototype() -> Value {
    use native_array_prototype::*;
    let mut table = crate::value::Table {
        data: IndexMap::<String, Value>::new(),
        metatable: None,
    };
    table
        .data
        .insert("__type".to_string(), Value::string("Array".to_string()));
    table
        .data
        // ...
        .insert(
            "push".to_string(),
            Value::NativeFunction(Rc::new(Box::new(native_array_push))),
        );
    table.data.insert(
        "pop".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_pop))),
    );
    table.data.insert(
        "len".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_len))),
    );

    let table_rc = Rc::new(std::cell::RefCell::new(table));
    let proto_val = Value::Object(table_rc.clone());

    // Set __index = self to allow method lookup on instances
    table_rc
        .borrow_mut()
        .data
        .insert("__index".to_string(), proto_val.clone());

    proto_val
}

pub fn native_array_push(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: ValueType::Null,
            operation: "push".into(),
        })?;
    }

    let obj = &args[0];
    if let Value::Object(table_rc) = obj {
        let mut table = table_rc.borrow_mut();
        // Since we are pushing to "Array", and we use IndexMap as dense vectorish thing:
        // Key is current len.
        let idx = table.data.len();
        let val = if args.len() > 1 { args[1].clone() } else { Value::Null };

        table.data.insert(idx.to_string(), val);
        return Ok(Value::Int((idx + 1) as i32));
    }
    Err(ValueError::TypeMismatch {
        expected: ValueType::Object,
        found: obj.get_type(),
        operation: "push".into(),
    })?
}

pub fn native_array_pop(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: ValueType::Null,
            operation: "pop".into(),
        })?;
    }
    let obj = &args[0];
    if let Value::Object(table_rc) = obj {
        let mut table = table_rc.borrow_mut();
        if let Some((_, val)) = table.data.pop() {
            return Ok(val);
        }
        return Ok(Value::Null);
    }
    Ok(Value::Null)
}

pub fn native_array_len(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    let obj = &args[0];
    if let Value::Object(table_rc) = obj {
        let table = table_rc.borrow();
        return Ok(Value::Int(table.data.len() as i32));
    }
    Ok(Value::Int(0))
}
