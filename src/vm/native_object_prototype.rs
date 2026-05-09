use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{NativeFnType, Table, Value, ValueError, ValueType};
use crate::vm::error::VMRuntimeError;
use crate::vm::native_coroutine::native_coroutine_yield;
use crate::vm::{Fiber, VM};

pub fn create_object_prototype() -> Value {
    let mut data = IndexMap::new();

    data.insert(
        "keys".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_keys) as Box<NativeFnType>)),
    );
    data.insert(
        "iter".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_iter) as Box<NativeFnType>)),
    );
    data.insert(
        "entries".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_entries) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(Table { data, metatable: None })))
}

fn native_object_keys(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: ValueType::Null,
            operation: "keys".into(),
        }
        .into());
    }

    let obj = &args[0];
    if let Value::Object(table_rc) = obj {
        let table = table_rc.borrow();
        let mut data = IndexMap::new();
        for (i, k) in table.data.keys().enumerate() {
            data.insert(i.to_string(), Value::string(k.clone()));
        }

        let mut res_table = Table { data, metatable: None };
        if let Some(Value::Object(proto)) = Some(vm.array_prototype.clone()) {
            res_table.metatable = Some(proto);
        }

        return Ok(Value::Object(Rc::new(RefCell::new(res_table))));
    }

    Err(ValueError::TypeMismatch {
        expected: ValueType::Object,
        found: obj.get_type(),
        operation: "keys".into(),
    }
    .into())
}

fn native_object_iter(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Null);
    }
    let obj_val = args[0].clone();
    if let Value::Object(table_rc) = obj_val {
        let keys: Vec<String> = {
            let table = table_rc.borrow();
            table.data.keys().cloned().collect()
        };
        let index = Rc::new(RefCell::new(0));

        let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
            let mut idx = index.borrow_mut();
            if *idx < keys.len() {
                let key = &keys[*idx];
                let val = {
                    let table = table_rc.borrow();
                    table.data.get(key).cloned().unwrap_or(Value::Null)
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
        return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
    }
    Ok(Value::Null)
}

fn native_object_entries(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Null);
    }
    let obj_val = args[0].clone();
    if let Value::Object(table_rc) = obj_val {
        let keys: Vec<String> = {
            let table = table_rc.borrow();
            table.data.keys().cloned().collect()
        };
        let index = Rc::new(RefCell::new(0));

        let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
            let mut idx = index.borrow_mut();
            if *idx < keys.len() {
                let key = &keys[*idx];
                let val = {
                    let table = table_rc.borrow();
                    table.data.get(key).cloned().unwrap_or(Value::Null)
                };

                let mut data = IndexMap::new();
                data.insert("key".to_string(), Value::string(key.clone()));
                data.insert("value".to_string(), val);
                let pair = Value::Object(Rc::new(RefCell::new(Table { data, metatable: None })));

                *idx += 1;
                return native_coroutine_yield(vm, vec![pair]);
            }
            Ok(Value::Null)
        };

        let mut fiber = Fiber::new();
        let nf_rc = Rc::new(Box::new(iter_body) as Box<NativeFnType>);
        fiber.native_function = Some(nf_rc.clone());
        fiber.stack.push(Value::NativeFunction(nf_rc));
        return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
    }
    Ok(Value::Null)
}
