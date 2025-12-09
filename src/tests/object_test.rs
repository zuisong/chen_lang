#[cfg(test)]
mod object_tests {
    use crate::compiler::compile;
    use crate::parse_pest::parse;
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

    /// 测试对象字面量的解析和编译
    #[test]
    fn test_object_literal() {
        let code = "#{ name: \"Alice\", age: 30 }";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(Value::Object(obj)) => {
                let table = obj.borrow();
                assert_eq!(
                    table.data.get("name"),
                    Some(&Value::string("Alice".to_string()))
                );
                assert_eq!(table.data.get("age"), Some(&Value::int(30)));
            }
            VMResult::Ok(other) => panic!("Expected Object, got {:?}", other),
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试字段访问
    #[test]
    fn test_field_access() {
        let code = "let obj = #{ name: \"Bob\", age: 25 } obj.name";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("Bob".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试字段赋值
    #[test]
    fn test_field_assignment() {
        let code = "let obj = #{ name: \"Alice\" } obj.city = \"Beijing\" obj.city";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("Beijing".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试索引访问
    #[test]
    fn test_index_access() {
        let code = "let obj = #{ name: \"Charlie\", age: 35 } let key = \"name\" obj[key]";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("Charlie".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试索引赋值
    #[test]
    fn test_index_assignment() {
        let code = "let obj = #{ name: \"Diana\" } obj[\"country\"] = \"USA\" obj[\"country\"]";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("USA".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试嵌套对象
    #[test]
    fn test_nested_objects() {
        let code = "let person = #{ name: \"Eve\", address: #{ city: \"Shanghai\", zip: 200 } } person.address.city";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::string("Shanghai".to_string()));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试空对象
    #[test]
    fn test_empty_object() {
        let code = "#{}";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(Value::Object(obj)) => {
                let table = obj.borrow();
                assert_eq!(table.data.len(), 0);
            }
            VMResult::Ok(other) => panic!("Expected Object, got {:?}", other),
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试访问不存在的字段返回 null
    #[test]
    fn test_undefined_field() {
        let code = "let obj = #{ name: \"Frank\" } obj.nonexistent";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::null());
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试对象字段包含多种类型
    #[test]
    fn test_mixed_types() {
        let code = "let obj = #{ name: \"Grace\", age: 28, active: true, score: 95.5 } obj.age";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                assert_eq!(value, Value::int(28));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }

    /// 测试对象引用共享
    #[test]
    fn test_object_reference_sharing() {
        let code = "let obj1 = #{ value: 10 } let obj2 = obj1 obj2.value = 20 obj1.value";
        
        let ast = parse(code).expect("Parse failed");
        let chars: Vec<char> = code.chars().collect();
        let program = compile(&chars, ast);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);

        match result {
            VMResult::Ok(value) => {
                // obj1 和 obj2 引用同一个对象，所以修改 obj2 会影响 obj1
                assert_eq!(value, Value::int(20));
            }
            VMResult::Error(e) => panic!("Expected success, got error: {}", e),
        }
    }
}
