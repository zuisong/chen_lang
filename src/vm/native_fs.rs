use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{NativeContext, NativeFnType, Value, ValueError};
use crate::vm::{VM, VMRuntimeError};

pub fn create_fs_object() -> Value {
    let fs_obj = Value::object();

    let read_file_fn = |_vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.is_empty() {
            return Err(VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: crate::value::ValueType::Null,
                operation: "fs.readTextFile".to_string(),
            }));
        }
        let path_arg = &ctx.args[0];
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.readTextFile".to_string(),
            })
        })?;

        match fs::read_to_string(path) {
            Ok(content) => Ok(Value::string(content)),
            Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.readTextFile: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })),
        }
    };

    let write_file_fn = |_vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.len() < 2 {
            return Err(VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: crate::value::ValueType::Null,
                operation: "fs.writeTextFile".to_string(),
            }));
        }
        let path_arg = &ctx.args[0];
        let content_arg = &ctx.args[1];

        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.writeTextFile".to_string(),
            })
        })?;

        let content = content_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: content_arg.get_type(),
                operation: "fs.writeTextFile".to_string(),
            })
        })?;

        match fs::write(path, content) {
            Ok(_) => Ok(Value::null()),
            Err(e) => Err(VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.writeTextFile: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })),
        }
    };

    let exists_fn = |_vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.is_empty() {
            return Err(VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: crate::value::ValueType::Null,
                operation: "fs.exists".to_string(),
            }));
        }
        let path_arg = &ctx.args[0];
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.exists".to_string(),
            })
        })?;

        Ok(Value::bool(std::path::Path::new(path).exists()))
    };

    let remove_fn = |_vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.is_empty() {
            return Err(VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: crate::value::ValueType::Null,
                operation: "fs.remove".to_string(),
            }));
        }
        let path_arg = &ctx.args[0];
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

    let read_dir_fn = |_vm: &mut VM, ctx: NativeContext| -> Result<Value, VMRuntimeError> {
        if ctx.args.is_empty() {
            return Err(VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: crate::value::ValueType::Null,
                operation: "fs.readDir".to_string(),
            }));
        }
        let path_arg = &ctx.args[0];
        let path = path_arg.as_string().ok_or_else(|| {
            VMRuntimeError::ValueError(ValueError::TypeMismatch {
                expected: crate::value::ValueType::String,
                found: path_arg.get_type(),
                operation: "fs.readDir".to_string(),
            })
        })?;

        let entries = fs::read_dir(path).map_err(|e| {
            VMRuntimeError::ValueError(ValueError::InvalidOperation {
                operator: format!("fs.readDir: {}", e),
                left_type: crate::value::ValueType::String,
                right_type: crate::value::ValueType::Null,
            })
        })?;

        let mut table = crate::value::Table {
            data: IndexMap::new(),
            metatable: None,
        };
        for (count, entry) in entries.flatten().enumerate() {
            table.data.insert(
                count.to_string(),
                Value::string(entry.file_name().to_string_lossy().to_string()),
            );
        }

        if let Value::Object(proto) = &_vm.array_prototype {
            table.metatable = Some(proto.clone());
        }

        Ok(Value::Object(Rc::new(RefCell::new(table))))
    };

    if let Value::Object(obj) = &fs_obj {
        let mut obj = obj.borrow_mut();
        let read_file = Value::NativeFunction(Rc::new(Box::new(read_file_fn) as Box<NativeFnType>));
        let write_file = Value::NativeFunction(Rc::new(Box::new(write_file_fn) as Box<NativeFnType>));
        let exists = Value::NativeFunction(Rc::new(Box::new(exists_fn) as Box<NativeFnType>));
        let remove = Value::NativeFunction(Rc::new(Box::new(remove_fn) as Box<NativeFnType>));
        let read_dir = Value::NativeFunction(Rc::new(Box::new(read_dir_fn) as Box<NativeFnType>));

        obj.data.insert("read_file".to_string(), read_file.clone());
        obj.data.insert("readTextFile".to_string(), read_file);
        obj.data.insert("write_file".to_string(), write_file.clone());
        obj.data.insert("writeTextFile".to_string(), write_file);
        obj.data.insert("exists".to_string(), exists);
        obj.data.insert("remove".to_string(), remove);
        obj.data.insert("read_dir".to_string(), read_dir.clone());
        obj.data.insert("readDir".to_string(), read_dir);
    }

    fs_obj
}
