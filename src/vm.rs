use std::collections::HashMap;
use tracing::debug;

use crate::value::{Value, RuntimeError};

/// 指令集 - 简化后的统一指令
#[derive(Debug, Clone)]
pub enum Instruction {
    // 栈操作
    Push(Value),              // 推入常量值
    Pop,                      // 弹出栈顶
    Dup,                      // 复制栈顶元素
    
    // 变量操作
    Load(String),             // 加载变量
    Store(String),            // 存储到变量
    
    // 运算操作（统一接口）
    Add,                      // 加法
    Subtract,                 // 减法
    Multiply,                 // 乘法
    Divide,                   // 除法
    Modulo,                   // 取模
    
    // 比较操作
    Equal,                    // 等于
    NotEqual,                 // 不等于
    LessThan,                 // 小于
    LessThanOrEqual,          // 小于等于
    GreaterThan,              // 大于
    GreaterThanOrEqual,       // 大于等于
    
    // 逻辑操作
    And,                      // 逻辑与
    Or,                       // 逻辑或
    Not,                      // 逻辑非
    
    // 控制流
    Jump(String),             // 无条件跳转
    JumpIfFalse(String),      // 条件跳转（栈顶为假时）
    JumpIfTrue(String),       // 条件跳转（栈顶为真时）
    
    // 函数调用
    Call(String, usize),      // 调用函数（函数名，参数个数）
    Return,                   // 返回
    
    // 标签（用于跳转目标）
    Label(String),            // 标签定义
}

/// 程序表示
#[derive(Debug, Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,  // 标签到指令索引的映射
}

/// VM执行结果
#[derive(Debug)]
pub enum VMResult {
    Ok(Value),
    Error(RuntimeError),
}

/// 虚拟机实现
pub struct VM {
    stack: Vec<Value>,                    // 操作数栈
    variables: HashMap<String, Value>,    // 变量存储
    pc: usize,                           // 程序计数器
    call_stack: Vec<usize>,              // 调用栈（保存返回地址）
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            pc: 0,
            call_stack: Vec::new(),
        }
    }
    
    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> VMResult {
        self.pc = 0;

        while self.pc < program.instructions.len() {
            let instruction = &program.instructions[self.pc];
            debug!("Executing instruction {}: {:?}", self.pc, instruction);

            match self.execute_instruction(instruction, program) {
                Ok(continue_execution) => {
                    if !continue_execution {
                        debug!("Execution stopped at PC {}", self.pc);
                        break;
                    }
                }
                Err(error) => {
                    debug!("Execution error at PC {}: {}", self.pc, error);
                    return VMResult::Error(error);
                }
            }

            self.pc += 1;
        }

        debug!("Execution completed. PC: {}, Stack: {:?}", self.pc, self.stack);

        // 返回栈顶值或null
        let result = self.stack.pop().unwrap_or(Value::null());
        VMResult::Ok(result)
    }
    
    /// 执行单条指令
    fn execute_instruction(&mut self, instruction: &Instruction, program: &Program) -> Result<bool, RuntimeError> {
        
        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value.clone());
            }
            
            Instruction::Pop => {
                self.stack.pop();
            }
            
            Instruction::Dup => {
                if let Some(top) = self.stack.last() {
                    self.stack.push(top.clone());
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        operator: "dup".to_string(),
                        left_type: crate::value::ValueType::Null,
                        right_type: crate::value::ValueType::Null,
                    });
                }
            }
            
            Instruction::Load(var_name) => {
                if let Some(value) = self.variables.get(var_name) {
                    debug!("Loading variable {} = {:?}", var_name, value);
                    self.stack.push(value.clone());
                } else {
                    return Err(RuntimeError::UndefinedVariable(var_name.clone()));
                }
            }
            
            Instruction::Store(var_name) => {
                if let Some(value) = self.stack.pop() {
                    debug!("Storing value {:?} to variable {}", value, var_name);
                    self.variables.insert(var_name.clone(), value);
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        operator: "store".to_string(),
                        left_type: crate::value::ValueType::Null,
                        right_type: crate::value::ValueType::Null,
                    });
                }
            }
            
            Instruction::Add => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.add(&right)?;
                self.stack.push(result);
            }
            
            Instruction::Subtract => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.subtract(&right)?;
                self.stack.push(result);
            }
            
            Instruction::Multiply => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.multiply(&right)?;
                self.stack.push(result);
            }
            
            Instruction::Divide => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.divide(&right)?;
                self.stack.push(result);
            }
            
            Instruction::Modulo => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.modulo(&right)?;
                self.stack.push(result);
            }
            
            Instruction::Equal => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.equal(&right);
                self.stack.push(result);
            }
            
            Instruction::NotEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.not_equal(&right);
                self.stack.push(result);
            }
            
            Instruction::LessThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.less_than(&right)?;
                self.stack.push(result);
            }
            
            Instruction::LessThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.less_equal(&right)?;
                self.stack.push(result);
            }
            
            Instruction::GreaterThan => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.greater_than(&right)?;
                self.stack.push(result);
            }
            
            Instruction::GreaterThanOrEqual => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.greater_equal(&right)?;
                self.stack.push(result);
            }
            
            Instruction::And => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.and(&right);
                self.stack.push(result);
            }
            
            Instruction::Or => {
                let right = self.stack.pop().unwrap_or(Value::null());
                let left = self.stack.pop().unwrap_or(Value::null());
                let result = left.or(&right);
                self.stack.push(result);
            }
            
            Instruction::Not => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let result = value.not();
                self.stack.push(result);
            }
            
            Instruction::Jump(label) => {
                if let Some(&target_pc) = program.labels.get(label) {
                    self.pc = target_pc;
                    return Ok(true); // 继续执行，但PC已更新
                } else {
                    return Err(RuntimeError::UndefinedVariable(format!("label: {}", label)));
                }
            }
            
            Instruction::JumpIfFalse(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if !condition.is_truthy() {
                    if let Some(&target_pc) = program.labels.get(label) {
                        self.pc = target_pc;
                        return Ok(true);
                    } else {
                        return Err(RuntimeError::UndefinedVariable(format!("label: {}", label)));
                    }
                }
            }
            
            Instruction::JumpIfTrue(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if condition.is_truthy() {
                    if let Some(&target_pc) = program.labels.get(label) {
                        self.pc = target_pc;
                        return Ok(true);
                    } else {
                        return Err(RuntimeError::UndefinedVariable(format!("label: {}", label)));
                    }
                }
            }
            
            Instruction::Call(func_name, arg_count) => {
                // 处理内置函数
                match func_name.as_str() {
                    "print" => {
                        // print函数不换行，按参数顺序输出
                        let mut values = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(value) = self.stack.pop() {
                                values.push(value);
                            }
                        }
                        // 反向输出以保持正确顺序
                        for value in values.iter().rev() {
                            print!("{}", value);
                        }
                        self.stack.push(Value::null()); // 返回null
                    }
                    "println" => {
                        for _ in 0..*arg_count {
                            if let Some(value) = self.stack.pop() {
                                print!("{}", value);
                            }
                        }
                        println!();
                        self.stack.push(Value::null());
                    }
                    _ => {
                        // 处理用户定义的函数
                        let func_label = format!("func_{}", func_name);
                        debug!("Calling function {} (label: {}), arg_count: {}", func_name, func_label, arg_count);
                        if let Some(&target_pc) = program.labels.get(&func_label) {
                            // 保存返回地址
                            self.call_stack.push(self.pc);
                            debug!("Saved return address: {}, jumping to {}", self.pc, target_pc);

                            // 跳转到函数
                            self.pc = target_pc;
                            return Ok(true); // 继续执行，但PC已更新
                        } else {
                            debug!("Function label {} not found in {:?}", func_label, program.labels);
                            return Err(RuntimeError::UndefinedVariable(format!("function: {}", func_name)));
                        }
                    }
                }
            }
            
            Instruction::Return => {
                // 检查是否有调用栈
                if let Some(return_pc) = self.call_stack.pop() {
                    // 从函数调用返回，恢复PC
                    // 注意：返回值应该已经在栈上了（由return语句推入）
                    debug!("Returning from function, restoring PC to {}", return_pc);
                    self.pc = return_pc;
                    return Ok(true); // 继续执行
                } else {
                    // 程序结束
                    debug!("Program end (no more call stack)");
                    return Ok(false); // 停止执行
                }
            }
            
            Instruction::Label(_) => {
                // 标签只是跳转目标，不执行任何操作
            }
        }
        
        Ok(true)
    }
    
    /// 获取当前栈状态（用于调试）
    pub fn get_stack(&self) -> &[Value] {
        &self.stack
    }
    
    /// 获取变量状态（用于调试）
    pub fn get_variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}

impl Program {
    /// 添加指令
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    /// 添加标签
    pub fn add_label(&mut self, label: String) {
        self.labels.insert(label, self.instructions.len());
    }
    
    /// 解析标签（在所有指令添加完成后调用）
    pub fn resolve_labels(&mut self) {
        for (i, instruction) in self.instructions.iter().enumerate() {
            if let Instruction::Label(label) = instruction {
                self.labels.insert(label.clone(), i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            VMResult::Ok(value) => assert_eq!(value, Value::int(8)),
            VMResult::Error(_) => panic!("Expected success"),
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
            VMResult::Ok(value) => assert_eq!(value, Value::int(42)),
            VMResult::Error(_) => panic!("Expected success"),
        }
    }
    
    #[test]
    fn test_float_operations() {
        let mut program = Program::default();
        program.add_instruction(Instruction::Push(Value::float(3.5)));
        program.add_instruction(Instruction::Push(Value::int(2)));
        program.add_instruction(Instruction::Add);
        
        let mut vm = VM::new();
        let result = vm.execute(&program);
        
        match result {
            VMResult::Ok(value) => assert_eq!(value, Value::float(5.5)),
            VMResult::Error(_) => panic!("Expected success"),
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
            VMResult::Ok(value) => assert_eq!(value, Value::string("Hello World".to_string())),
            VMResult::Error(_) => panic!("Expected success"),
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
            VMResult::Ok(value) => assert_eq!(value, Value::bool(false)),
            VMResult::Error(_) => panic!("Expected success"),
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
            VMResult::Ok(value) => assert_eq!(value, Value::null()),
            VMResult::Error(_) => panic!("Expected success"),
        }
    }
}