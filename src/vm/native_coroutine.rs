use super::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::vm::{Fiber, FiberState};

pub fn create_coroutine_object() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "create".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_create) as Box<NativeFnType>)),
    );
    table.data.insert(
        "resume".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_resume) as Box<NativeFnType>)),
    );
     table.data.insert(
        "status".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_status) as Box<NativeFnType>)),
    );
    table.data.insert(
        "yield".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_yield) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn native_coroutine_create(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    if let Some(co_obj) = vm.variables.get("coroutine") {
        if !args.is_empty() && &args[0] == co_obj {
            args.remove(0);
        }
    }

    if args.is_empty() {
         return Err(ValueError::TypeMismatch {
            expected: ValueType::Function,
            found: ValueType::Null,
            operation: "coroutine.create".into(),
        }
        .into());
    }

    let func_val = &args[0];
    // Allow Reference to Function or NativeFunction
    if let Value::Function(_) = func_val {
        // Create a new fiber
        let mut fiber = Fiber::new();
        // Push all arguments (function + params) to stack
        for arg in args {
            fiber.stack.push(arg);
        }
        return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
    }
    
    // Also support NativeFunction?
     if let Value::NativeFunction(_) = func_val {
         let mut fiber = Fiber::new();
         for arg in args {
             fiber.stack.push(arg);
         }
         return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
    }

    Err(ValueError::TypeMismatch {
        expected: ValueType::Function,
        found: func_val.get_type(),
        operation: "coroutine.create".into(),
    }
    .into())
}

fn native_coroutine_resume(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    if let Some(co_obj) = vm.variables.get("coroutine") {
        if !args.is_empty() && &args[0] == co_obj {
            args.remove(0);
        }
    }

    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: ValueType::Null,
            operation: "coroutine.resume".into(),
        }.into());
    }
    
    let co_val = &args[0];
    let fiber_rc = if let Value::Coroutine(c) = co_val {
        c.clone()
    } else {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: co_val.get_type(),
            operation: "coroutine.resume".into(),
        }
        .into());
    };
    
    let mut fiber = fiber_rc.borrow_mut();
    
    if fiber.state == FiberState::Dead {
        return Ok(Value::bool(false)); // Cannot resume dead fiber
    }
    
    if fiber.state == FiberState::Running {
         return Err(ValueError::InvalidOperation {
            operator: "resume".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }.into());
    }
    
    // Passed args (skipping coroutine itself)
    let passed_args = &args[1..];
    
    // Check if it's a new fiber (PC == 0 and Call Stack is empty)
    // Stack length can be > 1 if arguments are passed during creation.
    let is_new = fiber.pc == 0 && fiber.call_stack.is_empty();

    // If new, setup the call frame
    if is_new {
        let func_val = fiber.stack.remove(0);
        
        let func_name = if let Value::Function(name) = func_val {
            name
        } else if let Value::NativeFunction(_) = func_val {
             return Err(ValueError::InvalidOperation {
                operator: "resume native coroutine not fully supported".into(),
                left_type: ValueType::Function,
                right_type: ValueType::Null,
            }.into());
        } else {
             return Err(ValueError::TypeMismatch {
                expected: ValueType::Function,
                found: func_val.get_type(),
                operation: "resume".into(),
            }.into());
        };

        // Resolve function address
        let program = vm.program.as_ref().ok_or_else(|| VMRuntimeError::UndefinedVariable("Program not loaded in VM".into()))?;
        let func_label = format!("func_{}", func_name);
        
        if let Some(sym) = program.syms.get(&func_label) {
            // Setup stack frame
             // 1. Push Args to stack
            for arg in passed_args {
                fiber.stack.push(arg.clone());
            }
            
            // 2. Set fp
            fiber.fp = 0; 
            let new_size = fiber.fp + sym.nlocals;
            fiber.stack.resize(new_size, Value::null());
            
            // 3. Set PC (-1 because loop increments)
            fiber.pc = (sym.location as usize) - 1;
        } else {
             return Err(VMRuntimeError::UndefinedVariable(format!("function: {}", func_name)));
        }
    } else {
        // Resuming yielded fiber
        // We used to push args here.
        // But we want the return value of `native_coroutine_resume` to be part of the stack push inside VM loop.
        // However, `native_coroutine_resume` returns to the CALLER context, not the SUSPENDED FIBER context?
        // Wait, execute_instruction calls native_fn. native_fn returns Result<Value>.
        // Then execute_instruction pushes Result to self.stack.
        // The `self.stack` it pushes to is the one AFTER context switch.
        // So `resume` returns value into the FIBER.
        // So we SHOULD return `args[1]` from native_coroutine_resume.
        // And we should NOT push manually here.
    }

    // Perform Context Switch
    
    let caller_state = Rc::new(RefCell::new(Fiber::new()));
    vm.save_state_to_fiber(&mut caller_state.borrow_mut());
    
    if let Some(current) = &vm.current_fiber {
        let mut c = current.borrow_mut();
        vm.save_state_to_fiber(&mut c);
        c.state = FiberState::Suspended;
    } else {
        caller_state.borrow_mut().state = FiberState::Suspended;
    }
    
    let caller_rc = if let Some(current) = &vm.current_fiber {
        current.clone()
    } else {
        caller_state
    };
    
    fiber.caller = Some(caller_rc);
    fiber.state = FiberState::Running;
    
    // Load new state
    vm.load_state_from_fiber(&fiber);
    vm.current_fiber = Some(fiber_rc.clone());

    if is_new {
        // For new fiber, we manually pushed args.
        // Return Null. CallStack will push Null next.
        // The first few instructions of function are usually Moves for args.
        // Or if logic relies on stack position, Null at top is fine as long as fp=0 and args are at 0, 1 etc.
        Ok(Value::Null)
    } else {
        // For suspended fiber, we want `let v = yield ...` to get the value passed to resume.
        // The value returned here is pushed to stack.
        // So return passed_args[0].
        if passed_args.is_empty() {
             Ok(Value::Null)
        } else {
             Ok(passed_args[0].clone())
        }
    }
}

fn native_coroutine_yield(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    if let Some(co_obj) = vm.variables.get("coroutine") {
        if !args.is_empty() && &args[0] == co_obj {
            args.remove(0);
        }
    }

    let current_fiber_rc = if let Some(c) = &vm.current_fiber {
        c.clone()
    } else {
          return Err(ValueError::InvalidOperation {
            operator: "yield from root".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }.into());
    };
    
    let mut current_fiber = current_fiber_rc.borrow_mut();
    
    let caller_rc = if let Some(c) = &current_fiber.caller {
        c.clone()
    } else {
          return Err(ValueError::InvalidOperation {
            operator: "yield without caller".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }.into());
    };
    
    // Save current state
    vm.save_state_to_fiber(&mut current_fiber);
    current_fiber.state = FiberState::Suspended;
    
    drop(current_fiber); // Release borrow
    
    // Load caller
    let caller = caller_rc.borrow();
    vm.load_state_from_fiber(&caller);
    
    vm.current_fiber = Some(caller_rc.clone());
    
    // We want `resume(...)` to return the value passed to `yield`.
    // The value returned here is pushed to Caller stack.
    if args.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(args[0].clone())
    }
}

fn native_coroutine_status(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    if let Some(co_obj) = vm.variables.get("coroutine") {
        if !args.is_empty() && &args[0] == co_obj {
            args.remove(0);
        }
    }

    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: ValueType::Null,
            operation: "coroutine.status".into(),
        }
        .into());
    }

    let co_val = &args[0];
    let fiber_rc = if let Value::Coroutine(c) = co_val {
        c.clone()
    } else {
         return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: co_val.get_type(),
            operation: "coroutine.status".into(),
        }
        .into());
    };

    let status = match fiber_rc.borrow().state {
        FiberState::Running => "running",
        FiberState::Suspended => "suspended",
        FiberState::Dead => "dead",
    };

    Ok(Value::string(status.to_string()))
}
