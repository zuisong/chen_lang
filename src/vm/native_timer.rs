use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use indexmap::IndexMap;

use crate::value::{NativeFnType, Value, ValueError, ValueType};
use crate::vm::error::VMRuntimeError;
use crate::vm::{Fiber, FiberState, VM};

pub fn create_timer_object() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "sleep".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_timer_sleep) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn native_timer_sleep(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
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

    // Capture current fiber logic
    let current_fiber_rc = if let Some(c) = &vm.current_fiber {
        c.clone()
    } else {
        // Root Fiber Promotion: valid for top-level code or when no fiber is explicitly created.
        let f = Rc::new(RefCell::new(Fiber::new()));
        vm.current_fiber = Some(f.clone());
        f
    };

    let mut current_fiber = current_fiber_rc.borrow_mut();

    // IMPORTANT: We must increment PC before saving state, because `execute_from`
    // will normally increment PC after instruction execution. But since we are breaking
    // the loop via `Yield`, the automatic increment is skipped (or must be handled).
    // If we save state NOW, we save `pc` pointing to the `Call` instruction.
    // When resumed, we execute `Call` again -> Infinite Loop.
    vm.pc += 1;

    // 1. Save state
    vm.save_state_to_fiber(&mut current_fiber);
    current_fiber.state = FiberState::Suspended;
    drop(current_fiber); // Release borrow

    // 2. Spawn Async Task
    let duration = Duration::from_millis(ms as u64);
    vm.async_state.spawn_sleep(duration, current_fiber_rc.clone());

    // 3. Signal VM to Yield
    // Note: The VM loop must handle `VMRuntimeError::Yield` by NOT incrementing PC again.
    Err(VMRuntimeError::Yield)
}
