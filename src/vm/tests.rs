use super::*;

#[test]
fn test_simple_arithmetic() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::int(5)));
    program.add_instruction(Instruction::Push(Value::int(3)));
    program.add_instruction(Instruction::Add);

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::int(8)),
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_variable_operations() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::int(42)));
    program.add_instruction(Instruction::Store("x".to_string()));
    program.add_instruction(Instruction::Load("x".to_string()));

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::int(42)),
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_float_operations() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::float(Decimal::from_str("3.5").unwrap())));
    program.add_instruction(Instruction::Push(Value::int(2)));
    program.add_instruction(Instruction::Add);

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::float(Decimal::from_str("5.5").unwrap())),
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_string_operations() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::string("Hello".to_string())));
    program.add_instruction(Instruction::Push(Value::string(" World".to_string())));
    program.add_instruction(Instruction::Add);

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::string("Hello World".to_string())),
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_comparison_operations() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::int(5)));
    program.add_instruction(Instruction::Push(Value::int(3)));
    program.add_instruction(Instruction::LessThan);

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::bool(false)),
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_builtin_functions() {
    let mut program = Program::default();
    program.add_instruction(Instruction::Push(Value::string("Hello".to_string())));
    program.add_instruction(Instruction::Call("print".to_string(), 1));

    let mut vm = VM::new();
    let result = vm.execute(&program);

    match result {
        Ok(value) => assert_eq!(value, Value::null()),
        Err(_) => panic!("Expected success"),
    }
}
