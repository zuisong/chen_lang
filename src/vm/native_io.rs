use std::io::{self, BufRead, Write};
use std::rc::Rc;

use crate::value::Value;
use crate::vm::VM;

pub fn create_io_object() -> Value {
    let io_obj = Value::object();

    if let Value::Object(obj) = &io_obj {
        let mut obj = obj.borrow_mut();

        let print_fn = |vm: &mut VM, args: Vec<Value>| {
            for val in args {
                write!(vm.stdout, "{}", val).unwrap();
            }
            vm.stdout.flush().unwrap();
            Ok(Value::null())
        };

        let println_fn = |vm: &mut VM, args: Vec<Value>| {
            for val in args {
                write!(vm.stdout, "{}", val).unwrap();
            }
            writeln!(vm.stdout).unwrap();
            vm.stdout.flush().unwrap();
            Ok(Value::null())
        };

        let readline_fn = |_vm: &mut VM, _args: Vec<Value>| {
            let stdin = io::stdin();
            let mut line = String::new();
            stdin
                .lock()
                .read_line(&mut line)
                .map_err(|e| crate::vm::VMRuntimeError::UncaughtException(e.to_string()))?;
            // Remove the trailing newline character(s)
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            Ok(Value::string(line))
        };

        obj.data
            .insert("print".to_string(), Value::NativeFunction(Rc::new(Box::new(print_fn))));
        obj.data.insert(
            "println".to_string(),
            Value::NativeFunction(Rc::new(Box::new(println_fn))),
        );
        obj.data.insert(
            "readline".to_string(),
            Value::NativeFunction(Rc::new(Box::new(readline_fn))),
        );
    }

    io_obj
}
