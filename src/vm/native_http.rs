use std::rc::Rc;

use crate::value::{Value, ValueError};
use crate::vm::{VM, VMRuntimeError};

pub fn create_http_object() -> Value {
    let http_obj = Value::object();

    let get_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let url_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let _url = url_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: url_arg.get_type(),
                operation: "http.get".to_string(),
            })
        })?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match reqwest::blocking::get(_url) {
                Ok(resp) => {
                    let text = resp.text().map_err(|e| {
                        VMRuntimeError::ValueError(ValueError::InvalidOperation {
                            operator: format!("http.get: {}", e),
                            left_type: crate::value::ValueType::String,
                            right_type: crate::value::ValueType::Null,
                        })
                    })?;
                    Ok(Value::string(text))
                }
                Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                    operator: format!("http.get: {}", e),
                    left_type: crate::value::ValueType::String,
                    right_type: crate::value::ValueType::Null,
                })),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "http.get: synchronous HTTP not supported in WASM".to_string(),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            }))
        }
    };

    let post_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let (url_arg, body_arg) = if args.len() > 2 {
            (&args[1], &args[2])
        } else {
            (&args[0], &args[1])
        };

        let _url = url_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: url_arg.get_type(),
                operation: "http.post".to_string(),
            })
        })?;

        let _body = body_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: body_arg.get_type(),
                operation: "http.post".to_string(),
            })
        })?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::blocking::Client::new();
            match client.post(_url).body(_body.to_string()).send() {
                Ok(resp) => {
                    let text = resp.text().map_err(|e| {
                        VMRuntimeError::ValueError(ValueError::InvalidOperation {
                            operator: format!("http.post: {}", e),
                            left_type: crate::value::ValueType::String,
                            right_type: crate::value::ValueType::Null,
                        })
                    })?;
                    Ok(Value::string(text))
                }
                Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                    operator: format!("http.post: {}", e),
                    left_type: crate::value::ValueType::String,
                    right_type: crate::value::ValueType::Null,
                })),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "http.post: synchronous HTTP not supported in WASM".to_string(),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            }))
        }
    };

    if let Value::Object(obj) = &http_obj {
        let mut obj = obj.borrow_mut();
        obj.data
            .insert("get".to_string(), Value::NativeFunction(Rc::new(Box::new(get_fn))));
        obj.data
            .insert("post".to_string(), Value::NativeFunction(Rc::new(Box::new(post_fn))));
    }

    http_obj
}
