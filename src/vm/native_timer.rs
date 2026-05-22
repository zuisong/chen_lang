use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use indexmap::IndexMap;

use crate::value::{NativeContext, NativeFnType, Value, ValueError, ValueType};
use crate::vm::VM;
use crate::vm::error::VMRuntimeError;

pub fn create_timer_object() -> Value {
    let timer = Value::Object(Rc::new(RefCell::new(crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    })));

    if let Value::Object(table_rc) = &timer {
        let mut table = table_rc.borrow_mut();
        let sleep_fn =
            Value::NativeFunction(Rc::new(
                Box::new(|vm: &mut VM, ctx: NativeContext| native_timer_sleep(vm, ctx)) as Box<NativeFnType>,
            ));

        table.data.insert("sleep".to_string(), sleep_fn);
    }

    timer
}

fn native_timer_sleep(vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    let args = ctx.args;
    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Int,
            found: ValueType::Null,
            operation: "timer.sleep".into(),
        }
        .into());
    }

    let ms = args[0].to_int().ok_or_else(|| ValueError::TypeMismatch {
        expected: ValueType::Int,
        found: args[0].get_type(),
        operation: "timer.sleep".into(),
    })?;

    if ms < 0 {
        return Err(ValueError::InvalidOperation {
            operator: "timer.sleep".into(),
            left_type: ValueType::Int,
            right_type: ValueType::Null,
        }
        .into());
    }

    let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
    let promise_val = Value::Promise(promise.clone());

    let duration = Duration::from_millis(ms as u64);
    vm.async_state.spawn_promise_task(promise, async move {
        crate::vm::rt::sleep(duration).await;
        Ok(Value::null())
    });

    Ok(promise_val)
}
