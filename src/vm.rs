use indexmap::IndexMap;
use std::io::Write;

use tracing::debug;

use crate::value::{RuntimeError, Value};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub location: i32,
    pub narguments: usize,
    pub nlocals: usize,
}

/// 指令集 - 简化后的统一指令
#[derive(Debug, Clone)]
pub enum Instruction {
    // 栈操作
    Push(Value), // 推入常量值
    Pop,         // 弹出栈顶
    Dup,         // 复制栈顶元素

    // 变量操作
    Load(String),  // 加载变量
    Store(String), // 存储到变量

    // 运算操作（统一接口）
    Add,      // 加法
    Subtract, // 减法
    Multiply, // 乘法
    Divide,   // 除法
    Modulo,   // 取模

    // 比较操作
    Equal,              // 等于
    NotEqual,           // 不等于
    LessThan,           // 小于
    LessThanOrEqual,    // 小于等于
    GreaterThan,        // 大于
    GreaterThanOrEqual, // 大于等于

    // 逻辑操作
    And, // 逻辑与
    Or,  // 逻辑或
    Not, // 逻辑非

    // 控制流
    Jump(String),        // 无条件跳转
    JumpIfFalse(String), // 条件跳转（栈顶为假时）
    JumpIfTrue(String),  // 条件跳转（栈顶为真时）

    // 函数调用
    Call(String, usize), // 调用函数（函数名，参数个数）
    Return,              // 返回

    // 标签（用于跳转目标）
    Label(String), // 标签定义

    // New Scope-related Instructions
    DupPlusFP(i32),
    MovePlusFP(usize),

    // Object operations
    NewObject,         // 创建空对象
    SetField(String),  // 设置对象字段: obj[field] = value (弹出 value, obj)
    GetField(String),  // 获取对象字段: obj[field] (弹出 obj, 压入 value)
    SetIndex,          // 设置对象索引: obj[index] = value (弹出 value, index, obj)
    GetIndex,          // 获取对象索引: obj[index] (弹出 index, obj, 压入 value)
    
    // Call function from stack
    CallStack(usize),  // Call function at stack[top-n-1], with n args
    
    // Array creation (Syntactic sugar for object with numeric keys)
    BuildArray(usize), 
}

/// 程序表示
#[derive(Debug, Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub syms: IndexMap<String, Symbol>, // 符号表
}

/// VM执行结果
#[derive(Debug)]
pub enum VMResult {
    Ok(Value),
    Error(RuntimeError),
}

/// 虚拟机实现
pub struct VM {
    stack: Vec<Value>,                 // 操作数栈
    variables: IndexMap<String, Value>, // 全局变量存储
    pc: usize,                         // 程序计数器
    fp: usize,                         // 帧指针
    call_stack: Vec<(usize, usize)>,   // 调用栈（保存返回地址, 旧fp）
    stdout: Box<dyn Write>,            // 标准输出
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        VM {
            stack: Vec::new(),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: Box::new(std::io::stdout()),
        }
    }

    pub fn with_writer(writer: Box<dyn Write>) -> Self {
        let mut variables = IndexMap::new();
        variables.insert("null".to_string(), Value::null());
        VM {
            stack: Vec::new(),
            variables,
            pc: 0,
            fp: 0,
            call_stack: Vec::new(),
            stdout: writer,
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

        debug!(
            "Execution completed. PC: {}, Stack: {:?}",
            self.pc, self.stack
        );

        // 返回栈顶值或null
        let result = self.stack.pop().unwrap_or(Value::null());
        VMResult::Ok(result)
    }

    /// 执行单条指令
    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        program: &Program,
    ) -> Result<bool, RuntimeError> {
        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value.clone());
            }

            Instruction::BuildArray(count) => {
                let mut table = crate::value::Table {
                    data: IndexMap::new(),
                    metatable: None,
                };
                
                // Pop count elements
                let start_index = self.stack.len().checked_sub(*count).ok_or(
                    RuntimeError::StackUnderflow("Stack underflow during array creation".to_string())
                )?;
                
                for i in 0..*count {
                     // Stack: [..., e0, e1, e2]
                     // start_index points to e0 (e.g. len - 3, i=0 gives index len-3)
                     let val = self.stack[start_index + i].clone();
                     // Use numeric strings keys "0", "1", ...
                     table.data.insert(i.to_string(), val);
                }
                
                self.stack.truncate(start_index);
                self.stack.push(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(table))));
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
                    debug!("All variables in VM: {:?}", self.variables);
                    self.stack.push(value.clone());
                } else {
                    // Check if it is a function
                    let func_label = format!("func_{}", var_name);
                    if program.syms.contains_key(&func_label) {
                        self.stack.push(Value::Function(var_name.clone()));
                    } else {
                        debug!(
                            "Variable {} not found! Available variables: {:?}",
                            var_name, self.variables
                        );
                        return Err(RuntimeError::UndefinedVariable(var_name.clone()));
                    }
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
                if let Some(target) = program.syms.get(label) {
                    self.pc = (target.location as usize) - 1;
                    return Ok(true); // 继续执行，但PC已更新
                } else {
                    return Err(RuntimeError::UndefinedVariable(format!("label: {}", label)));
                }
            }

            Instruction::JumpIfFalse(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if !condition.is_truthy() {
                    if let Some(target) = program.syms.get(label) {
                        self.pc = (target.location as usize) - 1;
                        return Ok(true);
                    } else {
                        return Err(RuntimeError::UndefinedVariable(format!("label: {}", label)));
                    }
                }
            }

            Instruction::JumpIfTrue(label) => {
                let condition = self.stack.pop().unwrap_or(Value::null());
                if condition.is_truthy() {
                    if let Some(target) = program.syms.get(label) {
                        self.pc = (target.location as usize) - 1;
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
                            write!(self.stdout, "{}", value).unwrap();
                        }
                        self.stdout.flush().unwrap();
                        self.stack.push(Value::null()); // 返回null
                    }
                    "println" => {
                        let mut values = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(value) = self.stack.pop() {
                                values.push(value);
                            }
                        }
                        // 反向输出以保持正确顺序
                        for value in values.iter().rev() {
                            write!(self.stdout, "{}", value).unwrap();
                        }
                        writeln!(self.stdout).unwrap();
                        self.stack.push(Value::null());
                    }
                    "set_meta" => {
                        if *arg_count != 2 {
                            return Err(RuntimeError::InvalidOperation {
                                operator: "set_meta".to_string(),
                                left_type: crate::value::ValueType::Null,
                                right_type: crate::value::ValueType::Null,
                            });
                        }
                        let metatable = self.stack.pop().unwrap_or(Value::null());
                        let obj = self.stack.pop().unwrap_or(Value::null());
                        obj.set_metatable(metatable)?;
                        self.stack.push(Value::null());
                    }
                    "get_meta" => {
                        if *arg_count != 1 {
                            return Err(RuntimeError::InvalidOperation {
                                operator: "get_meta".to_string(),
                                left_type: crate::value::ValueType::Null,
                                right_type: crate::value::ValueType::Null,
                            });
                        }
                        let obj = self.stack.pop().unwrap_or(Value::null());
                        let metatable = obj.get_metatable();
                        self.stack.push(metatable);
                    }
                    _ => {
                        // 处理用户定义的函数
                        let func_label = format!("func_{}", func_name);
                        debug!(
                            "Calling function {} (label: {}), arg_count: {}",
                            func_name, func_label, *arg_count
                        );

                        // Try direct symbol lookup, then variable lookup
                        let target_symbol = if let Some(sym) = program.syms.get(&func_label) {
                            Some(sym)
                        } else if let Some(Value::Function(real_name)) = self.variables.get(func_name) {
                             let real_label = format!("func_{}", real_name);
                             program.syms.get(&real_label)
                        } else {
                             None
                        };

                        if let Some(target_symbol) = target_symbol {
                            if *arg_count != target_symbol.narguments {
                                return Err(RuntimeError::InvalidOperation {
                                    operator: "call".to_string(),
                                    left_type: crate::value::ValueType::Null,
                                    right_type: crate::value::ValueType::Null,
                                });
                            }

                            // 1. 保存返回地址和旧fp
                            self.call_stack.push((self.pc, self.fp));

                            // 2. 设置新fp
                            self.fp = self.stack.len() - *arg_count;

                            // 3. 为局部变量分配空间
                            self.stack
                                .resize(self.fp + target_symbol.nlocals, Value::null());

                            // 4. 跳转到函数
                            self.pc = (target_symbol.location as usize) - 1;
                            return Ok(true);
                        } else {
                            debug!(
                                "Function label {} not found in {:?}",
                                func_label, program.syms
                            );
                            return Err(RuntimeError::UndefinedVariable(format!(
                                "function: {}",
                                func_name
                            )));
                        }
                    }
                }
            }
            Instruction::Return => {
                // 1. Pop the return value
                let return_value = self.stack.pop().unwrap_or(Value::null());

                // 2. Pop the call frame
                if let Some((return_pc, old_fp)) = self.call_stack.pop() {
                    // 3. Destroy the current stack frame
                    self.stack.truncate(self.fp);

                    // 4. Restore pc and fp
                    self.pc = return_pc;
                    self.fp = old_fp;

                    // 5. Push return value onto the caller's stack
                    self.stack.push(return_value);

                    return Ok(true);
                } else {
                    // No more call frames, program is ending
                    debug!("Program end (no more call stack)");
                    return Ok(false); // 停止执行
                }
            }

            Instruction::Label(_) => {
                // 标签只是跳转目标，不执行任何操作
            }

            Instruction::MovePlusFP(offset) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.fp + offset;

                // Ensure the stack is large enough
                if index >= self.stack.len() {
                    self.stack.resize(index + 1, Value::null());
                }

                self.stack[index] = value;
            }

            Instruction::DupPlusFP(offset) => {
                let index = self.fp + (*offset as usize);
                let value = self.stack.get(index).cloned().unwrap_or(Value::null());
                self.stack.push(value);
            }

            // Object operations
            Instruction::NewObject => {
                self.stack.push(Value::object());
            }

            Instruction::GetField(field) => {
                let obj = self.stack.pop().unwrap_or(Value::null());
                // Use metatable-aware field access
                let value = obj.get_field_with_meta(field);
                self.stack.push(value);
            }

            Instruction::SetField(field) => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                // Use metatable-aware field setting
                obj.set_field_with_meta(field.clone(), value)?;
            }

            Instruction::GetIndex => {
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                match obj {
                    Value::Object(table_ref) => {
                        let key = index.to_string();
                        let table = table_ref.borrow();
                        let value = table.data.get(&key).cloned().unwrap_or(Value::null());
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(RuntimeError::InvalidOperation {
                            operator: "get_index".to_string(),
                            left_type: obj.get_type(),
                            right_type: crate::value::ValueType::Null,
                        });
                    }
                }
            }

            Instruction::SetIndex => {
                let value = self.stack.pop().unwrap_or(Value::null());
                let index = self.stack.pop().unwrap_or(Value::null());
                let obj = self.stack.pop().unwrap_or(Value::null());
                match obj {
                    Value::Object(table_ref) => {
                        let key = index.to_string();
                        table_ref.borrow_mut().data.insert(key, value);
                    }
                    _ => {
                        return Err(RuntimeError::InvalidOperation {
                            operator: "set_index".to_string(),
                            left_type: obj.get_type(),
                            right_type: crate::value::ValueType::Null,
                        });
                    }
                }
            }


        Instruction::CallStack(arg_count) => {
             // 1. Get function from stack (it's below args)
             // Stack: [... func, arg1, ... argN]
             let func_idx = self.stack.len().checked_sub(*arg_count + 1).ok_or(
                 RuntimeError::StackUnderflow("CallStack: missing function".to_string())
             )?;
             
             let func_val = self.stack.remove(func_idx);
             
             match func_val {
                 Value::Function(func_name) => {
                     // Reuse logic for user defined functions
                     // Note: We don't support builtins via CallStack yet
                     let func_label = format!("func_{}", func_name);
                     debug!(
                         "Calling stack function {} (label: {}), arg_count: {}",
                         func_name, func_label, *arg_count
                     );
                     
                     if let Some(target_symbol) = program.syms.get(&func_label) {
                         if *arg_count != target_symbol.narguments {
                             return Err(RuntimeError::InvalidOperation {
                                 operator: "call_stack".to_string(),
                                 left_type: crate::value::ValueType::Function,
                                 right_type: crate::value::ValueType::Null,
                             });
                         }

                         // 1. Save return address and old fp
                         self.call_stack.push((self.pc, self.fp));

                         // 2. Set new fp
                         // Stack is now [... args], so fp is at start of args
                         self.fp = self.stack.len() - *arg_count;

                         // 3. Allocate space for locals
                         self.stack.resize(self.fp + target_symbol.nlocals, Value::null());

                         // 4. Jump
                         self.pc = (target_symbol.location as usize) - 1;
                         return Ok(true);
                     } else {
                         return Err(RuntimeError::UndefinedVariable(format!(
                             "function: {}",
                             func_name
                         )));
                     }
                 }
                 _ => {
                     return Err(RuntimeError::InvalidOperation {
                         operator: "call_stack".to_string(),
                         left_type: func_val.get_type(),
                         right_type: crate::value::ValueType::Null,
                     });
                 }
             }
        }

        }


        Ok(true)
    }

    /// 获取当前栈状态（用于调试）
    pub fn get_stack(&self) -> &[Value] {
        &self.stack
    }

    /// 获取变量状态（用于调试）
    pub fn get_variables(&self) -> &IndexMap<String, Value> {
        &self.variables
    }
}

impl Program {
    /// 添加指令
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
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
