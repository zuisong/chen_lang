#[cfg(test)]
mod object_tests {
    use pretty_assertions::assert_matches;

    use crate::value::Value;
    use crate::vm::{Instruction, Program, VM};

    #[test]
    fn test_vm_new_object() {
        let mut program = Program::default();
        program.add_instruction(Instruction::NewObject);

        let mut vm = VM::new();
        let result = vm.execute(&program);

        assert_matches!(result, Ok(Value::Object(_)), "Expected success");
    }

    #[test]
    fn test_vm_set_get_field() {
        let mut program = Program::default();

        program.add_instruction(Instruction::NewObject);

        program.add_instruction(Instruction::Dup);
        program.add_instruction(Instruction::Push(Value::string("Chen".to_string())));
        program.add_instruction(Instruction::SetField("name".to_string()));

        program.add_instruction(Instruction::GetField("name".to_string()));

        let mut vm = VM::new();
        let result = vm.execute(&program);

        let value = result.unwrap_or_else(|e| panic!("Expected success, got error: {:?}", e));
        assert_eq!(value, Value::string("Chen".to_string()));
    }

    #[test]
    fn test_vm_set_get_index() {
        let mut program = Program::default();

        program.add_instruction(Instruction::NewObject);

        program.add_instruction(Instruction::Dup);
        program.add_instruction(Instruction::Push(Value::string("age".to_string())));
        program.add_instruction(Instruction::Push(Value::int(25)));
        program.add_instruction(Instruction::SetIndex);

        program.add_instruction(Instruction::Push(Value::string("age".to_string())));
        program.add_instruction(Instruction::GetIndex);

        let mut vm = VM::new();
        let result = vm.execute(&program);

        let value = result.unwrap_or_else(|e| panic!("Expected success, got error: {:?}", e));
        assert_eq!(value, Value::int(25));
    }

    #[test]
    fn test_object_basics() {
        let code = r#"local io = require("stdlib/io")
local obj = { name = "Chen", age = 25 }
io.write(obj.name)
io.write(obj.age)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Chen"), "Output should contain 'Chen'");
        assert!(output.contains("25"), "Output should contain '25'");
    }

    #[test]
    fn test_field_assignment() {
        let code = r#"local io = require("stdlib/io")
local obj = { name = "Alice" }
obj.city = "Shanghai"
io.write(obj.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Shanghai"), "Output should contain 'Shanghai'");
    }

    #[test]
    fn test_index_operations() {
        let code = r#"local io = require("stdlib/io")
local obj = { name = "Bob" }
obj["country"] = "China"
io.write(obj["country"])"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("China"), "Output should contain 'China'");
    }

    #[test]
    fn test_nested_objects() {
        let code = r#"local io = require("stdlib/io")
local person = { name = "Eve", address = { city = "Beijing", zip = 100000 } }
io.write(person.address.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Beijing"), "Output should contain 'Beijing'");
    }

    #[test]
    fn test_metatable_inheritance() {
        let code = r#"local io = require("stdlib/io")
local Animal = {
    __index = {
        speak = "Some sound",
        legs = 4
    }
}

local dog = { name = "Buddy" }
setmetatable(dog, Animal)

io.write(dog.name)
io.write(dog.speak)
io.write(dog.legs)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Buddy"), "Output should contain 'Buddy'");
        assert!(output.contains("Some sound"), "Output should contain 'Some sound'");
        assert!(output.contains("4"), "Output should contain '4'");
    }

    #[test]
    fn test_metatable_functions() {
        let code = r#"local io = require("stdlib/io")
local proto = { __index = { greet = "Hello" } }
local obj = { name = "Alice" }
setmetatable(obj, proto)
io.write(obj.greet)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Hello"), "Output should contain 'Hello'");
    }

    #[test]
    fn test_metatable_precedence() {
        let code = r#"local io = require("stdlib/io")
local proto = { value = 100 }
local obj = { value = 10 }
setmetatable(obj, proto)
io.write(obj.value)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(
            output.contains("10"),
            "Output should contain '10' (direct field, not metatable)"
        );
        assert!(!output.contains("100"), "Output should not contain '100'");
    }

    #[test]
    fn test_object_reference() {
        let code = r#"local io = require("stdlib/io")
local obj1 = { value = 10 }
local obj2 = obj1
obj2.value = 20
io.write(obj1.value)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("20"), "Output should contain '20' (shared reference)");
    }

    #[test]
    fn test_dynamic_fields() {
        let code = r#"local io = require("stdlib/io")
local person = { name = "Grace" }
person.age = 28
person.city = "Shanghai"
io.write(person.name)
io.write(person.age)
io.write(person.city)"#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Grace"), "Output should contain 'Grace'");
        assert!(output.contains("28"), "Output should contain '28'");
        assert!(output.contains("Shanghai"), "Output should contain 'Shanghai'");
    }

    #[test]
    fn test_object_equality() {
        let code = r#"local io = require("stdlib/io")
        local obj1 = { a = 1 }
        local obj2 = { a = 1 }
        local obj3 = obj1

        io.println(obj1 == obj2) -- Should be false (different references)
        io.println(obj1 == obj3) -- Should be true (same reference)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "false", "Different objects should not be equal");
        assert_eq!(lines[1], "true", "Same object reference should be equal");
    }

    #[test]
    fn test_object_mixed_types() {
        let code = r#"local io = require("stdlib/io")
        local obj = {
            i = 42,
            f = 3.14,
            b = true,
            s = "string",
            n = nil,
            o = { nested = true }
        }
        io.write(obj.i)
        io.write(obj.f)
        io.write(obj.b)
        io.write(obj.s)
        io.write(obj.n)
        io.write(obj.o.nested)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("42"));
        assert!(output.contains("3.14"));
        assert!(output.contains("true"));
        assert!(output.contains("string"));
        assert!(output.contains("nil"));
    }

    #[test]
    fn test_metatable_chain() {
        let code = r#"local io = require("stdlib/io")
        local grand = { __index = { name = "Grandpa" } }
        local parent = { __index = { age = 50 } }

        -- Chain: parent -> grand
        setmetatable(parent.__index, grand)

        local child = { }
        -- Chain: child -> parent
        setmetatable(child, parent)

        io.write("Age: " .. child.age)
        io.write("Name: " .. child.name)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("Age: 50"), "Should find field in parent");
        assert!(output.contains("Name: Grandpa"), "Should find field in grandparent");
    }

    #[test]
    fn test_get_and_clear_meta() {
        let code = r#"local io = require("stdlib/io")
        local meta = { __index = { x = 1 } }
        local obj = { }

        -- 1. Initial should be nil
        if getmetatable(obj) == nil then
            io.write("Initial: nil")
        else
            io.write("Initial: not nil")
        end

        -- 2. Set meta
        setmetatable(obj, meta)
        local m = getmetatable(obj)
        if m == meta then
            io.write("Meta match: true")
        else
            io.write("Meta match: false")
        end

        io.write("Field x: " .. obj.x)

        -- 3. Clear meta
        setmetatable(obj, nil)
        if getmetatable(obj) == nil then
            io.write("Cleared: nil")
        else
            io.write("Cleared: not nil")
        end

        if obj.x == nil then
            io.write("Field x cleared: nil")
        else
            io.write("Field x cleared: " .. obj.x)
        end
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Initial: nil"));
        assert!(output.contains("Meta match: true"));
        assert!(output.contains("Field x: 1"));
        assert!(output.contains("Cleared: nil"));
        assert!(output.contains("Field x cleared: nil"));
    }

    #[test]
    fn test_method_call() {
        let code = r#"local io = require("stdlib/io")
        function greet(self, name)
            return "Hello " .. name
        end

        local obj = { }
        obj.say = greet

        io.write(obj:say("World"))
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_circular_reference() {
        let code = r#"local io = require("stdlib/io")
        local a = { name = "A" }
        local b = { name = "B" }
        a.next = b
        b.prev = a
        io.write(a.next.name)
        io.write(a.next.prev.name)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B"));
        assert!(output.contains("A"));
    }

    #[test]
    fn test_prototype_method() {
        let code = r#"local io = require("stdlib/io")
        function speak(self)
            return "I am an object"
        end

        local proto = { speak = speak }
        local obj = { }
        setmetatable(obj, { __index = proto })

        io.write(obj:speak())
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("I am an object"));
    }

    #[test]
    fn test_explicit_self_method() {
        let code = r#"local io = require("stdlib/io")
        function increment(self)
            self.count = self.count + 1
        end

        local counter = { count = 0 }
        counter.inc = increment

        counter:inc()
        io.write(counter.count)
        counter:inc()
        io.write(counter.count)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("1"));
        assert!(output.contains("2"));
    }

    #[test]
    fn test_class_simulation() {
        let code = r#"local io = require("stdlib/io")


        function point_str(self)
            return "(" .. self.x .. "," .. self.y .. ")"
        end
        function NewPoint(x, y)


            -- 1. define methods (usually outside as public prototype)
            local methods = {
                str = point_str
            }

            -- 2. create instance
            local instance = { x = x, y = y }

            -- 3. set up inheritance
            setmetatable(instance, { __index = methods })

            return instance
        end

        local p = NewPoint(10, 20)
        io.write(p:str())
        "#;

        let result = crate::run_captured(code.to_string());
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("(10,20)"));
    }
    #[test]
    fn test_metatable_index_function() {
        let code = r#"local io = require("stdlib/io")
        function index_handler(obj, key)
            return "fallback_" .. key
        end

        local proto = {
            __index = index_handler
        }
        local obj = { name = "Alice" }
        setmetatable(obj, proto)

        io.println(obj.name)
        io.println(obj.age)
        io.println(obj["city"])
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "Alice");
        assert_eq!(lines[1], "fallback_age");
        assert_eq!(lines[2], "fallback_city");
    }

    #[test]
    fn test_metatable_newindex_function() {
        let code = r#"local io = require("stdlib/io")
        local store = {}

        function newindex_handler(obj, key, value)
            store[key] = "intercepted_" .. value
        end

        local proto = {
            __newindex = newindex_handler
        }
        local obj = {}
        setmetatable(obj, proto)

        obj.name = "Alice"
        obj["age"] = 25

        io.println(obj.name)
        io.println(store.name)
        io.println(store.age)
        "#;

        let result = crate::run_captured(code.to_string());
        assert!(result.is_ok(), "Execution should succeed: {:?}", result.err());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "nil");
        assert_eq!(lines[1], "intercepted_Alice");
        assert_eq!(lines[2], "intercepted_25");
    }
}

#[test]
fn test_nested_function_class() {
    let code = r#"local io = require("stdlib/io")
        function NewPoint(x, y)
            function point_str(self)
                return "(" .. self.x .. "," .. self.y .. ")"
            end

            local methods = {
                str = point_str
            }

            local instance = { x = x, y = y }
            setmetatable(instance, { __index = methods })

            return instance
        end

        local p = NewPoint(10, 20)
        io.write(p:str())
        "#;

    let result = crate::run_captured(code.to_string());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("(10,20)"));
}
