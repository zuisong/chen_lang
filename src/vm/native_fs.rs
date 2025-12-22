use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{Table, Value, ValueError};
use crate::vm::{VM, VMRuntimeError};

pub fn create_fs_object() -> Value {
    let fs_obj = Value::object();

    let read_file_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let path_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.read_file".to_string(),
            })
        })?;

        match fs::read_to_string(path) {
            Ok(content) => Ok(Value::string(content)),
            Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.read_file: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })),
        }
    };

    let write_file_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let (path_arg, content_arg) = if args.len() > 2 {
            (&args[1], &args[2])
        } else {
            (&args[0], &args[1])
        };

        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.write_file".to_string(),
            })
        })?;

        let content = content_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: content_arg.get_type(),
                operation: "fs.write_file".to_string(),
            })
        })?;

        match fs::write(path, content) {
            Ok(_) => Ok(Value::null()),
            Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.write_file: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })),
        }
    };

    let exists_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let path_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.exists".to_string(),
            })
        })?;

        Ok(Value::bool(std::path::Path::new(path).exists()))
    };

    let remove_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let path_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.remove".to_string(),
            })
        })?;

        let metadata = fs::metadata(path).map_err(|e| {
            VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.remove: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })
        })?;

        let res = if metadata.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };

        match res {
            Ok(_) => Ok(Value::null()),
            Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.remove: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })),
        }
    };

    let read_dir_fn = |_vm: &mut VM, args: Vec<Value>| -> Result<Value, VMRuntimeError> {
        let path_arg = if args.len() > 1 { &args[1] } else { &args[0] };
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.read_dir".to_string(),
            })
        })?;

        let entries = fs::read_dir(path).map_err(|e| {
            VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.read_dir: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })
        })?;

        let mut table = Table {
            data: IndexMap::new(),
            metatable: None,
        };
        let mut count = 0;
        for entry in entries {
            if let Ok(entry) = entry {
                table.data.insert(
                    count.to_string(),
                    Value::string(entry.file_name().to_string_lossy().to_string()),
                );
                count += 1;
            }
        }

        if let Value::Object(proto) = &_vm.array_prototype {
            table.metatable = Some(proto.clone());
        }

        Ok(Value::Object(Rc::new(RefCell::new(table))))
    };

    if let Value::Object(obj) = &fs_obj {
        let mut obj = obj.borrow_mut();
        obj.data.insert(
            "read_file".to_string(),
            Value::NativeFunction(Rc::new(Box::new(read_file_fn))),
        );
        obj.data.insert(
            "write_file".to_string(),
            Value::NativeFunction(Rc::new(Box::new(write_file_fn))),
        );
        obj.data.insert(
            "exists".to_string(),
            Value::NativeFunction(Rc::new(Box::new(exists_fn))),
        );
        obj.data.insert(
            "remove".to_string(),
            Value::NativeFunction(Rc::new(Box::new(remove_fn))),
        );
        obj.data.insert(
            "read_dir".to_string(),
            Value::NativeFunction(Rc::new(Box::new(read_dir_fn))),
        );
    }

    fs_obj
}
