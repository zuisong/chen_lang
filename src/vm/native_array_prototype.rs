use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::vm::native_coroutine::native_coroutine_yield;

pub fn create_array_prototype() -> Value {
    use native_array_prototype::*;
    let mut table = crate::value::Table {
        data: IndexMap::<String, Value>::new(),
        metatable: None,
    };
    table
        .data
        .insert("__type".to_string(), Value::string("Array".to_string()));

    table.data.insert(
        "push".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_push) as Box<NativeFnType>)),
    );
    table.data.insert(
        "pop".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_pop) as Box<NativeFnType>)),
    );
    table.data.insert(
        "len".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_len) as Box<NativeFnType>)),
    );
    table.data.insert(
        "iter".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_iter) as Box<NativeFnType>)),
    );
    table.data.insert(
        "entries".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_array_entries) as Box<NativeFnType>)),
    );

    let table_rc = Rc::new(RefCell::new(table));
    let proto_val = Value::Object(table_rc.clone());

    // Set __index = self to allow method lookup on instances
    table_rc
        .borrow_mut()
        .data
        .insert("__index".to_string(), proto_val.clone());

    proto_val
}

pub fn native_array_push(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
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

pub fn native_array_pop(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
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

pub fn native_array_len(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
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

pub fn native_array_iter(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Null);
    }
    let table_rc = match &args[0] {
        Value::Object(t) => t.clone(),
        _ => return Ok(Value::Null),
    };

    let len = {
        let table = table_rc.borrow();
        table.data.len()
    };
    let index = Rc::new(RefCell::new(0));

    let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
        let mut idx = index.borrow_mut();
        if *idx < len {
            let val = {
                let table = table_rc.borrow();
                table
                    .data
                    .get_index(*idx)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null)
            };
            *idx += 1;
            return native_coroutine_yield(vm, vec![val]);
        }
        Ok(Value::Null)
    };

    let mut fiber = Fiber::new();
    let nf_rc = Rc::new(Box::new(iter_body) as Box<NativeFnType>);
    fiber.native_function = Some(nf_rc.clone());
    fiber.stack.push(Value::NativeFunction(nf_rc));
    Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))))
}

pub fn native_array_entries(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Null);
    }
    let table_rc = match &args[0] {
        Value::Object(t) => t.clone(),
        _ => return Ok(Value::Null),
    };

    let len = {
        let table = table_rc.borrow();
        table.data.len()
    };
    let index = Rc::new(RefCell::new(0));

    let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
        let mut idx = index.borrow_mut();
        if *idx < len {
            let val = {
                let table = table_rc.borrow();
                table
                    .data
                    .get_index(*idx)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null)
            };

            let mut data = IndexMap::new();
            data.insert("key".to_string(), Value::Int(*idx as i32));
            data.insert("value".to_string(), val);
            let pair = Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None })));

            *idx += 1;
            return native_coroutine_yield(vm, vec![pair]);
        }
        Ok(Value::Null)
    };

    let mut fiber = Fiber::new();
    let nf_rc = Rc::new(Box::new(iter_body) as Box<NativeFnType>);
    fiber.native_function = Some(nf_rc.clone());
    fiber.stack.push(Value::NativeFunction(nf_rc));
    Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))))
}
