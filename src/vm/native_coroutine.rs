use std::cell::RefCell;
use std::rc::Rc;

use super::*;
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
    table.data.insert(
        "spawn".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_spawn) as Box<NativeFnType>)),
    );
    table.data.insert(
        "await_all".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_coroutine_await_all) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn native_coroutine_create(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
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
    // Allow Reference to Fn or NativeFunction
    match func_val {
        Value::Fn(_) => {
            // Create a new fiber
            let mut fiber = Fiber::new();
            // Push all arguments (function + params) to stack
            for arg in args {
                fiber.stack.push(arg);
            }
            return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
        }
        Value::NativeFunction(_) => {
            let mut fiber = Fiber::new();
            for arg in args {
                fiber.stack.push(arg);
            }
            return Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))));
        }
        _ => {}
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
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
    }

    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: ValueType::Null,
            operation: "coroutine.resume".into(),
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
        }
        .into());
    }

    // Passed args (skipping coroutine itself)
    let passed_args = &args[1..];

    // Check if it's a new fiber (PC == 0 and Call Stack is empty)
    // Stack length can be > 1 if arguments are passed during creation.
    let is_new = fiber.pc == 0 && fiber.call_stack.is_empty();

    // If new, setup the call frame
    if is_new {
        let func_val = fiber.stack.remove(0);

        let closure = match func_val {
            Value::Fn(c) => c,
            Value::NativeFunction(_) => {
                return Err(ValueError::InvalidOperation {
                    operator: "resume native coroutine not fully supported".into(),
                    left_type: ValueType::Function,
                    right_type: ValueType::Null,
                }
                .into());
            }
            _ => {
                return Err(ValueError::TypeMismatch {
                    expected: ValueType::Function,
                    found: func_val.get_type(),
                    operation: "resume".into(),
                }
                .into());
            }
        };

        // Resolve function address
        let program = closure.program.clone();
        let func_label = format!("func_{}", closure.name);

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

            // 4. Set program and current_closure for the fiber
            fiber.program = Some(program);
            fiber.current_closure = Some(closure);
        } else {
            return Err(VMRuntimeError::UndefinedVariable(format!("function: {}", closure.name)));
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
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
    }

    let current_fiber_rc = if let Some(c) = &vm.current_fiber {
        c.clone()
    } else {
        return Err(ValueError::InvalidOperation {
            operator: "yield from root".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }
        .into());
    };

    let mut current_fiber = current_fiber_rc.borrow_mut();

    let caller_rc = if let Some(c) = &current_fiber.caller {
        c.clone()
    } else {
        return Err(ValueError::InvalidOperation {
            operator: "yield without caller".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }
        .into());
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
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
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

/// coroutine.spawn(co, args...) - 非阻塞启动协程
/// 将协程放入 ready_queue，由事件循环调度执行
fn native_coroutine_spawn(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    // 移除 self 参数（coroutine 对象本身）
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
    }

    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Coroutine,
            found: ValueType::Null,
            operation: "coroutine.spawn".into(),
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
            operation: "coroutine.spawn".into(),
        }
        .into());
    };

    let mut fiber = fiber_rc.borrow_mut();

    if fiber.state == FiberState::Dead {
        return Err(ValueError::InvalidOperation {
            operator: "spawn dead coroutine".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }
        .into());
    }

    if fiber.state == FiberState::Running {
        return Err(ValueError::InvalidOperation {
            operator: "spawn running coroutine".into(),
            left_type: ValueType::Coroutine,
            right_type: ValueType::Null,
        }
        .into());
    }

    // 传递的参数（跳过协程本身）
    let passed_args = &args[1..];

    // 检查是否是新创建的协程
    let is_new = fiber.pc == 0 && fiber.call_stack.is_empty();

    if is_new {
        let func_val = fiber.stack.remove(0);

        let closure = match func_val {
            Value::Fn(c) => c,
            _ => {
                return Err(ValueError::TypeMismatch {
                    expected: ValueType::Function,
                    found: func_val.get_type(),
                    operation: "spawn".into(),
                }
                .into());
            }
        };

        // 直接使用闭包中已存储的符号信息，而不是重新查找
        let sym = &closure.func_symbol;
        let program = closure.program.clone();

        // 设置栈帧
        for arg in passed_args {
            fiber.stack.push(arg.clone());
        }

        fiber.fp = 0;
        let new_size = fiber.fp + sym.nlocals;
        fiber.stack.resize(new_size, Value::null());

        // 设置 PC（不需要 -1，因为 execute_from 直接从 pc 开始执行）
        fiber.pc = sym.location as usize;

        fiber.program = Some(program);
        fiber.current_closure = Some(closure);
        fiber.state = FiberState::Suspended;
        // 标记：新 spawn 的协程，恢复时不需要 push 值
        fiber.skip_push_on_resume = true;
        // 标记：spawn 创建的协程，完成时需要减少 pending_tasks
        fiber.is_spawned = true;
    }

    // 不设置 caller，让协程独立运行
    fiber.caller = None;

    drop(fiber);

    // 将协程放入 ready_queue，由事件循环调度
    vm.async_state
        .ready_queue
        .borrow_mut()
        .push_back((fiber_rc.clone(), Value::Null));
    *vm.async_state.pending_tasks.borrow_mut() += 1;

    // 立即返回协程对象（不阻塞）
    Ok(args[0].clone())
}

/// coroutine.await_all([co1, co2, ...]) - 等待所有协程完成
/// 返回所有协程的结果数组
fn native_coroutine_await_all(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut args = args;
    // 移除 self 参数
    if let Some(co_obj) = vm.variables.get("coroutine")
        && !args.is_empty()
        && &args[0] == co_obj
    {
        args.remove(0);
    }

    if args.is_empty() {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: ValueType::Null,
            operation: "coroutine.await_all".into(),
        }
        .into());
    }

    // 获取协程数组
    let coroutines_val = &args[0];
    let coroutines: Vec<Rc<RefCell<Fiber>>> = if let Value::Object(obj) = coroutines_val {
        let obj = obj.borrow();
        let mut list = Vec::new();
        for i in 0.. {
            if let Some(val) = obj.data.get(&i.to_string()) {
                if let Value::Coroutine(c) = val {
                    list.push(c.clone());
                } else {
                    return Err(ValueError::TypeMismatch {
                        expected: ValueType::Coroutine,
                        found: val.get_type(),
                        operation: "coroutine.await_all".into(),
                    }
                    .into());
                }
            } else {
                break;
            }
        }
        list
    } else {
        return Err(ValueError::TypeMismatch {
            expected: ValueType::Object,
            found: coroutines_val.get_type(),
            operation: "coroutine.await_all".into(),
        }
        .into());
    };

    if coroutines.is_empty() {
        // 空数组，直接返回空数组
        return Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table {
            data: IndexMap::new(),
            metatable: None,
        }))));
    }

    // 检查是否所有协程都已完成
    let all_dead = coroutines.iter().all(|c| c.borrow().state == FiberState::Dead);

    if all_dead {
        // 所有协程已完成，收集结果
        let mut results = IndexMap::new();
        for (i, co) in coroutines.iter().enumerate() {
            let result = co.borrow().result.clone().unwrap_or(Value::Null);
            results.insert(i.to_string(), result);
        }
        return Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table {
            data: results,
            metatable: None,
        }))));
    }

    // 还有协程未完成，需要 yield 并等待
    // 保存当前 Fiber 状态
    let current_fiber_rc = if let Some(c) = &vm.current_fiber {
        c.clone()
    } else {
        // 创建一个新的 Fiber 来表示主执行流程
        let f = Rc::new(RefCell::new(Fiber::new()));
        vm.current_fiber = Some(f.clone());
        f
    };

    let mut current_fiber = current_fiber_rc.borrow_mut();

    // PC 指向下一条指令（await_all 调用之后）
    vm.pc += 1;

    vm.save_state_to_fiber(&mut current_fiber);
    current_fiber.state = FiberState::Suspended;
    drop(current_fiber);

    // 将等待的协程列表和当前 Fiber 关联起来
    // 使用一个监控任务来检查何时所有协程完成
    let queue = vm.async_state.ready_queue.clone();
    let pending = vm.async_state.pending_tasks.clone();
    let notify = vm.async_state.notify.clone();
    let fiber_for_await = current_fiber_rc.clone();
    let coroutines_for_check = coroutines.clone();

    *pending.borrow_mut() += 1;

    // 启动一个监控任务
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::spawn_local(async move {
        loop {
            // 检查是否所有协程都已完成
            let all_done = coroutines_for_check
                .iter()
                .all(|c| c.borrow().state == FiberState::Dead);

            if all_done {
                // 收集结果
                let mut results = IndexMap::new();
                for (i, co) in coroutines_for_check.iter().enumerate() {
                    let result = co.borrow().result.clone().unwrap_or(Value::Null);
                    results.insert(i.to_string(), result);
                }
                let result_value = Value::Object(Rc::new(RefCell::new(crate::value::Table {
                    data: results,
                    metatable: None,
                })));

                queue.borrow_mut().push_back((fiber_for_await, result_value));
                *pending.borrow_mut() -= 1;
                notify.notify_one();
                break;
            }

            // 等待通知
            notify.notified().await;
        }
    });

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let all_done = coroutines_for_check
                .iter()
                .all(|c| c.borrow().state == FiberState::Dead);

            if all_done {
                let mut results = IndexMap::new();
                for (i, co) in coroutines_for_check.iter().enumerate() {
                    let result = co.borrow().result.clone().unwrap_or(Value::Null);
                    results.insert(i.to_string(), result);
                }
                let result_value = Value::Object(Rc::new(RefCell::new(crate::value::Table {
                    data: results,
                    metatable: None,
                })));

                queue.borrow_mut().push_back((fiber_for_await, result_value));
                *pending.borrow_mut() -= 1;
                notify.notify_one();
                break;
            }

            notify.notified().await;
        }
    });

    Err(VMRuntimeError::Yield)
}
