use std::cell::RefCell;
use std::rc::Rc;

use chen_lang::value::Value;

use crate::common::run_chen_lang_code_with_setup;

struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }
    fn increment(&mut self) {
        self.count += 1;
    }
    fn get_value(&self) -> i32 {
        self.count
    }
}

#[test]
fn test_call_rust_object_method() {
    // 1. Create the Rust object wrapped in Rc<RefCell> for shared mutability
    let counter = Rc::new(RefCell::new(Counter::new()));

    let code = r#"
    counter.increment()
    counter.increment()
    let val = counter.get()
    io.print(val)
    "#;

    // 2. Setup the VM with the mapped object
    let output = run_chen_lang_code_with_setup(code, |vm| {
        // Create a Chen Lang Object
        let counter_obj = Value::object();

        // Helper to insert native function into the object
        let bind_method = |name: &str, func: Box<chen_lang::value::NativeFnType>| {
            if let Value::Object(obj_ref) = &counter_obj {
                obj_ref
                    .borrow_mut()
                    .data
                    .insert(name.to_string(), Value::NativeFunction(Rc::new(func)));
            }
        };

        // Bind 'increment' method
        {
            let counter = counter.clone();
            bind_method(
                "increment",
                Box::new(move |_vm, _args| {
                    counter.borrow_mut().increment();
                    Ok(Value::Null)
                }),
            );
        }

        // Bind 'get' method
        {
            let counter = counter.clone();
            bind_method(
                "get",
                Box::new(move |_vm, _args| {
                    let val = counter.borrow().get_value();
                    Ok(Value::int(val))
                }),
            );
        }

        // Register the object as a global variable
        vm.register_global_var("counter", counter_obj);
    })
    .expect("Execution failed");

    assert_eq!(output, "2");
}

#[test]
fn test_rust_object_with_args() {
    let code = r#"
    let result = calculator.add(10, 20)
    io.print(result)
    "#;

    let output = run_chen_lang_code_with_setup(code, |vm| {
        let calc_obj = Value::object();

        if let Value::Object(obj_ref) = &calc_obj {
            let mut obj = obj_ref.borrow_mut();

            // Add method takes 2 arguments
            obj.data.insert(
                "add".to_string(),
                Value::NativeFunction(Rc::new(Box::new(|_vm, args| {
                    let a = args.get(0).and_then(|v| v.to_int()).unwrap_or(0);
                    let b = args.get(1).and_then(|v| v.to_int()).unwrap_or(0);
                    Ok(Value::int(a + b))
                }))),
            );
        }

        vm.register_global_var("calculator", calc_obj);
    })
    .expect("Execution failed");

    assert_eq!(output, "30");
}
