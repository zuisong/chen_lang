use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use indexmap::IndexMap;
use jiff::Timestamp;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::{compiler, parser};

pub mod error;
pub mod fiber;
pub mod interpreter;
pub mod program;

mod native_array_prototype;

mod native_date;
mod native_fs;
#[cfg(feature = "http")]
mod native_http;
mod native_io;
mod native_json;
mod native_object_prototype;
mod native_process;
mod native_string_prototype;
mod native_timer;

pub mod rt;
use rt::AsyncState;

#[cfg(test)]
mod vm_tests;

pub use error::{RuntimeErrorWithContext, VMResult, VMRuntimeError};
pub use fiber::{ExceptionHandler, Fiber, FiberState};
use native_array_prototype::create_array_prototype;

use native_date::create_date_object;
use native_fs::create_fs_object;
#[cfg(feature = "http")]
use native_http::create_http_object;
use native_io::create_io_object;
use native_json::create_json_object;
use native_object_prototype::create_object_prototype;
use native_process::create_process_object;
use native_string_prototype::create_string_prototype;
use native_symbol::create_symbol_object;
use native_timer::create_timer_object;

pub mod native_symbol;
pub use program::{Instruction, Program, Symbol};

pub(crate) use crate::value::{NativeContext, NativeFnType, ObjClosure, Value, ValueError, ValueType};

/// 虚拟机实现
pub struct VM {
    pub stack: Vec<Value>,                  // 操作数栈
    pub variables: IndexMap<String, Value>, // 全局变量存储
    pub pc: usize,                          // 程序计数器
    pub fp: usize,                          // 帧指针
    // (pc, fp, program, closure)
    pub call_stack: Vec<fiber::CallFrame>,     // 调用栈
    pub module_cache: IndexMap<String, Value>, // Module Cache
    pub stdout: Box<dyn Write>,                // 标准输出
    pub array_prototype: Value,                // 数组原型对象
    pub string_prototype: Value,               // 字符串原型对象
    pub object_prototype: Value,               // 对象原型对象
    pub exception_handlers: Vec<ExceptionHandler>,
    pub open_upvalues: Vec<Rc<RefCell<crate::value::UpvalueState>>>,

    pub current_fiber: Option<Rc<RefCell<Fiber>>>,
    pub program: Option<Rc<Program>>,
    pub current_closure: Option<Rc<ObjClosure>>,
    pub current_this: Option<Value>,

    // Async Runtime State
    pub async_state: AsyncState,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self::with_writer(Box::new(std::io::stdout()))
    }

    fn load_user_module(&mut self, path: &str) -> Result<Value, VMRuntimeError> {
        if let Some(cached_val) = self.module_cache.get(path) {
            return Ok(cached_val.clone());
        }

        let code = std::fs::read_to_string(path)
            .map_err(|e| VMRuntimeError::UncaughtException(format!("Failed to load {}: {}", path, e)))?;
        let ast = parser::parse_from_source(&code)
            .map_err(|e| VMRuntimeError::UncaughtException(format!("Parse error in {}: {}", path, e)))?;
        let module_program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);

        let saved_stack_size = self.stack.len();
        let saved_pc = self.pc;
        let saved_fp = self.fp;
        let res = self.execute_rc(Rc::new(module_program));

        self.pc = saved_pc;
        self.fp = saved_fp;

        match res {
            Ok(val) => {
                self.stack.truncate(saved_stack_size);
                self.module_cache.insert(path.to_string(), val.clone());
                Ok(val)
            }
            Err(e) => {
                self.stack.truncate(saved_stack_size);
                Err(e.error)
            }
        }
    }

    fn create_console_object() -> Value {
        let console = Value::object();
        if let Value::Object(console_obj) = &console {
            let mut console_obj = console_obj.borrow_mut();
            console_obj.data.insert(
                "print".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        for (i, val) in ctx.args.iter().enumerate() {
                            if i > 0 {
                                write!(vm.stdout, " ").unwrap();
                            }
                            write!(vm.stdout, "{}", val).unwrap();
                        }
                        vm.stdout.flush().unwrap();
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            console_obj.data.insert(
                "log".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        for (i, val) in ctx.args.iter().enumerate() {
                            if i > 0 {
                                write!(vm.stdout, " ").unwrap();
                            }
                            write!(vm.stdout, "{}", val).unwrap();
                        }
                        writeln!(vm.stdout).unwrap();
                        vm.stdout.flush().unwrap();
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            // Aliases for standard JS console methods
            let log_fn = console_obj.data.get("log").unwrap().clone();
            console_obj.data.insert("info".to_string(), log_fn.clone());
            console_obj.data.insert("warn".to_string(), log_fn.clone());
            console_obj.data.insert("error".to_string(), log_fn.clone());
            console_obj.data.insert("debug".to_string(), log_fn);

            console_obj.data.insert(
                "readLine".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, _ctx: crate::value::NativeContext| {
                        use std::io::BufRead;

                        let stdin = std::io::stdin();
                        let mut line = String::new();
                        stdin
                            .lock()
                            .read_line(&mut line)
                            .map_err(|e| VMRuntimeError::UncaughtException(e.to_string()))?;
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        Ok(Value::string(line))
                    }) as Box<NativeFnType>,
                )),
            );
        }
        console
    }

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        let object_prototype = create_object_prototype();
        variables.insert("Object".to_string(), object_prototype.clone());
        variables.insert("JSON".to_string(), create_json_object());
        variables.insert("Symbol".to_string(), create_symbol_object());
        variables.insert("console".to_string(), Self::create_console_object());

        let chen = Value::object();
        if let Value::Object(chen_obj) = &chen {
            let mut chen_obj = chen_obj.borrow_mut();
            chen_obj.data.insert("fs".to_string(), create_fs_object());
            #[cfg(feature = "http")]
            chen_obj.data.insert("http".to_string(), create_http_object());
            chen_obj.data.insert("process".to_string(), create_process_object());
            chen_obj.data.insert("timer".to_string(), create_timer_object());
            chen_obj.data.insert("date".to_string(), create_date_object());
            chen_obj.data.insert("io".to_string(), create_io_object());
            chen_obj.data.insert(
                "setMeta".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        if args.len() < 2 {
                            return Err(ValueError::InvalidOperation {
                                operator: "Chen.setMeta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }
                            .into());
                        }
                        let obj = &args[0];
                        let meta = args[1].clone();
                        obj.set_metatable(meta)?;
                        Ok(Value::null())
                    }) as Box<NativeFnType>,
                )),
            );
            chen_obj.data.insert(
                "getMeta".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        let Some(obj) = args.first() else {
                            return Err(ValueError::InvalidOperation {
                                operator: "Chen.getMeta".to_string(),
                                left_type: ValueType::Null,
                                right_type: ValueType::Null,
                            }
                            .into());
                        };
                        Ok(obj.get_metatable())
                    }) as Box<NativeFnType>,
                )),
            );
            chen_obj.data.insert(
                "load".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let args = ctx.args;
                        let Some(path_arg) = args.first() else {
                            return Err(ValueError::TypeMismatch {
                                expected: ValueType::String,
                                found: ValueType::Null,
                                operation: "Chen.load".to_string(),
                            }
                            .into());
                        };
                        let path = path_arg.as_string().ok_or_else(|| {
                            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                                expected: ValueType::String,
                                found: path_arg.get_type(),
                                operation: "Chen.load".to_string(),
                            })
                        })?;
                        vm.load_user_module(path)
                    }) as Box<NativeFnType>,
                )),
            );
        }
        variables.insert("Chen".to_string(), chen);

        let promise_obj = Value::object();
        if let Value::Object(obj) = &promise_obj {
            let mut obj = obj.borrow_mut();
            obj.data.insert(
                "resolve".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let val = ctx.args.first().cloned().unwrap_or(Value::null());
                        Ok(crate::promise::Promise::resolve_static(val))
                    }) as Box<NativeFnType>,
                )),
            );
            obj.data.insert(
                "reject".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|_vm: &mut VM, ctx: crate::value::NativeContext| {
                        let val = ctx.args.first().cloned().unwrap_or(Value::null());
                        Ok(crate::promise::Promise::reject_static(val))
                    }) as Box<NativeFnType>,
                )),
            );
            obj.data.insert(
                "new".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let executor = ctx.args.first().cloned().unwrap_or(Value::Null);
                        let pending_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                        
                        let p_resolve = pending_promise.clone();
                        let resolve_cb = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                            let val = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                            vm_inner.settle_promise(p_resolve.clone(), crate::promise::PromiseState::Fulfilled(val));
                            Ok(Value::Null)
                        }) as Box<NativeFnType>));

                        let p_reject = pending_promise.clone();
                        let reject_cb = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                            let val = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                            vm_inner.settle_promise(p_reject.clone(), crate::promise::PromiseState::Rejected(val));
                            Ok(Value::Null)
                        }) as Box<NativeFnType>));

                        match executor {
                            Value::Fn(closure) => {
                                let sym = &closure.func_symbol;
                                let mut fiber = Fiber::new();
                                fiber.program = Some(closure.program.clone());
                                fiber.current_closure = Some(closure.clone());
                                fiber.current_this = None;
                                fiber.fp = 0;
                                fiber.pc = sym.location as usize;
                                
                                fiber.stack.push(resolve_cb);
                                fiber.stack.push(reject_cb);
                                let nlocals = sym.nlocals;
                                let total_slots = std::cmp::max(2, sym.narguments) + nlocals;
                                fiber.stack.resize(total_slots, Value::null());
                                
                                fiber.state = FiberState::Running;
                                fiber.is_spawned = true;
                                fiber.skip_push_on_resume = true;
                                fiber.reject_on_error_promise = Some(pending_promise.clone());

                                let fiber_rc = Rc::new(RefCell::new(fiber));
                                vm.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                                *vm.async_state.pending_tasks.borrow_mut() += 1;
                                vm.async_state.notify.notify_one();
                            }
                            Value::NativeFunction(native_fn) => {
                                let native_ctx = NativeContext {
                                    this: None,
                                    args: vec![resolve_cb, reject_cb],
                                };
                                if let Err(e) = native_fn(vm, native_ctx) {
                                    vm.settle_promise(pending_promise.clone(), crate::promise::PromiseState::Rejected(Value::string(e.to_string())));
                                }
                            }
                            _ => {
                                vm.settle_promise(pending_promise.clone(), crate::promise::PromiseState::Rejected(Value::string("Promise executor must be a function".to_string())));
                            }
                        }
                        
                        Ok(Value::Promise(pending_promise))
                    }) as Box<NativeFnType>,
                )),
            );
            obj.data.insert(
                "all".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let iterable = ctx.args.first().cloned().unwrap_or(Value::Null);
                        let elements = {
                            let mut elems = Vec::new();
                            if let Value::Object(table_rc) = &iterable {
                                let table = table_rc.borrow();
                                let mut i = 0;
                                while let Some(elem) = table.data.get(&i.to_string()) {
                                    elems.push(elem.clone());
                                    i += 1;
                                }
                                if i == 0 {
                                    if let Some(Value::String(s)) = table.data.get("__type") {
                                        if s.as_ref() == "Array" {
                                            // Empty Array
                                        } else {
                                            return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                        }
                                    } else {
                                        return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                    }
                                }
                            } else {
                                return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an object/array".to_string())));
                            }
                            elems
                        };
                        
                        let n = elements.len();
                        let result_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                        
                        if n == 0 {
                            let empty_arr = vm.create_array(vec![]);
                            vm.settle_promise(result_promise.clone(), crate::promise::PromiseState::Fulfilled(empty_arr));
                            return Ok(Value::Promise(result_promise));
                        }
                        
                        struct AllState {
                            remaining: usize,
                            values: Vec<Value>,
                            result_promise: Rc<RefCell<crate::promise::Promise>>,
                        }
                        
                        let state = Rc::new(RefCell::new(AllState {
                            remaining: n,
                            values: vec![Value::Null; n],
                            result_promise: result_promise.clone(),
                        }));
                        
                        for (idx, elem) in elements.into_iter().enumerate() {
                            match elem {
                                Value::Promise(p_rc) => {
                                    let state_resolve = state.clone();
                                    let on_fulfilled = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let val = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        let mut s = state_resolve.borrow_mut();
                                        if s.remaining > 0 {
                                            s.values[idx] = val;
                                            s.remaining -= 1;
                                            if s.remaining == 0 {
                                                let arr = vm_inner.create_array(s.values.clone());
                                                vm_inner.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Fulfilled(arr));
                                            }
                                        }
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let state_reject = state.clone();
                                    let on_rejected = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let reason = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        let mut s = state_reject.borrow_mut();
                                        if s.remaining > 0 {
                                            s.remaining = 0;
                                            vm_inner.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Rejected(reason));
                                        }
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let next_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                                    let reaction = crate::promise::Reaction::Callback {
                                        on_fulfilled: Some(on_fulfilled),
                                        on_rejected: Some(on_rejected),
                                        next_promise,
                                    };
                                    
                                    let p_state = p_rc.borrow().state.clone();
                                    match p_state {
                                        crate::promise::PromiseState::Pending => {
                                            p_rc.borrow_mut().reactions.push(reaction);
                                        }
                                        _ => {
                                            vm.schedule_reaction(reaction, &p_state);
                                        }
                                    }
                                }
                                non_promise => {
                                    let mut s = state.borrow_mut();
                                    if s.remaining > 0 {
                                        s.values[idx] = non_promise;
                                        s.remaining -= 1;
                                        if s.remaining == 0 {
                                            let arr = vm.create_array(s.values.clone());
                                            vm.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Fulfilled(arr));
                                        }
                                    }
                                }
                            }
                        }
                        
                        Ok(Value::Promise(result_promise))
                    }) as Box<NativeFnType>,
                )),
            );
            obj.data.insert(
                "race".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let iterable = ctx.args.first().cloned().unwrap_or(Value::Null);
                        let elements = {
                            let mut elems = Vec::new();
                            if let Value::Object(table_rc) = &iterable {
                                let table = table_rc.borrow();
                                let mut i = 0;
                                while let Some(elem) = table.data.get(&i.to_string()) {
                                    elems.push(elem.clone());
                                    i += 1;
                                }
                                if i == 0 {
                                    if let Some(Value::String(s)) = table.data.get("__type") {
                                        if s.as_ref() == "Array" {
                                            // OK
                                        } else {
                                            return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                        }
                                    } else {
                                        return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                    }
                                }
                            } else {
                                return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an object/array".to_string())));
                            }
                            elems
                        };
                        
                        let result_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                        
                        for elem in elements {
                            match elem {
                                Value::Promise(p_rc) => {
                                    let p_res = result_promise.clone();
                                    let on_fulfilled = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let val = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        vm_inner.settle_promise(p_res.clone(), crate::promise::PromiseState::Fulfilled(val));
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let p_rej = result_promise.clone();
                                    let on_rejected = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let reason = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        vm_inner.settle_promise(p_rej.clone(), crate::promise::PromiseState::Rejected(reason));
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let next_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                                    let reaction = crate::promise::Reaction::Callback {
                                        on_fulfilled: Some(on_fulfilled),
                                        on_rejected: Some(on_rejected),
                                        next_promise,
                                    };
                                    
                                    let p_state = p_rc.borrow().state.clone();
                                    match p_state {
                                        crate::promise::PromiseState::Pending => {
                                            p_rc.borrow_mut().reactions.push(reaction);
                                        }
                                        _ => {
                                            vm.schedule_reaction(reaction, &p_state);
                                        }
                                    }
                                }
                                non_promise => {
                                    vm.settle_promise(result_promise.clone(), crate::promise::PromiseState::Fulfilled(non_promise));
                                }
                            }
                        }
                        
                        Ok(Value::Promise(result_promise))
                    }) as Box<NativeFnType>,
                )),
            );
            obj.data.insert(
                "allSettled".to_string(),
                Value::NativeFunction(Rc::new(
                    Box::new(|vm: &mut VM, ctx: crate::value::NativeContext| {
                        let iterable = ctx.args.first().cloned().unwrap_or(Value::Null);
                        let elements = {
                            let mut elems = Vec::new();
                            if let Value::Object(table_rc) = &iterable {
                                let table = table_rc.borrow();
                                let mut i = 0;
                                while let Some(elem) = table.data.get(&i.to_string()) {
                                    elems.push(elem.clone());
                                    i += 1;
                                }
                                if i == 0 {
                                    if let Some(Value::String(s)) = table.data.get("__type") {
                                        if s.as_ref() == "Array" {
                                            // OK
                                        } else {
                                            return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                        }
                                    } else {
                                        return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an array".to_string())));
                                    }
                                }
                            } else {
                                return Ok(crate::promise::Promise::reject_static(Value::string("Iterable must be an object/array".to_string())));
                            }
                            elems
                        };
                        
                        let n = elements.len();
                        let result_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                        
                        if n == 0 {
                            let empty_arr = vm.create_array(vec![]);
                            vm.settle_promise(result_promise.clone(), crate::promise::PromiseState::Fulfilled(empty_arr));
                            return Ok(Value::Promise(result_promise));
                        }
                        
                        struct AllSettledState {
                            remaining: usize,
                            results: Vec<Value>,
                            result_promise: Rc<RefCell<crate::promise::Promise>>,
                        }
                        
                        let state = Rc::new(RefCell::new(AllSettledState {
                            remaining: n,
                            results: vec![Value::Null; n],
                            result_promise: result_promise.clone(),
                        }));
                        
                        for (idx, elem) in elements.into_iter().enumerate() {
                            let make_res = |status: &str, k: &str, v: Value| -> Value {
                                let mut table = crate::value::Table {
                                    data: IndexMap::new(),
                                    metatable: None,
                                };
                                table.data.insert("status".to_string(), Value::string(status.to_string()));
                                table.data.insert(k.to_string(), v);
                                Value::Object(Rc::new(RefCell::new(table)))
                            };
                            
                            match elem {
                                Value::Promise(p_rc) => {
                                    let state_resolve = state.clone();
                                    let on_fulfilled = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let val = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        let mut s = state_resolve.borrow_mut();
                                        if s.remaining > 0 {
                                            s.results[idx] = make_res("fulfilled", "value", val);
                                            s.remaining -= 1;
                                            if s.remaining == 0 {
                                                let arr = vm_inner.create_array(s.results.clone());
                                                vm_inner.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Fulfilled(arr));
                                            }
                                        }
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let state_reject = state.clone();
                                    let on_rejected = Value::NativeFunction(Rc::new(Box::new(move |vm_inner: &mut VM, ctx_inner: crate::value::NativeContext| {
                                        let reason = ctx_inner.args.first().cloned().unwrap_or(Value::Null);
                                        let mut s = state_reject.borrow_mut();
                                        if s.remaining > 0 {
                                            s.results[idx] = make_res("rejected", "reason", reason);
                                            s.remaining -= 1;
                                            if s.remaining == 0 {
                                                let arr = vm_inner.create_array(s.results.clone());
                                                vm_inner.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Fulfilled(arr));
                                            }
                                        }
                                        Ok(Value::Null)
                                    }) as Box<NativeFnType>));
                                    
                                    let next_promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
                                    let reaction = crate::promise::Reaction::Callback {
                                        on_fulfilled: Some(on_fulfilled),
                                        on_rejected: Some(on_rejected),
                                        next_promise,
                                    };
                                    
                                    let p_state = p_rc.borrow().state.clone();
                                    match p_state {
                                        crate::promise::PromiseState::Pending => {
                                            p_rc.borrow_mut().reactions.push(reaction);
                                        }
                                        _ => {
                                            vm.schedule_reaction(reaction, &p_state);
                                        }
                                    }
                                }
                                non_promise => {
                                    let mut s = state.borrow_mut();
                                    if s.remaining > 0 {
                                        s.results[idx] = make_res("fulfilled", "value", non_promise);
                                        s.remaining -= 1;
                                        if s.remaining == 0 {
                                            let arr = vm.create_array(s.results.clone());
                                            vm.settle_promise(s.result_promise.clone(), crate::promise::PromiseState::Fulfilled(arr));
                                        }
                                    }
                                }
                            }
                        }
                        
                        Ok(Value::Promise(result_promise))
                    }) as Box<NativeFnType>,
                )),
            );
        }
        variables.insert("Promise".to_string(), promise_obj);

        VM {
            stack: Vec::with_capacity(1024),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: writer,
            array_prototype: create_array_prototype(),
            string_prototype: create_string_prototype(),
            object_prototype,
            exception_handlers: Vec::new(),
            open_upvalues: Vec::new(),
            current_fiber: None,
            program: None,
            current_closure: None,
            current_this: None,
            module_cache: IndexMap::new(),
            async_state: AsyncState::new(),
        }
    }

    /// 注册全局变量
    pub fn register_global_var(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// 注册字符串类型的全局变量
    pub fn add_var_str(&mut self, name: &str, value: &str) {
        self.register_global_var(name, Value::string(value.to_string()));
    }

    /// 注册布尔类型的全局变量
    pub fn add_var_bool(&mut self, name: &str, value: bool) {
        self.register_global_var(name, Value::bool(value));
    }

    /// 注册整数类型的全局变量
    pub fn add_var_int(&mut self, name: &str, value: i32) {
        self.register_global_var(name, Value::int(value));
    }

    /// 注册浮点类型的全局变量
    pub fn add_var_float(&mut self, name: &str, value: f64) {
        self.register_global_var(name, Value::float(Decimal::from_f64_retain(value).unwrap_or_default()));
    }

    /// 获取当前栈状态（用于调试）
    pub fn get_stack(&self) -> &[Value] {
        &self.stack
    }

    /// 获取变量状态（用于调试）
    pub fn get_variables(&self) -> &IndexMap<String, Value> {
        &self.variables
    }

    /// 快速抛出运行时异常（供 native function 使用）
    pub fn throw_str(&mut self, msg: impl Into<String>) -> Result<Value, VMRuntimeError> {
        Err(VMRuntimeError::UncaughtException(msg.into()))
    }

    pub fn settle_promise(&mut self, promise_rc: Rc<RefCell<crate::promise::Promise>>, state: crate::promise::PromiseState) {
        let reactions = promise_rc.borrow_mut().settle(state);
        for reaction in reactions {
            self.schedule_reaction(reaction, &promise_rc.borrow().state);
        }
    }

    pub fn schedule_reaction(&mut self, reaction: crate::promise::Reaction, settled_state: &crate::promise::PromiseState) {
        match reaction {
            crate::promise::Reaction::ResumeFiber(fiber) => {
                let res = match settled_state {
                    crate::promise::PromiseState::Fulfilled(val) => Ok(val.clone()),
                    crate::promise::PromiseState::Rejected(err) => {
                        let msg = match err {
                            Value::String(s) => s.to_string(),
                            other => format!("{:?}", other),
                        };
                        Err(VMRuntimeError::UncaughtException(msg))
                    }
                    _ => unreachable!(),
                };
                self.async_state.ready_queue.borrow_mut().push_back((fiber, res));
                self.async_state.notify.notify_one();
            }
            crate::promise::Reaction::Callback {
                on_fulfilled,
                on_rejected,
                next_promise,
            } => {
                match settled_state {
                    crate::promise::PromiseState::Fulfilled(val) => {
                        if let Some(callback) = on_fulfilled {
                            if matches!(callback, Value::Fn(_) | Value::NativeFunction(_)) {
                                if let Err(e) = self.spawn_callback_fiber(callback, val.clone(), next_promise.clone(), None) {
                                    self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(Value::string(e)));
                                }
                            } else {
                                self.settle_promise(next_promise, crate::promise::PromiseState::Fulfilled(val.clone()));
                            }
                        } else {
                            self.settle_promise(next_promise, crate::promise::PromiseState::Fulfilled(val.clone()));
                        }
                    }
                    crate::promise::PromiseState::Rejected(reason) => {
                        if let Some(callback) = on_rejected {
                            if matches!(callback, Value::Fn(_) | Value::NativeFunction(_)) {
                                if let Err(e) = self.spawn_callback_fiber(callback, reason.clone(), next_promise.clone(), None) {
                                    self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(Value::string(e)));
                                }
                            } else {
                                self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(reason.clone()));
                            }
                        } else {
                            self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(reason.clone()));
                        }
                    }
                    _ => unreachable!(),
                }
            }
            crate::promise::Reaction::Finally {
                on_finally,
                next_promise,
            } => {
                if let Err(e) = self.spawn_callback_fiber(on_finally, Value::null(), next_promise.clone(), Some(settled_state.clone())) {
                    self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(Value::string(e)));
                }
            }
        }
    }

    fn spawn_callback_fiber(
        &mut self,
        callback: Value,
        arg: Value,
        next_promise: Rc<RefCell<crate::promise::Promise>>,
        finally_initial_state: Option<crate::promise::PromiseState>,
    ) -> Result<(), String> {
        match callback {
            Value::Fn(closure) => {
                let sym = &closure.func_symbol;
                let mut fiber = Fiber::new();
                fiber.program = Some(closure.program.clone());
                fiber.current_closure = Some(closure.clone());
                fiber.current_this = None;
                fiber.fp = 0;
                fiber.pc = sym.location as usize;
                
                fiber.stack.push(arg);
                let nlocals = sym.nlocals;
                let total_slots = std::cmp::max(1, sym.narguments) + nlocals;
                fiber.stack.resize(total_slots, Value::null());
                
                fiber.state = FiberState::Running;
                fiber.is_spawned = true;
                fiber.skip_push_on_resume = true;
                fiber.associated_promise = Some(next_promise);
                fiber.finally_initial_state = finally_initial_state;

                let fiber_rc = Rc::new(RefCell::new(fiber));
                self.async_state.ready_queue.borrow_mut().push_back((fiber_rc, Ok(Value::null())));
                *self.async_state.pending_tasks.borrow_mut() += 1;
                self.async_state.notify.notify_one();
                Ok(())
            }
            Value::NativeFunction(native_fn) => {
                let native_ctx = NativeContext {
                    this: None,
                    args: vec![arg],
                };
                match native_fn(self, native_ctx) {
                    Ok(res) => {
                        let final_state = if let Some(initial_state) = finally_initial_state {
                            initial_state
                        } else {
                            crate::promise::PromiseState::Fulfilled(res)
                        };
                        self.settle_promise(next_promise, final_state);
                    }
                    Err(e) => {
                        self.settle_promise(next_promise, crate::promise::PromiseState::Rejected(Value::string(e.to_string())));
                    }
                }
                Ok(())
            }
            _ => Err("Callback is not callable".to_string()),
        }
    }
}

impl VM {
    pub fn create_array(&self, elements: Vec<Value>) -> Value {
        let mut table = crate::value::Table {
            data: IndexMap::new(),
            metatable: if let Value::Object(proto_rc) = &self.array_prototype {
                Some(proto_rc.clone())
            } else {
                None
            },
        };
        table.data.insert("__type".to_string(), Value::string("Array".to_string()));
        for (i, val) in elements.into_iter().enumerate() {
            table.data.insert(i.to_string(), val);
        }
        Value::Object(Rc::new(RefCell::new(table)))
    }
}

pub fn native_iter_self(_vm: &mut VM, ctx: NativeContext) -> Result<Value, VMRuntimeError> {
    Ok(ctx.this.clone().unwrap_or(Value::Null))
}

