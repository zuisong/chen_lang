use crate::value::NativeContext;
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
        "trim".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_trim) as Box<NativeFnType>)),
    );
    table.data.insert(
        "upper".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_upper) as Box<NativeFnType>)),
    );
    table.data.insert(
        "toUpperCase".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_upper) as Box<NativeFnType>)),
    );
    table.data.insert(
        "lower".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_lower) as Box<NativeFnType>)),
    );
    table.data.insert(
        "toLowerCase".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_lower) as Box<NativeFnType>)),
    );
    table.data.insert(
        "iter".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_iter) as Box<NativeFnType>)),
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

fn string_receiver<'a>(ctx: &'a NativeContext) -> Option<&'a Value> {
    ctx.this.as_ref().or_else(|| ctx.args.first())
}

pub fn native_string_trim(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    match string_receiver(&ctx) {
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

pub fn native_string_upper(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    match string_receiver(&ctx) {
        Some(Value::String(s)) => Ok(Value::string(s.to_uppercase())),
        Some(v) => Err(crate::vm::VMRuntimeError::ValueError(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string.upper".into(),
        })),
        None => Err(VMRuntimeError::StackUnderflow("string.upper".into())),
    }
}

pub fn native_string_lower(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    match string_receiver(&ctx) {
        Some(Value::String(s)) => Ok(Value::string(s.to_lowercase())),
        Some(v) => Err(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string.lower".into(),
        })?,
        None => Err(VMRuntimeError::StackUnderflow("string.lower".into())),
    }
}

fn native_string_iter(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let Some(s_val) = string_receiver(&ctx).cloned() else {
        return Ok(Value::Null);
    };
    if let Value::String(s) = s_val {
        let chars: Vec<String> = s.chars().map(|c| c.to_string()).collect();
        let index = Rc::new(RefCell::new(0));

        let next_body = move |_vm: &mut VM, _ctx: NativeContext| {
            let mut idx = index.borrow_mut();
            let mut result_data = IndexMap::new();
            if *idx < chars.len() {
                let val = Value::string(chars[*idx].clone());
                *idx += 1;
                result_data.insert("value".to_string(), val);
                result_data.insert("done".to_string(), Value::Bool(false));
            } else {
                result_data.insert("value".to_string(), Value::Null);
                result_data.insert("done".to_string(), Value::Bool(true));
            }
            Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table { data: result_data, metatable: None }))))
        };

        let mut data = IndexMap::new();
        data.insert(
            "next".to_string(),
            Value::NativeFunction(Rc::new(Box::new(next_body) as Box<NativeFnType>)),
        );
        data.insert(
            "iter".to_string(),
            Value::NativeFunction(Rc::new(Box::new(native_iter_self) as Box<NativeFnType>)),
        );
        return Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None }))));
    }
    Ok(Value::Null)
}
