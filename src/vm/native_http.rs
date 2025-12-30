#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::str::FromStr;

#[cfg(not(target_arch = "wasm32"))]
use indexmap::IndexMap;

use crate::value::{Table, Value, ValueError, ValueType};
use crate::vm::{VM, VMRuntimeError};

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
struct HttpResponse {
    status: i32,
    body: String,
    headers: reqwest::header::HeaderMap,
}

#[cfg(not(target_arch = "wasm32"))]
fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    headers: Option<&Value>,
) -> Result<HttpResponse, String> {
    let client = reqwest::blocking::Client::new();
    let method = reqwest::Method::from_str(method).map_err(|e| e.to_string())?;

    let mut builder = client.request(method, url);
    if let Some(b) = body {
        builder = builder.body(b.to_string());
    }

    if let Some(h) = headers {
        let header_map = value_to_header_map(h)?;
        builder = builder.headers(header_map);
    }

    match builder.send() {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let headers = resp.headers().clone();
            let text = resp.text().map_err(|e| e.to_string())?;
            Ok(HttpResponse {
                status,
                body: text,
                headers,
            })
        }
        Err(e) => Err(e.to_string()),
    }
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
        let body_arg = args.get(2);
        let headers_arg = args.get(3);

        let method = method_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: ValueType::String,
                found: method_arg.get_type(),
                operation: "http.request (method)".to_string(),
            })
        })?;

        let url = url_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: ValueType::String,
                found: url_arg.get_type(),
                operation: "http.request (url)".to_string(),
            })
        })?;

        let body = if let Some(val) = body_arg {
            if matches!(val.get_type(), ValueType::Null) {
                None
            } else {
                Some(val.as_string().ok_or_else(|| {
                    VMRuntimeError::ValueError(ValueError::TypeMismatch {
                        expected: ValueType::String,
                        found: val.get_type(),
                        operation: "http.request (body)".to_string(),
                    })
                })?)
            }
        } else {
            None
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let response = perform_request(method, url, body, headers_arg).map_err(|e| {
                VMRuntimeError::ValueError(ValueError::InvalidOperation {
                    operator: format!("http.request: {}", e),
                    left_type: ValueType::String,
                    right_type: ValueType::Null,
                })
            })?;

            // Construct Response Object
            let mut response_obj = Table {
                data: IndexMap::new(),
                metatable: None,
            };

            response_obj
                .data
                .insert("status".to_string(), Value::int(response.status));
            response_obj
                .data
                .insert("body".to_string(), Value::string(response.body));

            let mut headers_obj = Table {
                data: IndexMap::new(),
                metatable: None,
            };
            for (k, v) in response.headers.iter() {
                if let Ok(val_str) = v.to_str() {
                    headers_obj
                        .data
                        .insert(k.to_string(), Value::string(val_str.to_string()));
                }
            }
            response_obj
                .data
                .insert("headers".to_string(), Value::Object(Rc::new(RefCell::new(headers_obj))));

            Ok(Value::Object(Rc::new(RefCell::new(response_obj))))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "http.request: synchronous HTTP not supported in WASM".to_string(),
                left_type: ValueType::String,
                right_type: ValueType::Null,
            }))
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
