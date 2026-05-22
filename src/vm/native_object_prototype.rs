use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{NativeContext, NativeFnType, Table, Value, ValueError, ValueType};
use crate::vm::error::VMRuntimeError;
use crate::vm::{VM, native_iter_self};

pub fn create_object_prototype() -> Value {
    let mut data = IndexMap::new();

    data.insert(
        "create".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_create) as Box<NativeFnType>)),
    );
    data.insert(
        "keys".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_keys) as Box<NativeFnType>)),
    );
    data.insert(
        "@@iterator".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_iter) as Box<NativeFnType>)),
    );
    data.insert(
        "entries".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_object_entries) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(Table { data, metatable: None })))
}

fn object_arg(ctx: &NativeContext, operation: &str) -> Result<Value, VMRuntimeError> {
    let Some(value) = ctx.args.first().cloned().or_else(|| ctx.this.clone()) else {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: ValueType::Null,
            operation: operation.into(),
        }
        .into());
    };

    if matches!(value, Value::Object(_)) {
        Ok(value)
    } else {
        Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: value.get_type(),
            operation: operation.into(),
        }
        .into())
    }
}

fn array_from_values(vm: &VM, values: impl IntoIterator<Item = Value>) -> Value {
    let mut table = Table {
        data: IndexMap::new(),
        metatable: match &vm.array_prototype {
            Value::Object(proto) => Some(proto.clone()),
            _ => None,
        },
    };

    for (i, value) in values.into_iter().enumerate() {
        table.data.insert(i.to_string(), value);
    }

    Value::Object(Rc::new(RefCell::new(table)))
}

fn native_object_create(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let proto = object_arg(&ctx, "Object.create")?;
    let mut table = Table {
        data: IndexMap::new(),
        metatable: None,
    };

    if let Value::Object(proto_rc) = proto {
        let mut meta_data = IndexMap::new();
        meta_data.insert("__index".to_string(), Value::Object(proto_rc));
        table.metatable = Some(Rc::new(RefCell::new(Table {
            data: meta_data,
            metatable: None,
        })));
    }

    Ok(Value::Object(Rc::new(RefCell::new(table))))
}

fn native_object_keys(vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let obj = object_arg(&ctx, "Object.keys")?;
    if let Value::Object(table_rc) = obj {
        let table = table_rc.borrow();
        return Ok(array_from_values(vm, table.data.keys().cloned().map(Value::string)));
    }

    unreachable!("object_arg already validated object value")
}

fn native_object_iter(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let obj_val = ctx.args.first().cloned().or(ctx.this).unwrap_or(Value::Null);
    if let Value::Object(table_rc) = obj_val {
        let keys: Vec<String> = {
            let table = table_rc.borrow();
            table.data.keys().cloned().collect()
        };
        let index = Rc::new(RefCell::new(0));

        let next_body = move |_vm: &mut VM, _ctx: NativeContext| {
            let mut idx = index.borrow_mut();
            let mut result_data = IndexMap::new();
            if *idx < keys.len() {
                let key = &keys[*idx];
                let val = {
                    let table = table_rc.borrow();
                    table.data.get(key).cloned().unwrap_or(Value::Null)
                };
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
            "@@iterator".to_string(),
            Value::NativeFunction(Rc::new(Box::new(native_iter_self) as Box<NativeFnType>)),
        );
        return Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None }))));
    }
    Ok(Value::Null)
}

fn native_object_entries(vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let obj = object_arg(&ctx, "Object.entries")?;
    if let Value::Object(table_rc) = obj {
        let table = table_rc.borrow();
        let entries = table.data.iter().map(|(key, value)| {
            let mut pair = Table {
                data: IndexMap::new(),
                metatable: match &vm.array_prototype {
                    Value::Object(proto) => Some(proto.clone()),
                    _ => None,
                },
            };
            pair.data.insert("0".to_string(), Value::string(key.clone()));
            pair.data.insert("1".to_string(), value.clone());
            Value::Object(Rc::new(RefCell::new(pair)))
        });

        return Ok(array_from_values(vm, entries));
    }

    unreachable!("object_arg already validated object value")
}
