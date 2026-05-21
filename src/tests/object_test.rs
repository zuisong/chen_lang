#[cfg(test)]
mod object_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use pretty_assertions::assert_matches;

    use crate::value::Value;
    use crate::vm::{Instruction, NativeFnType, Program, VM, VMRuntimeError};
    use crate::{compiler, parser};

    fn run_value(code: &str) -> Result<Value, VMRuntimeError> {
        let ast = parser::parse_from_source(code).unwrap();
        let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);
        let mut vm = VM::new();
        vm.execute(&program).map_err(|err| err.error)
    }

    fn run_capture_values(code: &str) -> Result<Vec<Value>, VMRuntimeError> {
        let ast = parser::parse_from_source(code).unwrap();
        let program = compiler::compile(&code.chars().collect::<Vec<char>>(), ast);
        let values = Rc::new(RefCell::new(Vec::new()));
        let captured_values = values.clone();
        let capture_fn = move |_vm: &mut VM, ctx: crate::value::NativeContext| {
            captured_values.borrow_mut().extend(ctx.args);
            Ok(Value::null())
        };

        let mut vm = VM::new();
        vm.variables.insert(
            "capture".to_string(),
            Value::NativeFunction(Rc::new(Box::new(capture_fn) as Box<NativeFnType>)),
        );
        vm.execute(&program).map_err(|err| err.error)?;
        Ok(values.borrow().clone())
    }

    /// 测试 VM 指令：NewObject
    #[test]
    fn test_vm_new_object() {
        let mut program = Program::default();
        program.add_instruction(Instruction::NewObject);

        let mut vm = VM::new();
        let result = vm.execute(&program);

        assert_matches!(result, Ok(Value::Object(_)), "Expected success");
    }

    /// 测试 VM 指令：SetField 和 GetField
    #[test]
    fn test_vm_set_get_field() {
        let mut program = Program::default();

        // 创建对象
        program.add_instruction(Instruction::NewObject);

        // 设置字段 name = "Chen"
        program.add_instruction(Instruction::Dup); // 复制对象引用
        program.add_instruction(Instruction::Push(Value::string("Chen".to_string())));
        program.add_instruction(Instruction::SetField("name".to_string()));

        // 获取字段 name
        program.add_instruction(Instruction::GetField("name".to_string()));

        let mut vm = VM::new();
        let result = vm.execute(&program);

        let value = result.unwrap_or_else(|e| panic!("Expected success, got error: {:?}", e));
        assert_eq!(value, Value::string("Chen".to_string()));
    }

    /// 测试 VM 指令：SetIndex 和 GetIndex
    #[test]
    fn test_vm_set_get_index() {
        let mut program = Program::default();

        // 创建对象
        program.add_instruction(Instruction::NewObject);

        // 设置索引 obj["age"] = 25
        program.add_instruction(Instruction::Dup);
        program.add_instruction(Instruction::Push(Value::string("age".to_string())));
        program.add_instruction(Instruction::Push(Value::int(25)));
        program.add_instruction(Instruction::SetIndex);

        // 获取索引 obj["age"]
        program.add_instruction(Instruction::Push(Value::string("age".to_string())));
        program.add_instruction(Instruction::GetIndex);

        let mut vm = VM::new();
        let result = vm.execute(&program);

        let value = result.unwrap_or_else(|e| panic!("Expected success, got error: {:?}", e));
        assert_eq!(value, Value::int(25));
    }

    /// 测试基础对象字面量和字段访问
    #[test]
    fn test_object_basics() {
        let code = r#"
let obj = { name: "Chen", age: 25 }
console.log(obj.name)
console.log(obj.age)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Chen"), "Output should contain 'Chen'");
        assert!(output.contains("25"), "Output should contain '25'");
    }

    /// 测试字段赋值
    #[test]
    fn test_field_assignment() {
        let code = r#"
let obj = { name: "Alice" }
obj.city = "Shanghai"
console.log(obj.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Shanghai"), "Output should contain 'Shanghai'");
    }

    /// 测试索引访问
    #[test]
    fn test_index_operations() {
        let code = r#"
let obj = { name: "Bob" }
obj["country"] = "China"
console.log(obj["country"])"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("China"), "Output should contain 'China'");
    }

    /// 测试嵌套对象
    #[test]
    fn test_nested_objects() {
        let code = r#"
let person = { name: "Eve", address: { city: "Beijing", zip: 100000 } }
console.log(person.address.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Beijing"), "Output should contain 'Beijing'");
    }

    /// 测试 Metatable 原型继承
    #[test]
    fn test_metatable_inheritance() {
        let code = r#"
let Animal = {
    __index: {
        speak: "Some sound",
        legs: 4
    }
}

let dog = { name: "Buddy" }
Chen.setMeta(dog, Animal)

console.log(dog.name)
console.log(dog.speak)
console.log(dog.legs)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Buddy"), "Output should contain 'Buddy'");
        assert!(output.contains("Some sound"), "Output should contain 'Some sound'");
        assert!(output.contains("4"), "Output should contain '4'");
    }

    /// 测试 set_meta 和 get_meta
    #[test]
    fn test_metatable_functions() {
        let code = r#"
let proto = { __index: { greet: "Hello" } }
let obj = { name: "Alice" }
Chen.setMeta(obj, proto)
console.log(obj.greet)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Hello"), "Output should contain 'Hello'");
    }

    /// 测试直接字段优先于 metatable
    #[test]
    fn test_metatable_precedence() {
        let code = r#"
let proto = { value: 100 }
let obj = { value: 10 }
Chen.setMeta(obj, proto)
console.log(obj.value)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        // Should use direct field (10) not metatable field (100)
        assert!(
            output.contains("10"),
            "Output should contain '10' (direct field, not metatable)"
        );
        assert!(!output.contains("100"), "Output should not contain '100'");
    }

    /// 测试对象引用共享
    #[test]
    fn test_object_reference() {
        let code = r#"
let obj1 = { value: 10 }
let obj2 = obj1
obj2.value = 20
console.log(obj1.value)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        // obj1 and obj2 share the same reference, so modifying obj2 affects obj1
        assert!(output.contains("20"), "Output should contain '20' (shared reference)");
    }

    /// 测试动态添加字段
    #[test]
    fn test_dynamic_fields() {
        let code = r#"
let person = { name: "Grace" }
person.age = 28
person.city = "Shanghai"
console.log(person.name)
console.log(person.age)
console.log(person.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Grace"), "Output should contain 'Grace'");
        assert!(output.contains("28"), "Output should contain '28'");
        assert!(output.contains("Shanghai"), "Output should contain 'Shanghai'");
    }

    /// 测试对象相等性（引用比较）
    #[test]
    fn test_object_equality() {
        let code = r#"
        let obj1 = { a: 1 }
        let obj2 = { a: 1 }
        let obj3 = obj1

        console.log(obj1 == obj2) // Should be false (different references)
        console.log(obj1 == obj3) // Should be true (same reference)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "false", "Different objects should not be equal");
        assert_eq!(lines[1], "true", "Same object reference should be equal");
    }

    /// 测试对象存储多种类型
    #[test]
    fn test_object_mixed_types() {
        let code = r#"
        let obj = {
            i: 42,
            f: 3.14,
            b: true,
            s: "string",
            n: null,
            o: { nested: true }
        }
        console.log(obj.i)
        console.log(obj.f)
        console.log(obj.b)
        console.log(obj.s)
        console.log(obj.n)
        console.log(obj.o.nested)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("42"));
        assert!(output.contains("3.14"));
        assert!(output.contains("true"));
        assert!(output.contains("string"));
        assert!(output.contains("null"));
    }

    /// 测试多层 Metatable 继承
    #[test]
    fn test_metatable_chain() {
        let code = r#"
        let grand = { __index: { name: "Grandpa" } }
        let parent = { __index: { age: 50 } }

        // Chain: parent -> grand
        Chen.setMeta(parent.__index, grand)

        let child = { }
        // Chain: child -> parent
        Chen.setMeta(child, parent)

        console.log("Age: " + child.age)
        console.log("Name: " + child.name)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Age: 50"), "Should find field in parent");
        assert!(output.contains("Name: Grandpa"), "Should find field in grandparent");
    }

    /// 测试 get_meta 和清除 meta
    #[test]
    fn test_get_and_clear_meta() {
        let code = r#"
        let meta = { __index: { x: 1 } }
        let obj = { }

        // 1. Initial should be null
        if (Chen.getMeta(obj) == null) {
            console.log("Initial: null")
        } else {
            console.log("Initial: not null")
        }

        // 2. Set meta
        Chen.setMeta(obj, meta)
        let m = Chen.getMeta(obj)
        if (m == meta) {
            console.log("Meta match: true")
        } else {
            console.log("Meta match: false")
        }

        console.log("Field x: " + obj.x)

        // 3. Clear meta
        Chen.setMeta(obj, null)
        if (Chen.getMeta(obj) == null) {
            console.log("Cleared: null")
        } else {
            console.log("Cleared: not null")
        }

        if (obj.x == null) {
            console.log("Field x cleared: null")
        } else {
            console.log("Field x cleared: " + obj.x)
        }
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Initial: null"));
        assert!(output.contains("Meta match: true"));
        assert!(output.contains("Field x: 1"));
        assert!(output.contains("Cleared: null"));
        assert!(output.contains("Field x cleared: null"));
    }

    /// 测试方法调用 (Assign function to field)
    #[test]
    fn test_method_call() {
        let code = r#"
        function greet(name) {
            return "Hello " + name + " from " + this.name
        }

        let obj = { name: "Object" }
        obj.say = greet

        console.log(obj.say("World"))
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.contains("Hello World from Object"));
    }

    /// 测试循环引用（仅创建，不打印以免栈溢出）
    #[test]
    fn test_circular_reference() {
        let code = r#"
        let a = { name: "A" }
        let b = { name: "B" }
        a.next = b
        b.prev = a
        console.log(a.next.name)
        console.log(a.next.prev.name)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B"));
        assert!(output.contains("A"));
    }

    /// 测试原型方法继承 (通过 __index)
    #[test]
    fn test_prototype_method() {
        let code = r#"
        function speak() {
            return "I am " + this.name
        }

        let proto = { speak: speak }
        let obj = { name: "an object" }
        Chen.setMeta(obj, { __index: proto })

        console.log(obj.speak())
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.contains("I am an object"));
    }

    /// 测试 this 绑定 (模拟方法)
    #[test]
    fn test_this_binding_method() {
        let code = r#"
        function increment() {
            this.count = this.count + 1
        }

        let counter = { count: 0 }
        counter.inc = increment

        counter.inc()
        console.log(counter.count)
        counter.inc()
        console.log(counter.count)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.contains("1"));
        assert!(output.contains("2"));
    }

    /// 模拟 Class 的行为 (构造函数 + 原型链方法)
    #[test]
    fn test_class_simulation() {
        let code = r#"
        function point_str() {
            return "(" + this.x + "," + this.y + ")"
        }
        function NewPoint(x, y) {
            // 1. 定义方法 (通常这些放在外面作为公共原型)
            let methods = {
                str: point_str
            }

            // 2. 创建实例
            let instance = { x: x, y: y }

            // 3. 建立继承关系
            Chen.setMeta(instance, { __index: methods })

            return instance
        }

        let p = NewPoint(10, 20)
        console.log(p.str()) // 像调用对象方法一样
        "#;

        let result = crate::run_captured(code.to_string());
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("(10,20)"));
    }
    /// 测试 __index 是一个函数的情况
    #[test]
    fn test_metatable_index_function() {
        let code = r#"
        function index_handler(obj, key) {
            return "fallback_" + key
        }

        let proto = {
            __index: index_handler
        }
        let obj = { name: "Alice" }
        Chen.setMeta(obj, proto)

        console.log(obj.name)
        console.log(obj.age)
        console.log(obj["city"])
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "Alice");
        assert_eq!(lines[1], "fallback_age");
        assert_eq!(lines[2], "fallback_city");
    }

    /// 测试 __newindex 是一个函数的情况
    #[test]
    fn test_metatable_newindex_function() {
        let code = r#"
        let store = {}
        
        function newindex_handler(obj, key, value) {
            store[key] = "intercepted_" + value 
        }

        let proto = {
            __newindex: newindex_handler
        }
        let obj = {}
        Chen.setMeta(obj, proto)

        obj.name = "Alice"
        obj["age"] = 25

        console.log(obj.name)
        console.log(store.name)
        console.log(store.age)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "null");
        assert_eq!(lines[1], "intercepted_Alice");
        assert_eq!(lines[2], "intercepted_25");
    }

    #[test]
    fn test_js_method_call_binds_this() {
        let values = run_capture_values(
            r#"
            function getName() {
                return this.name
            }

            let obj = { name: "Alice", getName: getName }
            capture(obj.getName())
            "#,
        )
        .unwrap();

        assert_eq!(values, vec![Value::string("Alice".to_string())]);
    }

    #[test]
    fn test_js_ordinary_function_call_does_not_bind_this() {
        let error = run_value(
            r#"
            function getName() {
                return this.name
            }

            let obj = { name: "Alice", getName: getName }
            let get = obj.getName
            get()
            "#,
        )
        .unwrap_err();

        assert_matches!(error, VMRuntimeError::ValueError(_));
    }

    #[test]
    fn test_js_unbound_this_errors() {
        let error = run_value(
            r#"
            function readThis() {
                return this
            }

            readThis()
            "#,
        )
        .unwrap_err();

        assert_matches!(error, VMRuntimeError::ValueError(_));
    }

    #[test]
    fn test_js_nested_method_calls_restore_outer_this() {
        let values = run_capture_values(
            r#"
            let obj2 = {
                name: "inner",
                method: function() {
                    return this.name
                }
            }

            let obj1 = {
                name: "outer",
                method: function() {
                    obj2.method()
                    capture(this.name)
                }
            }

            obj1.method()
            "#,
        )
        .unwrap();

        assert_eq!(values, vec![Value::string("outer".to_string())]);
    }

    #[test]
    fn test_js_object_create_inherits_missing_fields() {
        let values = run_capture_values(
            r#"
            let proto = { name: "proto" }
            let obj = Object.create(proto)
            obj.own = "own"
            capture(obj.name, obj.own)
            "#,
        )
        .unwrap();

        assert_eq!(
            values,
            vec![Value::string("proto".to_string()), Value::string("own".to_string())]
        );
    }

    #[test]
    fn test_js_object_keys_and_entries_return_arrays() {
        let values = run_capture_values(
            r#"
            let obj = { a: 1, b: 2 }
            let keys = Object.keys(obj)
            let entries = Object.entries(obj)
            capture(keys[0], keys[1], entries[0][0], entries[0][1], entries[1][0], entries[1][1])
            "#,
        )
        .unwrap();

        assert_eq!(
            values,
            vec![
                Value::string("a".to_string()),
                Value::string("b".to_string()),
                Value::string("a".to_string()),
                Value::int(1),
                Value::string("b".to_string()),
                Value::int(2),
            ]
        );
    }

    #[test]
    fn test_js_chen_set_meta_and_get_meta() {
        let values = run_capture_values(
            r#"
            let meta = { __index: { value: 42 } }
            let obj = {}
            Chen.setMeta(obj, meta)
            capture(obj.value, Chen.getMeta(obj) == meta)
            "#,
        )
        .unwrap();

        assert_eq!(values, vec![Value::int(42), Value::bool(true)]);
    }

    #[test]
    fn test_js_missing_field_returns_null() {
        let values = run_capture_values(
            r#"
            let obj = {}
            capture(obj.missing)
            "#,
        )
        .unwrap();

        assert_eq!(values, vec![Value::null()]);
    }

    #[test]
    fn test_js_runtime_globals_are_available() {
        let values = run_capture_values(
            r#"
            capture(Chen != null, Chen.fs != null, Chen.process != null, Chen.timer != null, Chen.date != null, Chen.coroutine != null)
            "#,
        )
        .unwrap();

        assert_eq!(
            values,
            vec![
                Value::bool(true),
                Value::bool(true),
                Value::bool(true),
                Value::bool(true),
                Value::bool(true),
                Value::bool(true),
            ]
        );
    }

    #[test]
    fn test_js_console_log_and_print() {
        let output = crate::run_captured(
            r#"
            console.print("hello")
            console.log(" world")
            "#
            .to_string(),
        )
        .unwrap();

        assert_eq!(output, "hello world\n");
    }

    #[test]
    fn test_js_extracted_console_methods_do_not_drop_first_argument() {
        let output = crate::run_captured(
            r#"
            let print = console.print
            let log = console.log
            print("hello")
            log(" world")
            "#
            .to_string(),
        )
        .unwrap();

        assert_eq!(output, "hello world\n");
    }

    #[test]
    fn test_js_extracted_chen_runtime_methods_work_without_receiver() {
        let output = crate::run_captured(
            r#"
            let sleep = Chen.timer.sleepMs
            let exec = Chen.process.exec
            sleep(0)
            console.print(exec("printf ok").stdout.trim())
            "#
            .to_string(),
        )
        .unwrap();

        assert_eq!(output, "ok");
    }

    #[test]
    fn test_js_json_global_parse_and_stringify() {
        let output = crate::run_captured(
            r#"
            let text = JSON.stringify({ ok: true })
            let parsed = JSON.parse(text)
            console.log(parsed.ok)
            "#
            .to_string(),
        )
        .unwrap();

        assert_eq!(output.trim(), "true");
    }

    #[test]
    fn test_js_chen_load_uses_cache() {
        let path = "target/chen_lang_test_cached_module.chen.js";
        std::fs::create_dir_all("target").unwrap();
        std::fs::write(
            path,
            r#"
            console.print("loaded")
            return { value: 7 }
            "#,
        )
        .unwrap();

        let code = format!(
            r#"
            let first = Chen.load("{path}")
            let second = Chen.load("{path}")
            console.log(first.value + second.value)
            "#
        );
        let output = crate::run_captured(code).unwrap();

        assert_eq!(output, "loaded14\n");
    }

    #[test]
    fn test_js_fs_camel_case_api_names() {
        let path = "target/chen_lang_test_fs_api.txt";
        let code = format!(
            r#"
            Chen.fs.writeTextFile("{path}", "hello")
            let text = Chen.fs.readTextFile("{path}")
            let entries = Chen.fs.readDir("target")
            console.log(text, Chen.fs.exists("{path}"), entries.length > 0)
            Chen.fs.remove("{path}")
            "#
        );
        let output = crate::run_captured(code).unwrap();

        assert_eq!(output.trim(), "hello true true");
        assert!(!std::path::Path::new(path).exists());
    }

    #[test]
    fn test_js_collection_length_and_array_methods() {
        let output = crate::run_captured(
            r#"
            let arr = [1, 2]
            console.log(arr.length)
            console.log(arr.push(3))
            console.log(arr.length)
            console.log(arr.pop())
            console.log(arr.length)
            "#
            .to_string(),
        )
        .unwrap();

        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines, vec!["2", "3", "3", "3", "2"]);
    }

    #[test]
    fn test_js_string_length_and_methods() {
        let output = crate::run_captured(
            r#"
            let s = "  Chen  "
            console.log(s.length)
            console.log(s.trim())
            console.log("abc".toUpperCase())
            console.log("ABC".toLowerCase())
            "#
            .to_string(),
        )
        .unwrap();

        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines, vec!["8", "Chen", "ABC", "abc"]);
    }

    #[test]
    fn test_js_runtime_api_aliases_exist() {
        let values = run_capture_values(
            r#"
            capture(Chen.timer.sleepMs != null, Chen.http == null || Chen.http.fetch != null, Chen.date.now != null, Chen.process.exec != null)
            "#,
        )
        .unwrap();

        assert_eq!(
            values,
            vec![
                Value::bool(true),
                Value::bool(true),
                Value::bool(true),
                Value::bool(true)
            ]
        );
    }

    #[test]
    fn test_js_expression_semantics() {
        let output = crate::run_captured(
            r#"
            console.log(null || "fallback")
            console.log(0 && "x")
            console.log(!"")
            console.log("count: " + 3)
            if ("") {
                console.log("yes")
            } else {
                console.log("no")
            }
            "#
            .to_string(),
        )
        .unwrap();

        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines, vec!["fallback", "0", "true", "count: 3", "no"]);
    }

    /// 测试 unbound this (extracted method)
    #[test]
    fn test_unbound_this() {
        let code = r#"
        function getName() {
            return this.name
        }
        let obj = { name: "Alice", getName: getName }
        let m = obj.getName // extracted
        m()
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(
            result.is_err(),
            "Expected error when calling extracted method without this binding"
        );
    }

    /// 测试 nested calls and this stability
    #[test]
    fn test_nested_this_stability() {
        let code = r#"
        let obj1 = {
            name: "obj1",
            method: function() {
                console.log(this.name)
                obj2.method()
                console.log(this.name)
            }
        }
        let obj2 = {
            name: "obj2",
            method: function() {
                console.log(this.name)
            }
        }
        obj1.method()
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "obj1");
        assert_eq!(lines[1], "obj2");
        assert_eq!(lines[2], "obj1");
    }
}

/// 测试嵌套函数定义 (Nested Functions)
#[test]
fn test_nested_function_class() {
    let code = r#"
        function NewPoint(x, y) {
            // 嵌套定义函数
            function point_str() {
                return "(" + this.x + "," + this.y + ")"
            }

            let methods = {
                str: point_str
            }

            let instance = { x: x, y: y }
            Chen.setMeta(instance, { __index: methods })

            return instance
        }

        let p = NewPoint(10, 20)
        console.log(p.str())
        "#;

    let result = crate::run_captured(code.to_string());
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.contains("(10,20)"));
}
