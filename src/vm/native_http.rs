use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::value::{NativeContext, NativeFnType, Table, Value, ValueError, ValueType};
use crate::vm::{VM, VMRuntimeError};

fn value_to_header_map(value: &Value) -> Result<reqwest::header::HeaderMap, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Value::Object(obj) = value {
        let obj = obj.borrow();
        for (k, v) in &obj.data {
            if let Some(s) = v.as_string() {
                let header_name =
                    reqwest::header::HeaderName::from_str(k).map_err(|e| format!("Invalid header name: {}", e))?;
                let header_value =
                    reqwest::header::HeaderValue::from_str(s).map_err(|e| format!("Invalid header value: {}", e))?;
                headers.insert(header_name, header_value);
            }
        }
    }
    Ok(headers)
}

pub fn create_http_object() -> Value {
    let http_obj = Value::object();

    let request_fn = |vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.len() < 2 {
            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "http.request".to_string(),
                left_type: ValueType::String,
                right_type: ValueType::Null,
            }));
        }

        let method_arg = &ctx.args[0];
        let url_arg = &ctx.args[1];
        let body_arg = ctx.args.get(2).cloned();
        let headers_arg = ctx.args.get(3).cloned();

        let method_str = method_arg
            .as_string()
            .ok_or_else(|| {
                VMRuntimeError::ValueError(ValueError::TypeMismatch {
                    expected: ValueType::String,
                    found: method_arg.get_type(),
                    operation: "http.request (method)".to_string(),
                })
            })?
            .to_string();

        let url_str = url_arg
            .as_string()
            .ok_or_else(|| {
                VMRuntimeError::ValueError(ValueError::TypeMismatch {
                    expected: ValueType::String,
                    found: url_arg.get_type(),
                    operation: "http.request (url)".to_string(),
                })
            })?
            .to_string();

        let body_str = if let Some(val) = &body_arg {
            if matches!(val.get_type(), ValueType::Null) {
                None
            } else {
                Some(
                    val.as_string()
                        .ok_or_else(|| {
                            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                                expected: ValueType::String,
                                found: val.get_type(),
                                operation: "http.request (body)".to_string(),
                            })
                        })?
                        .to_string(),
                )
            }
        } else {
            None
        };

        let promise = Rc::new(RefCell::new(crate::promise::Promise::new()));
        let promise_val = Value::Promise(promise.clone());

        let ready_queue = vm.async_state.ready_queue.clone();
        let pending_tasks = vm.async_state.pending_tasks.clone();
        let notify = vm.async_state.notify.clone();

        *pending_tasks.borrow_mut() += 1;

        // Spawn Async Task
        tokio::task::spawn_local(async move {
            let client = reqwest::Client::new();

            let method = match reqwest::Method::from_str(&method_str) {
                Ok(m) => m,
                Err(e) => {
                    let mut p = promise.borrow_mut();
                    let reactions = p.reject(Value::string(format!("HTTP invalid method: {}", e)));
                    let mut q = ready_queue.borrow_mut();
                    for reaction in reactions {
                        if let crate::promise::Reaction::ResumeFiber(f) = reaction {
                            q.push_back((f, Err(VMRuntimeError::UncaughtException(format!("HTTP invalid method: {}", e)))));
                        }
                    }
                    *pending_tasks.borrow_mut() -= 1;
                    notify.notify_one();
                    return;
                }
            };

            let mut builder = client.request(method, &url_str);

            if let Some(b) = body_str {
                builder = builder.body(b);
            }

            if let Some(h) = headers_arg {
                match value_to_header_map(&h) {
                    Ok(headers) => builder = builder.headers(headers),
                    Err(e) => {
                        let mut p = promise.borrow_mut();
                        let reactions = p.reject(Value::string(format!("HTTP header error: {}", e)));
                        let mut q = ready_queue.borrow_mut();
                        for reaction in reactions {
                            if let crate::promise::Reaction::ResumeFiber(f) = reaction {
                                q.push_back((f, Err(VMRuntimeError::UncaughtException(format!("HTTP header error: {}", e)))));
                            }
                        }
                        *pending_tasks.borrow_mut() -= 1;
                        notify.notify_one();
                        return;
                    }
                }
            }

            let resp_res = builder.send().await;

            let result = match resp_res {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i32;
                    let headers = resp.headers().clone();
                    let text = resp.text().await.unwrap_or_default();

                    // Construct Response Object (Table)
                    let mut response_data = IndexMap::new();
                    response_data.insert("status".to_string(), Value::int(status));
                    response_data.insert("body".to_string(), Value::string(text));

                    let mut headers_data = IndexMap::new();
                    for (k, v) in headers.iter() {
                        if let Ok(val_str) = v.to_str() {
                            headers_data.insert(k.to_string(), Value::string(val_str.to_string()));
                        }
                    }

                    response_data.insert(
                        "headers".to_string(),
                        Value::Object(Rc::new(RefCell::new(Table {
                            data: headers_data,
                            metatable: None,
                        }))),
                    );

                    Ok(Value::Object(Rc::new(RefCell::new(Table {
                        data: response_data,
                        metatable: None,
                    }))))
                }
                Err(e) => Err(VMRuntimeError::UncaughtException(format!("HTTP request error: {}", e))),
            };

            let mut p = promise.borrow_mut();
            let mut q = ready_queue.borrow_mut();
            match result {
                Ok(val) => {
                    let reactions = p.resolve(val.clone());
                    for reaction in reactions {
                        if let crate::promise::Reaction::ResumeFiber(f) = reaction {
                            q.push_back((f, Ok(val.clone())));
                        }
                    }
                }
                Err(err) => {
                    let reactions = p.reject(Value::string(err.to_string()));
                    for reaction in reactions {
                        if let crate::promise::Reaction::ResumeFiber(f) = reaction {
                            q.push_back((f, Err(err.clone())));
                        }
                    }
                }
            }
            *pending_tasks.borrow_mut() -= 1;
            notify.notify_one();
        });

        Ok(promise_val)
    };

    if let Value::Object(obj) = &http_obj {
        let mut obj = obj.borrow_mut();
        let request = Value::NativeFunction(Rc::new(Box::new(request_fn) as Box<NativeFnType>));
        obj.data.insert("request".to_string(), request.clone());
        obj.data.insert("fetch".to_string(), request);
    }

    http_obj
}
