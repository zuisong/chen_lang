#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
use std::rc::Rc;

use crate::value::{Value, ValueError};
use crate::vm::{VM, VMRuntimeError};

pub fn create_process_object() -> Value {
    let process_obj = Value::object();

    let exec_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let cmd_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let _cmd_str = cmd_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: cmd_arg.get_type(),
                operation: "process.exec".to_string(),
            })
        })?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", _cmd_str]).output()
            } else {
                Command::new("sh").args(["-c", _cmd_str]).output()
            }
            .map_err(|e| {
                VMRuntimeError::ValueError(ValueError::InvalidOperation {
                    operator: format!("process.exec: {}", e),
                    left_type: crate::value::ValueType::String,
                    right_type: crate::value::ValueType::Null,
                })
            })?;

            let result_obj = Value::object();
            if let Value::Object(obj) = &result_obj {
                let mut obj = obj.borrow_mut();
                obj.data
                    .insert("code".to_string(), Value::int(output.status.code().unwrap_or(0)));
                obj.data.insert(
                    "stdout".to_string(),
                    Value::string(String::from_utf8_lossy(&output.stdout).to_string()),
                );
                obj.data.insert(
                    "stderr".to_string(),
                    Value::string(String::from_utf8_lossy(&output.stderr).to_string()),
                );
            }
            Ok(result_obj)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: "process.exec: process execution not supported in WASM".to_string(),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            }))
        }
    };

    if let Value::Object(obj) = &process_obj {
        let mut obj = obj.borrow_mut();
        obj.data
            .insert("exec".to_string(), Value::NativeFunction(Rc::new(Box::new(exec_fn))));
    }

    process_obj
}
