use std::cell::RefCell;

use super::*;
pub fn create_json_object() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "parse".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_json_parse) as Box<NativeFnType>)),
    );
    table.data.insert(
        "stringify".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_json_stringify) as Box<NativeFnType>)),
    );
    Value::Object(Rc::new(RefCell::new(table)))
}

fn native_json_parse(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    // args[0] is JSON object, args[1] is string
    if let Some(Value::String(s)) = args.get(1) {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|_e| ValueError::InvalidOperation {
            operator: "JSON.parse".into(),
            left_type: ValueType::String,
            right_type: ValueType::Null,
        })?;
        return Ok(json_to_chen(v));
    }
    Ok(Value::Null)
}

fn native_json_stringify(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if let Some(val) = args.get(1) {
        let j = chen_to_json(val);
        return Ok(Value::string(j.to_string()));
    }
    Ok(Value::Null)
}

fn json_to_chen(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::bool(b), // assuming wrapper
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Value::Int(n.as_i64().unwrap() as i32) // Truncate :S
            } else {
                Value::Float(Decimal::from_f64(n.as_f64().unwrap()).unwrap_or_default())
            }
        }
        serde_json::Value::String(s) => Value::string(s),
        serde_json::Value::Array(arr) => {
            let mut data = IndexMap::new();
            for (i, val) in arr.into_iter().enumerate() {
                data.insert(i.to_string(), json_to_chen(val));
            }
            // We should ideally set Array prototype here... but we don't have access to VM.array_prototype
            // Objects created by JSON.parse won't have methods unless we fix this.
            // Limitation accepted for now.
            Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None })))
        }
        serde_json::Value::Object(obj) => {
            let mut data = IndexMap::new();
            for (k, v) in obj {
                data.insert(k, json_to_chen(v));
            }
            Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None })))
        }
    }
}

fn chen_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::to_value(f).unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.to_string()),
        Value::Object(rc) => {
            let table = rc.borrow();

            // Special handling for Date objects
            if let Some(Value::String(type_name)) = table.data.get("__type")
                && **type_name == "Date"
                && let Some(Value::String(ts_str)) = table.data.get("__timestamp")
                && let Ok(ts_val) = ts_str.parse::<i64>()
                && let Ok(ts) = Timestamp::from_millisecond(ts_val)
            {
                // Default JSON format for Date is ISO 8601 string
                return serde_json::Value::String(ts.to_string());
            }

            // Check if array-like (all numeric keys)
            // Simple heuristic: if empty or has "0"
            let is_array = !table.data.is_empty() && table.data.contains_key("0");
            if is_array {
                let mut arr = Vec::new();
                // Naive iteration (keys might not be sorted or complete)
                // But IndexMap preserves insertion order.
                // If it's a valid Array object, "0", "1"...
                for (_, val) in &table.data {
                    if let Value::String(_) = val {
                        // Skip non-value fields like __type?
                        // Actually Arrays have properties only "0".."N".
                        // But we might have "__index" etc? No, those are in metatable/prototype.
                        // Only explicit fields are in data.
                    }
                    arr.push(chen_to_json(val));
                }
                serde_json::Value::Array(arr)
            } else {
                let mut map = serde_json::Map::new();
                for (k, val) in &table.data {
                    map.insert(k.clone(), chen_to_json(val));
                }
                serde_json::Value::Object(map)
            }
        }
        _ => serde_json::Value::Null, // Function etc
    }
}
