use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::vm::native_coroutine::native_coroutine_yield;

/// 注册 Lua 风格的全局函数和标准库（type、tostring、pairs、table、string、math、os 等）。
pub fn register_global_libs(vm: &mut VM) {
    register_global(vm, "type", native_type);
    register_global(vm, "tostring", native_tostring);
    register_global(vm, "tonumber", native_tonumber);
    register_global(vm, "select", native_select);
    register_global(vm, "unpack", native_unpack);
    register_global(vm, "pcall", native_pcall);
    register_global(vm, "xpcall", native_xpcall);
    register_global(vm, "assert", native_assert);
    register_global(vm, "rawequal", native_rawequal);
    register_global(vm, "rawget", native_rawget);
    register_global(vm, "rawset", native_rawset);
    register_global(vm, "rawlen", native_rawlen);
    register_global(vm, "pairs", native_pairs);
    register_global(vm, "ipairs", native_ipairs);
    register_global(vm, "next", native_next);
    register_global(vm, "collectgarbage", |_vm, _args| Ok(Value::Int(0)));

    // 全局库
    vm.register_global_var("table", create_table_library());
    vm.register_global_var("string", crate::vm::native_string_lib::create_string_library());
    vm.register_global_var("math", create_math_library());
    vm.register_global_var("os", create_os_library());

    // 元表库（Lua 别名）
    vm.register_global_var(
        "setmetatable",
        vm.variables.get("set_meta").cloned().unwrap_or(Value::Null),
    );
    vm.register_global_var(
        "getmetatable",
        vm.variables.get("get_meta").cloned().unwrap_or(Value::Null),
    );

    // _G 与 _VERSION
    vm.register_global_var(
        "_VERSION",
        Value::string("Chen Lang 0.2.0 (Lua 5.2 compatible)".to_string()),
    );
    let g = Value::object();
    if let Value::Object(g_table) = &g {
        for (k, v) in vm.variables.clone().iter() {
            g_table.borrow_mut().data.insert(k.clone(), v.clone());
        }
    }
    vm.register_global_var("_G", g.clone());
    if let Value::Object(g_table) = &g {
        let self_ref = g.clone();
        g_table.borrow_mut().data.insert("_G".to_string(), self_ref);
    }
}

fn register_global(vm: &mut VM, name: &str, f: fn(&mut VM, Vec<Value>) -> Result<Value, VMRuntimeError>) {
    vm.register_global_var(name, Value::NativeFunction(Rc::new(Box::new(f))));
}

// --- 全局基础函数 ---

/// 将值转换为字符串，支持 `__tostring` 元方法。
pub(crate) fn value_to_string(vm: &mut VM, val: &Value) -> String {
    if let Value::Object(_) = val {
        if let Some(mm) = val.get_metamethod_from_object("__tostring") {
            let mut fiber = Fiber::new();
            fiber.stack.push(mm);
            fiber.stack.push(val.clone());
            let res = crate::vm::native_coroutine::resume_fiber(vm, Rc::new(RefCell::new(fiber)), vec![val.clone()]);
            if let Ok(s) = res {
                return s.to_string();
            }
        }
    }
    val.to_string()
}

fn native_type(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let v = args.first().cloned().unwrap_or(Value::Null);
    let t = match &v {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Object(_) => "table",
        Value::Fn(_) | Value::NativeFunction(_) => "function",
        Value::Coroutine(_) => "thread",
        Value::Null => "nil",
    };
    Ok(Value::string(t.to_string()))
}

fn native_tostring(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let v = args.first().cloned().unwrap_or(Value::Null);
    Ok(Value::string(value_to_string(vm, &v)))
}

fn native_tonumber(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let v = args.first().cloned().unwrap_or(Value::Null);
    match &v {
        Value::Int(_) | Value::Float(_) => Ok(v),
        Value::String(s) => {
            let s = s.trim();
            if let Ok(i) = s.parse::<i32>() {
                Ok(Value::Int(i))
            } else if let Ok(d) = s.parse::<rust_decimal::Decimal>() {
                Ok(Value::Float(d))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::float(
                    rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default(),
                ))
            } else {
                Ok(Value::Null)
            }
        }
        _ => Ok(Value::Null),
    }
}

/// `select(n, ...)`：n>0 返回第 n 个及之后的值；n<0 返回最后 |n| 个值；n=='#' 返回个数
fn native_select(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Ok(Value::Null);
    }
    match &args[0] {
        Value::String(s) if s.as_str() == "#" => Ok(Value::Int((args.len() - 1) as i32)),
        v => {
            if let Some(n) = v.to_int() {
                if n == 0 {
                    return Err(VMRuntimeError::UncaughtException("select: index starts at 1".into()));
                }
                let total = args.len() - 1;
                let rest: Vec<Value> = if n > 0 {
                    let start = (n - 1).min(total as i32) as usize;
                    args[(start + 1)..].to_vec()
                } else {
                    let count = (-n).min(total as i32) as usize;
                    args[(total + 1 - count)..].to_vec()
                };
                // 多返回值协议：把除了最后一个之外的所有值压栈，返回最后一个
                for val in &rest[..rest.len().saturating_sub(1)] {
                    vm.stack.push(val.clone());
                }
                Ok(rest.last().cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
    }
}

/// `unpack(t [, i [, j]])`：返回数组片段。该语言数组为 0-based。
fn native_unpack(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    if let Value::Object(table_rc) = &t {
        let table = table_rc.borrow();
        let n = table.data.len() as i32;
        let i = args.get(1).and_then(|v| v.to_int()).unwrap_or(0).max(0);
        let j = args.get(2).and_then(|v| v.to_int()).unwrap_or(n - 1).min(n - 1);
        let mut pushed = false;
        for idx in i..=j {
            if let Some(v) = table.data.get(&idx.to_string()) {
                vm.stack.push(v.clone());
                pushed = true;
            }
        }
        if !pushed {
            vm.stack.push(Value::Null);
        }
        return Ok(Value::Null);
    }
    vm.stack.push(Value::Null);
    Ok(Value::Null)
}

// --- pcall / xpcall / assert ---

fn native_pcall(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    pcall_impl(vm, args, false)
}

fn native_xpcall(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    pcall_impl(vm, args, true)
}

fn pcall_impl(vm: &mut VM, mut args: Vec<Value>, has_handler: bool) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Err(VMRuntimeError::UncaughtException("pcall: expected function".into()));
    }
    let f = args.remove(0);

    let handler = if has_handler && !args.is_empty() {
        Some(args.remove(0))
    } else {
        None
    };

    // 创建 fiber 运行 f
    let mut fiber = Fiber::new();
    if let Value::NativeFunction(nf) = &f {
        fiber.native_function = Some(nf.clone());
    }
    fiber.stack.push(f);
    let resume_args: Vec<Value> = args;
    for a in &resume_args {
        fiber.stack.push(a.clone());
    }
    let fiber_rc = Rc::new(RefCell::new(fiber));

    let result = crate::vm::native_coroutine::resume_fiber(vm, fiber_rc, resume_args);

    match result {
        Ok(val) => {
            vm.stack.push(Value::bool(true));
            Ok(val)
        }
        Err(e) => {
            let err_msg = Value::string(error_to_string(&e));
            let final_val = if let Some(h) = handler {
                // 调用错误处理函数
                let mut hfiber = Fiber::new();
                hfiber.stack.push(h.clone());
                hfiber.stack.push(err_msg.clone());
                let hres =
                    crate::vm::native_coroutine::resume_fiber(vm, Rc::new(RefCell::new(hfiber)), vec![err_msg.clone()]);
                hres.unwrap_or(err_msg)
            } else {
                err_msg
            };
            vm.stack.push(Value::bool(false));
            Ok(final_val)
        }
    }
}

fn error_to_string(e: &VMRuntimeError) -> String {
    match e {
        VMRuntimeError::UncaughtException(msg) => msg.clone(),
        _ => e.to_string(),
    }
}

/// `assert(v [, message])`：v 为真返回 v，否则抛出错误
fn native_assert(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let v = args.first().cloned().unwrap_or(Value::Null);
    if v.is_truthy() {
        Ok(v)
    } else {
        let msg = args
            .get(1)
            .map(|m| m.to_string())
            .unwrap_or_else(|| "assertion failed!".to_string());
        Err(VMRuntimeError::UncaughtException(msg))
    }
}

// --- raw* 函数 ---

fn native_rawequal(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let a = args.first().cloned().unwrap_or(Value::Null);
    let b = args.get(1).cloned().unwrap_or(Value::Null);
    Ok(Value::bool(a == b))
}

fn native_rawget(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    let k = args.get(1).cloned().unwrap_or(Value::Null);
    if let Value::Object(table_rc) = &t {
        let table = table_rc.borrow();
        return Ok(table.data.get(&k.to_string()).cloned().unwrap_or(Value::Null));
    }
    Ok(Value::Null)
}

fn native_rawset(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    let k = args.get(1).cloned().unwrap_or(Value::Null);
    let v = args.get(2).cloned().unwrap_or(Value::Null);
    if let Value::Object(table_rc) = &t {
        table_rc.borrow_mut().data.insert(k.to_string(), v);
        return Ok(t);
    }
    Ok(Value::Null)
}

fn native_rawlen(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    match &t {
        Value::String(s) => Ok(Value::Int(s.chars().count() as i32)),
        Value::Object(table_rc) => Ok(Value::Int(table_rc.borrow().data.len() as i32)),
        _ => Ok(Value::Int(0)),
    }
}

// --- pairs / ipairs / next ---

/// `pairs(t)`：返回产生 {key, value} 对的迭代协程
fn native_pairs(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    let table_rc = match &t {
        Value::Object(t) => t.clone(),
        _ => {
            return Err(VMRuntimeError::UncaughtException(
                "bad argument #1 to 'pairs' (table expected)".into(),
            ));
        }
    };

    let keys: Vec<String> = {
        let table = table_rc.borrow();
        table.data.keys().cloned().collect()
    };
    let index = Rc::new(RefCell::new(0));

    let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
        let mut idx = index.borrow_mut();
        if *idx < keys.len() {
            let key = keys[*idx].clone();
            let val = {
                let table = table_rc.borrow();
                table.data.get(&key).cloned().unwrap_or(Value::Null)
            };
            *idx += 1;
            let mut data = IndexMap::new();
            data.insert("key".to_string(), Value::string(key));
            data.insert("value".to_string(), val);
            let pair = Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None })));
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

/// `ipairs(t)`：返回产生 {key, value} 对（仅整数键 1..n）的迭代协程
fn native_ipairs(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    let table_rc = match &t {
        Value::Object(t) => t.clone(),
        _ => {
            return Err(VMRuntimeError::UncaughtException(
                "bad argument #1 to 'ipairs' (table expected)".into(),
            ));
        }
    };

    let len = {
        let table = table_rc.borrow();
        table.data.len()
    };
    // 该语言数组为 0-based（BuildArray 使用 "0".."n-1" 作为键），因此从 0 开始
    let index = Rc::new(RefCell::new(0));

    let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
        let mut idx = index.borrow_mut();
        if *idx >= len {
            return Ok(Value::Null);
        }
        let key = idx.to_string();
        let val = {
            let table = table_rc.borrow();
            table.data.get(&key).cloned()
        };
        match val {
            Some(val) => {
                let mut data = IndexMap::new();
                data.insert("key".to_string(), Value::Int(*idx as i32));
                data.insert("value".to_string(), val);
                let pair = Value::Object(Rc::new(RefCell::new(crate::value::Table { data, metatable: None })));
                *idx += 1;
                return native_coroutine_yield(vm, vec![pair]);
            }
            None => {
                let _ = len;
                Ok(Value::Null)
            }
        }
    };

    let mut fiber = Fiber::new();
    let nf_rc = Rc::new(Box::new(iter_body) as Box<NativeFnType>);
    fiber.native_function = Some(nf_rc.clone());
    fiber.stack.push(Value::NativeFunction(nf_rc));
    Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))))
}

/// `next(t [, k])`：返回 (key, value)；无更多元素返回 nil
fn native_next(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = args.first().cloned().unwrap_or(Value::Null);
    if let Value::Object(table_rc) = &t {
        let table = table_rc.borrow();
        let prev_key = args.get(1).map(|k| k.to_string());
        let keys: Vec<String> = table.data.keys().cloned().collect();
        if keys.is_empty() {
            vm.stack.push(Value::Null);
            return Ok(Value::Null);
        }
        let idx = match prev_key {
            Some(pk) if !pk.is_empty() => keys.iter().position(|k| *k == pk).map(|i| i + 1),
            _ => Some(0),
        };
        if let Some(idx) = idx {
            if idx < keys.len() {
                let key = &keys[idx];
                let val = table.data.get(key).cloned().unwrap_or(Value::Null);
                vm.stack.push(Value::string(key.clone()));
                return Ok(val);
            }
        }
        vm.stack.push(Value::Null);
        return Ok(Value::Null);
    }
    vm.stack.push(Value::Null);
    Ok(Value::Null)
}

// --- table 库 ---

pub fn create_table_library() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "insert".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_insert) as Box<NativeFnType>)),
    );
    table.data.insert(
        "remove".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_remove) as Box<NativeFnType>)),
    );
    table.data.insert(
        "concat".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_concat) as Box<NativeFnType>)),
    );
    table.data.insert(
        "sort".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_sort) as Box<NativeFnType>)),
    );
    table.data.insert(
        "unpack".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_unpack) as Box<NativeFnType>)),
    );
    table.data.insert(
        "pack".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_pack) as Box<NativeFnType>)),
    );
    table.data.insert(
        "getn".to_string(),
        Value::NativeFunction(Rc::new(Box::new(table_getn) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn table_as_ref<'a>(v: &'a Value, name: &str) -> Result<Rc<RefCell<crate::value::Table>>, VMRuntimeError> {
    match v {
        Value::Object(t) => Ok(t.clone()),
        _ => Err(VMRuntimeError::UncaughtException(format!(
            "bad argument #1 to '{}' (table expected)",
            name
        ))),
    }
}

/// `table.insert(t [, pos], value)`（0-based）
fn table_insert(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.len() < 2 {
        return Err(VMRuntimeError::UncaughtException(
            "table.insert: not enough arguments".into(),
        ));
    }
    let t = table_as_ref(&args[0], "insert")?;
    let (pos, value) = if args.len() >= 3 {
        (args[1].to_int().unwrap_or(-1), args[2].clone())
    } else {
        (-1, args[1].clone())
    };

    let mut table = t.borrow_mut();
    let n = table.data.len() as i32;
    if pos == -1 || pos >= n {
        table.data.insert(n.to_string(), value);
    } else {
        // 从 pos 开始右移
        for i in (pos..n).rev() {
            let v = table.data.get(&i.to_string()).cloned().unwrap_or(Value::Null);
            table.data.insert((i + 1).to_string(), v);
        }
        table.data.insert(pos.to_string(), value);
    }
    Ok(Value::Null)
}

/// `table.remove(t [, pos])`（0-based）
fn table_remove(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    if args.is_empty() {
        return Err(VMRuntimeError::UncaughtException(
            "table.remove: not enough arguments".into(),
        ));
    }
    let t = table_as_ref(&args[0], "remove")?;
    let mut table = t.borrow_mut();
    let n = table.data.len() as i32;
    let pos = args.get(1).and_then(|v| v.to_int()).unwrap_or(n - 1);

    if pos < 0 || pos >= n {
        return Ok(Value::Null);
    }
    let removed = table.data.shift_remove(&pos.to_string()).unwrap_or(Value::Null);
    // 左移
    for i in (pos + 1)..n {
        let v = table.data.shift_remove(&i.to_string()).unwrap_or(Value::Null);
        table.data.insert((i - 1).to_string(), v);
    }
    Ok(removed)
}

/// `table.concat(t [, sep [, i [, j]]])`（0-based）
fn table_concat(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = table_as_ref(&args[0], "concat")?;
    let sep = args.get(1).and_then(|v| v.as_string()).unwrap_or("").to_string();
    let table = t.borrow();
    let n = table.data.len() as i32;
    let i = args.get(2).and_then(|v| v.to_int()).unwrap_or(0);
    let j = args.get(3).and_then(|v| v.to_int()).unwrap_or(n - 1);

    let mut parts = Vec::new();
    for idx in i..=j {
        if let Some(v) = table.data.get(&idx.to_string()) {
            parts.push(v.to_string());
        }
    }
    Ok(Value::string(parts.join(&sep)))
}

/// `table.sort(t [, comp])`：默认升序排序（数字/字符串）
fn table_sort(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = table_as_ref(&args[0], "sort")?;
    let mut table = t.borrow_mut();
    let n = table.data.len() as i32;
    let mut values: Vec<Value> = Vec::new();
    for i in 0..n {
        values.push(table.data.get(&i.to_string()).cloned().unwrap_or(Value::Null));
    }
    values.sort_by(|a, b| compare_values(a, b));
    for (i, v) in values.into_iter().enumerate() {
        table.data.insert(i.to_string(), v);
    }
    Ok(Value::Null)
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    if let (Some(x), Some(y)) = (a.to_float(), b.to_float()) {
        return x.cmp(&y);
    }
    if let (Value::String(x), Value::String(y)) = (a, b) {
        return x.cmp(y);
    }
    std::cmp::Ordering::Equal
}

/// `table.unpack(t [, i [, j]])`
fn table_unpack(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    native_unpack(vm, args)
}

/// `table.pack(...)`：返回带 n 字段的数组（0-based，与语言一致）
fn table_pack(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut data = IndexMap::new();
    for (i, v) in args.iter().enumerate() {
        data.insert((i).to_string(), v.clone());
    }
    data.insert("n".to_string(), Value::Int(args.len() as i32));
    Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table {
        data,
        metatable: None,
    }))))
}

fn table_getn(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let t = table_as_ref(&args[0], "getn")?;
    Ok(Value::Int(t.borrow().data.len() as i32))
}

// --- math 库 ---

pub fn create_math_library() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "pi".to_string(),
        Value::float(rust_decimal::Decimal::from_str_exact("3.141592653589793").unwrap()),
    );
    table
        .data
        .insert("huge".to_string(), Value::float(rust_decimal::Decimal::MAX));
    let _ = table.data.insert("maxinteger".to_string(), Value::Int(i32::MAX));
    table.data.insert("mininteger".to_string(), Value::Int(i32::MIN));

    let funcs: &[(&str, fn(&mut VM, Vec<Value>) -> Result<Value, VMRuntimeError>)] = &[
        ("abs", math_abs),
        ("floor", math_floor),
        ("ceil", math_ceil),
        ("max", math_max),
        ("min", math_min),
        ("sqrt", math_sqrt),
        ("pow", math_pow),
        ("exp", math_exp),
        ("log", math_log),
        ("sin", math_sin),
        ("cos", math_cos),
        ("tan", math_tan),
        ("asin", math_asin),
        ("acos", math_acos),
        ("atan", math_atan),
        ("atan2", math_atan2),
        ("deg", math_deg),
        ("rad", math_rad),
        ("random", math_random),
        ("randomseed", math_randomseed),
        ("round", math_round),
        ("sign", math_sign),
        ("fmod", math_fmod),
        ("fabs", math_fabs),
        ("modf", math_modf),
        ("clamp", math_clamp),
    ];
    for (name, f) in funcs {
        table
            .data
            .insert(name.to_string(), Value::NativeFunction(Rc::new(Box::new(*f))));
    }

    Value::Object(Rc::new(RefCell::new(table)))
}

fn math_num_arg(args: &[Value], idx: usize) -> Result<rust_decimal::Decimal, VMRuntimeError> {
    match args.get(idx) {
        Some(v) => v
            .to_float()
            .ok_or_else(|| VMRuntimeError::UncaughtException("bad argument (number expected)".into())),
        None => Err(VMRuntimeError::UncaughtException("math: missing argument".into())),
    }
}

fn to_f64(d: rust_decimal::Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

fn from_f64(f: f64) -> Value {
    Value::float(rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default())
}

fn math_abs(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    Ok(Value::float(d.abs()))
}

fn math_floor(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    Ok(Value::Int(d.floor().to_i32().unwrap_or(0)))
}

fn math_ceil(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    Ok(Value::Int(d.ceil().to_i32().unwrap_or(0)))
}

fn math_max(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut best: Option<rust_decimal::Decimal> = None;
    let mut best_val = Value::Null;
    for v in &args {
        if let Some(d) = v.to_float() {
            if best.map_or(true, |b| d > b) {
                best = Some(d);
                best_val = v.clone();
            }
        }
    }
    Ok(best_val)
}

fn math_min(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut best: Option<rust_decimal::Decimal> = None;
    let mut best_val = Value::Null;
    for v in &args {
        if let Some(d) = v.to_float() {
            if best.map_or(true, |b| d < b) {
                best = Some(d);
                best_val = v.clone();
            }
        }
    }
    Ok(best_val)
}

fn math_sqrt(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    Ok(from_f64(to_f64(d).sqrt()))
}

fn math_pow(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let a = to_f64(math_num_arg(&args, 0)?);
    let b = to_f64(math_num_arg(&args, 1)?);
    Ok(from_f64(a.powf(b)))
}

fn math_exp(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = to_f64(math_num_arg(&args, 0)?);
    Ok(from_f64(d.exp()))
}

fn math_log(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = to_f64(math_num_arg(&args, 0)?);
    if args.len() > 1 {
        let base = to_f64(math_num_arg(&args, 1)?);
        Ok(from_f64(d.ln() / base.ln()))
    } else {
        Ok(from_f64(d.ln()))
    }
}

fn math_sin(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).sin()))
}

fn math_cos(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).cos()))
}

fn math_tan(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).tan()))
}

fn math_asin(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).asin()))
}

fn math_acos(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).acos()))
}

fn math_atan(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).atan()))
}

fn math_atan2(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let y = to_f64(math_num_arg(&args, 0)?);
    let x = to_f64(math_num_arg(&args, 1)?);
    Ok(from_f64(y.atan2(x)))
}

fn math_deg(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).to_degrees()))
}

fn math_rad(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    Ok(from_f64(to_f64(math_num_arg(&args, 0)?).to_radians()))
}

fn math_random(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);
    // 简单 xorshift 伪随机
    let mut x = SEED.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, Ordering::Relaxed);

    let rand_u32 = (x >> 32) as u32;
    match args.len() {
        0 => Ok(Value::float(
            rust_decimal::Decimal::from_f64_retain(rand_u32 as f64 / u32::MAX as f64).unwrap(),
        )),
        1 => {
            let m = args[0].to_int().unwrap_or(1);
            Ok(Value::Int((rand_u32 % m.max(1) as u32) as i32 + 1))
        }
        _ => {
            let lo = args[0].to_int().unwrap_or(1);
            let hi = args[1].to_int().unwrap_or(lo);
            let span = (hi - lo + 1).max(1) as u32;
            Ok(Value::Int(lo + (rand_u32 % span) as i32))
        }
    }
}

fn math_randomseed(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    static SEED: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);
    if let Some(n) = args.first().and_then(|v| v.to_int()) {
        SEED.store(n as u64, Ordering::Relaxed);
    }
    Ok(Value::Null)
}

fn math_round(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    let f = to_f64(d);
    Ok(Value::Int(f.round() as i32))
}

fn math_sign(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    Ok(Value::Int(if d.is_sign_negative() { -1 } else { 1 }))
}

fn math_fmod(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let a = math_num_arg(&args, 0)?;
    let b = math_num_arg(&args, 1)?;
    Ok(Value::float(a % b))
}

fn math_fabs(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    math_abs(_vm, args)
}

fn math_modf(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let d = math_num_arg(&args, 0)?;
    let f = to_f64(d);
    let int_part = f.trunc() as i32;
    let frac = f - f.trunc();
    let _ = int_part;
    Ok(from_f64(frac))
}

fn math_clamp(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let x = math_num_arg(&args, 0)?;
    let lo = math_num_arg(&args, 1)?;
    let hi = math_num_arg(&args, 2)?;
    let result = if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    };
    Ok(Value::float(result))
}

// --- os 库 ---

pub fn create_os_library() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table.data.insert(
        "time".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_time) as Box<NativeFnType>)),
    );
    table.data.insert(
        "clock".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_clock) as Box<NativeFnType>)),
    );
    table.data.insert(
        "date".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_date) as Box<NativeFnType>)),
    );
    table.data.insert(
        "getenv".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_getenv) as Box<NativeFnType>)),
    );
    table.data.insert(
        "tmpname".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_tmpname) as Box<NativeFnType>)),
    );
    table.data.insert(
        "exit".to_string(),
        Value::NativeFunction(Rc::new(Box::new(os_exit) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn os_time(_vm: &mut VM, _args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(Value::Int(now as i32))
}

fn os_clock(_vm: &mut VM, _args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let secs = std::time::Instant::now().elapsed().as_secs_f64();
    Ok(Value::float(
        rust_decimal::Decimal::from_f64_retain(secs).unwrap_or_default(),
    ))
}

fn os_date(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    use jiff::Timestamp;
    let fmt = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "%c".to_string());
    let now = Timestamp::now();
    let zoned = now.to_zoned(jiff::tz::TimeZone::system());
    if fmt.starts_with('!') {
        let utc = now.to_zoned(jiff::tz::TimeZone::UTC);
        return Ok(Value::string(utc.strftime(&fmt[1..]).to_string()));
    }
    Ok(Value::string(zoned.strftime(&fmt).to_string()))
}

fn os_getenv(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let name = args.first().and_then(|v| v.as_string()).unwrap_or("");
    match std::env::var(name) {
        Ok(v) => Ok(Value::string(v)),
        Err(_) => Ok(Value::Null),
    }
}

fn os_tmpname(_vm: &mut VM, _args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(Value::string(format!(
        "/tmp/chen_lang_{}_{}.tmp",
        std::process::id(),
        n
    )))
}

fn os_exit(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let code = args.first().and_then(|v| v.to_int()).unwrap_or(0);
    std::process::exit(code);
}
