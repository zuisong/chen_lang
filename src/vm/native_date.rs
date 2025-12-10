use super::*;

pub fn create_date_object() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table
        .data
        .insert("__type".to_string(), Value::string("Date".to_string()));
    table.data.insert(
        "new".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_date_new) as Box<NativeFnType>)),
    );
    table.data.insert(
        "format".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_date_format) as Box<NativeFnType>)),
    );
    table.data.insert(
        "timestamp".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_date_timestamp) as Box<NativeFnType>)),
    );

    let table_rc = Rc::new(std::cell::RefCell::new(table));
    let val = Value::Object(table_rc.clone());
    // Class acts as prototype for instances
    table_rc.borrow_mut().data.insert("__index".to_string(), val.clone());
    val
}

fn native_date_new(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut ts = Timestamp::now().as_millisecond();
    // args[0] is Date class itself
    if args.len() > 1 {
        match &args[1] {
            // Support creating from string or timestamp if we had it
            Value::String(s) => {
                if let Ok(parsed) = s.parse::<Timestamp>() {
                    ts = parsed.as_millisecond();
                }
            }
            // Temporarily support int if fits?
            Value::Int(n) => ts = *n as i64,
            _ => {}
        }
    }

    // Create Instance
    let mut data = IndexMap::new();
    data.insert("__timestamp".to_string(), Value::string(ts.to_string()));
    data.insert("__type".to_string(), Value::string("Date".to_string()));

    let table_rc = Rc::new(std::cell::RefCell::new(crate::value::Table { data, metatable: None }));

    // Set prototype
    if let Some(Value::Object(cls_rc)) = args.first() {
        table_rc.borrow_mut().metatable = Some(cls_rc.clone());
    }

    Ok(Value::Object(table_rc))
}

fn native_date_format(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    // args[0] is instance
    if let Some(obj) = args.first()
        && let Value::Object(table_rc) = obj
    {
        let table = table_rc.borrow();
        if let Some(Value::String(ts_str)) = table.data.get("__timestamp")
            && let Ok(ts_val) = ts_str.parse::<i64>()
            && let Ok(ts) = Timestamp::from_millisecond(ts_val)
        {
            // Default format or arg
            let fmt = if args.len() > 1 {
                if let Value::String(s) = &args[1] {
                    s.to_string()
                } else {
                    "%Y-%m-%d %H:%M:%S".to_string()
                }
            } else {
                "%Y-%m-%d %H:%M:%S".to_string()
            };
            // Use system timezone for display
            let zoned = ts.to_zoned(jiff::tz::TimeZone::system());
            return Ok(Value::string(zoned.strftime(&fmt).to_string()));
        }
    }
    Ok(Value::Null)
}

fn native_date_timestamp(args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if let Some(obj) = args.first()
        && let Value::Object(table_rc) = obj
    {
        let table = table_rc.borrow();
        if let Some(Value::String(ts_str)) = table.data.get("__timestamp") {
            if let Ok(ts_val) = ts_str.parse::<i32>() {
                return Ok(Value::Int(ts_val));
            }
            // Return as string if overflow i32?
            return Ok(Value::string(ts_str.to_string()));
        }
    }
    Ok(Value::Null)
}
