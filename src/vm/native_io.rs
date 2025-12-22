use std::io::Write;
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

        obj.data
            .insert("print".to_string(), Value::NativeFunction(Rc::new(Box::new(print_fn))));
        obj.data.insert(
            "println".to_string(),
            Value::NativeFunction(Rc::new(Box::new(println_fn))),
        );
    }

    io_obj
}
