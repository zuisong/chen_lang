#[cfg(test)]
mod object_tests {
    use crate::value::Value;
    use crate::vm::{Instruction, Program, VMResult, VM};

    /// 测试 VM 指令：NewObject
    #[test]
    fn test_vm_new_object() {
        let mut program = Program::default();
        program.add_instruction(Instruction::NewObject);

        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert!(matches!(value, Value::Object(_)));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
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

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("Chen".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
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

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::int(25));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试基础对象字面量和字段访问
    #[test]
    fn test_object_basics() {
        let code = r#"let obj = #{ name: "Chen", age: 25 }
println(obj.name)
println(obj.age)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        assert!(output.contains("Chen"), "Output should contain 'Chen'");
        assert!(output.contains("25"), "Output should contain '25'");
    }

    /// 测试字段赋值
    #[test]
    fn test_field_assignment() {
        let code = r#"let obj = #{ name: "Alice" }
obj.city = "Shanghai"
println(obj.city)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        assert!(output.contains("Shanghai"), "Output should contain 'Shanghai'");
    }

    /// 测试索引访问
    #[test]
    fn test_index_operations() {
        let code = r#"let obj = #{ name: "Bob" }
obj["country"] = "China"
println(obj["country"])"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        assert!(output.contains("China"), "Output should contain 'China'");
    }

    /// 测试嵌套对象
    #[test]
    fn test_nested_objects() {
        let code = r#"let person = #{ name: "Eve", address: #{ city: "Beijing", zip: 100000 } }
println(person.address.city)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        assert!(output.contains("Beijing"), "Output should contain 'Beijing'");
    }

    /// 测试 Metatable 原型继承
    #[test]
    fn test_metatable_inheritance() {
        let code = r#"let Animal = #{
    __index: #{
        speak: "Some sound",
        legs: 4
    }
}

let dog = #{ name: "Buddy" }
set_meta(dog, Animal)

println(dog.name)
println(dog.speak)
println(dog.legs)"#;
        
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
        let code = r#"let proto = #{ __index: #{ greet: "Hello" } }
let obj = #{ name: "Alice" }
set_meta(obj, proto)
println(obj.greet)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        assert!(output.contains("Hello"), "Output should contain 'Hello'");
    }

    /// 测试直接字段优先于 metatable
    #[test]
    fn test_metatable_precedence() {
        let code = r#"let proto = #{ value: 100 }
let obj = #{ value: 10 }
set_meta(obj, proto)
println(obj.value)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        // Should use direct field (10) not metatable field (100)
        assert!(output.contains("10"), "Output should contain '10' (direct field, not metatable)");
        assert!(!output.contains("100"), "Output should not contain '100'");
    }

    /// 测试对象引用共享
    #[test]
    fn test_object_reference() {
        let code = r#"let obj1 = #{ value: 10 }
let obj2 = obj1
obj2.value = 20
println(obj1.value)"#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());
        
        let output = result.unwrap();
        // obj1 and obj2 share the same reference, so modifying obj2 affects obj1
        assert!(output.contains("20"), "Output should contain '20' (shared reference)");
    }

    /// 测试动态添加字段
    #[test]
    fn test_dynamic_fields() {
        let code = r#"let person = #{ name: "Grace" }
person.age = 28
person.city = "Shanghai"
println(person.name)
println(person.age)
println(person.city)"#;
        
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
        let obj1 = #{ a: 1 }
        let obj2 = #{ a: 1 }
        let obj3 = obj1
        
        println(obj1 == obj2) # Should be false (different references)
        println(obj1 == obj3) # Should be true (same reference)
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
        let obj = #{
            i: 42,
            f: 3.14,
            b: true,
            s: "string",
            n: null,
            o: #{ nested: true }
        }
        println(obj.i)
        println(obj.f)
        println(obj.b)
        println(obj.s)
        println(obj.n)
        println(obj.o.nested)
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
        let grand = #{ __index: #{ name: "Grandpa" } }
        let parent = #{ __index: #{ age: 50 } }
        
        # Chain: parent -> grand
        set_meta(parent.__index, grand)
        
        let child = #{ }
        # Chain: child -> parent
        set_meta(child, parent)
        
        println("Age: " + child.age)
        println("Name: " + child.name)
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
        let meta = #{ __index: #{ x: 1 } }
        let obj = #{ }
        
        # 1. Initial should be null
        if get_meta(obj) == null {
            println("Initial: null")
        } else {
            println("Initial: not null")
        }
        
        # 2. Set meta
        set_meta(obj, meta)
        let m = get_meta(obj)
        if m == meta {
            println("Meta match: true")
        } else {
            println("Meta match: false")
        }
        
        println("Field x: " + obj.x)
        
        # 3. Clear meta
        set_meta(obj, null)
        if get_meta(obj) == null {
            println("Cleared: null")
        } else {
            println("Cleared: not null")
        }
        
        if obj.x == null {
            println("Field x cleared: null")
        } else {
            println("Field x cleared: " + obj.x)
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
        def greet(name) {
            return "Hello " + name
        }
        
        let obj = #{ }
        obj.say = greet
        
        println(obj.say("World"))
        "#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello World"));
    }
    
    /// 测试循环引用（仅创建，不打印以免栈溢出）
    #[test]
    fn test_circular_reference() {
        let code = r#"
        let a = #{ name: "A" }
        let b = #{ name: "B" }
        a.next = b
        b.prev = a
        println(a.next.name)
        println(a.next.prev.name)
        "#;
        
        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("B"));
        assert!(output.contains("A"));
    }
}
