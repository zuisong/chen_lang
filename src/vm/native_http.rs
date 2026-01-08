#[allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::value::{Table, Value, ValueError, ValueType};
use crate::vm::{Fiber, FiberState, VM, VMRuntimeError};

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

    let request_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        if args.len() < 2 {
            return Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "http.request".to_string(),
                left_type: ValueType::String,
                right_type: ValueType::Null,
            }));
        }

        let method_arg = &args[0];
        let url_arg = &args[1];
        let body_arg = args.get(2).cloned();
        let headers_arg = args.get(3).cloned();

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

        // Unified Async Logic
        {
            // Capture Current Fiber
            let current_fiber_rc = if let Some(c) = &_vm.current_fiber {
                c.clone()
            } else {
                let f = Rc::new(RefCell::new(Fiber::new()));
                _vm.current_fiber = Some(f.clone());
                f
            };

            let mut current_fiber = current_fiber_rc.borrow_mut();

            // Increment PC to point to next instruction upon resume
            _vm.pc += 1;

            _vm.save_state_to_fiber(&mut current_fiber);
            current_fiber.state = FiberState::Suspended;
            drop(current_fiber);

            let fiber_for_future = current_fiber_rc.clone();

            // Spawn Async Task
            _vm.async_state.spawn_future(
                async move {
                    let client = reqwest::Client::new();

                    let method = match reqwest::Method::from_str(&method_str) {
                        Ok(m) => m,
                        Err(e) => return Err(VMRuntimeError::UncaughtException(format!("HTTP invalid method: {}", e))),
                    };

                    let mut builder = client.request(method, &url_str);

                    if let Some(b) = body_str {
                        builder = builder.body(b);
                    }

                    if let Some(h) = headers_arg {
                        match value_to_header_map(&h) {
                            Ok(headers) => builder = builder.headers(headers),
                            Err(e) => {
                                return Err(VMRuntimeError::UncaughtException(format!("HTTP header error: {}", e)));
                            }
                        }
                    }

                    let resp_res = builder.send().await;

                    match resp_res {
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
                    }
                },
                fiber_for_future,
            );

            Err(VMRuntimeError::Yield)
        }
    };

    if let Value::Object(obj) = &http_obj {
        let mut obj = obj.borrow_mut();
        obj.data.insert(
            "request".to_string(),
            Value::NativeFunction(Rc::new(Box::new(request_fn))),
        );
    }

    http_obj
}
