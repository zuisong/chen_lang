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
}
