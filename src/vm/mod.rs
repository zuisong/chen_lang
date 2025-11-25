use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Instruction {
    // Stack operations
    DupPlusFP(i32),          // 复制fp偏移量i处的值并推入栈顶
    MoveMinusFP(usize, i32), // 将fp-偏移量i处的值移动到fp+偏移量local_offset处
    MovePlusFP(usize),       // 将栈顶值移动到fp+偏移量i处
    Store(i32),              // 将栈顶值存储到偏移量i处
    Return,                  // 返回函数
    JumpIfNotZero(String),   // 如果栈顶值不为0则跳转到label处
    Jump(String),            // 无条件跳转到label处
    Call(String, usize),     // 调用函数
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Not,
    JumpIfZero(String),      // 如果栈顶值为0则跳转到label处
    StoreString(String),      // 存储字符串字面量
}

#[derive(Debug)]
pub struct Symbol {
    pub location: i32,
    pub narguments: usize,
    pub nlocals: usize,
}

#[derive(Debug, Default)]
pub struct Program {
    pub syms: HashMap<String, Symbol>,  // 函数符号表
    pub instructions: Vec<Instruction>, // 指令列表
}

pub fn eval(pgrm: Program) {
    let mut pc: i32 = 0; // 程序计数器
    let mut fp: i32 = 0; // 帧指针
    let mut data: Vec<i32> = vec![]; // 数据栈
    let mut string_pool: Vec<String> = vec![]; // 字符串池

    while pc < pgrm.instructions.len() as i32 {
        match &pgrm.instructions[pc as usize] {
            Instruction::DupPlusFP(i) => {
                data.push(data[(fp + i) as usize]);
                pc += 1;
            }
            Instruction::MoveMinusFP(local_offset, fp_offset) => {
                // Calculate the correct offset to access parameters
                // Stack layout: [..., old_fp, return_addr, narguments, arg1, arg2, ..., local1, local2, ...]
                // fp points to first local, so to access parameter at fp_offset, we need:
                // fp - (nlocals - local_offset) - 3 (for narguments, return_addr, old_fp)
                // But, compiler passes fp_offset as: narguments - (param_index + 1)
                // So we need to adjust the calculation
                let src_index = (fp - fp_offset - 3) as usize;
                let dst_index = fp as usize + local_offset;
                
                // Ensure stack has enough space for the destination
                while dst_index >= data.len() {
                    data.push(0);
                }
                
                data[dst_index] = data[src_index];
                pc += 1;
            }
            Instruction::MovePlusFP(i) => {
                let val = data.pop().unwrap();
                let index = fp as usize + *i;
                // Accounts for top-level locals
                while index >= data.len() {
                    data.push(0);
                }
                data[index] = val;
                pc += 1;
            }
            Instruction::JumpIfNotZero(label) => {
                let top = data.pop().unwrap();
                if top != 0 {
                    pc = pgrm.syms[label].location;
                } else {
                    pc += 1;
                }
            }
            Instruction::JumpIfZero(label) => {
                let top = data.pop().unwrap();
                if top == 0 {
                    pc = pgrm.syms[label].location;
                } else {
                    pc += 1;
                }
            }
            Instruction::Jump(label) => {
                pc = pgrm.syms[label].location;
            }
            Instruction::Return => {
                let ret = data.pop().unwrap();

                // Clean up the local stack
                while fp < data.len() as i32 {
                    data.pop();
                }

                // Restore pc and fp
                let mut narguments = data.pop().unwrap();
                pc = data.pop().unwrap();
                fp = data.pop().unwrap();
                println!("RETURN: Restored pc={}, fp={}", pc, fp);

                // Clean up arguments
                while narguments > 0 {
                    data.pop();
                    narguments -= 1;
                }

                // Add back return value
                data.push(ret);
            }
            Instruction::Call(label, narguments) => {
                // Handle builtin functions
                if label == "print" || label == "println" {
                    for _ in 0..*narguments {
                        let val = data.pop().unwrap();
                        // 检查是否是字符串（负数）
                        if val < 0 {
                            // 从字符串池中获取字符串
                            let index = (-val) as usize - 1; // 转换为0-based索引
                            if index < string_pool.len() {
                                print!("{}", string_pool[index]);
                            } else {
                                print!("str_{}", val);
                            }
                        } else {
                            print!("{}", val);
                        }
                        print!(" ");
                    }
                    if label == "println" {
                        println!();
                    } else {
                        print!(""); // print 也换行以保持一致性
                    }
                    pc += 1;
                    continue;
                }

                data.push(fp);
                data.push(pc + 1);
                data.push(pgrm.syms[label].narguments as i32);
                pc = pgrm.syms[label].location;

                // Set up space for all arguments/locals BEFORE setting fp
                let mut nlocals = pgrm.syms[label].nlocals;
                while nlocals > 0 {
                    data.push(0);
                    nlocals -= 1;
                }
                
                // Now set fp to point to the first local
                fp = data.len() as i32;
            }
            Instruction::Add => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                
                // 检查是否是字符串连接操作
                if left < 0 || right < 0 {
                    // 至少有一个操作数是字符串
                    let left_str = if left < 0 {
                        let index = (-left) as usize - 1;
                        if index < string_pool.len() {
                            string_pool[index].clone()
                        } else {
                            format!("str_{}", left)
                        }
                    } else {
                        left.to_string()
                    };
                    
                    let right_str = if right < 0 {
                        let index = (-right) as usize - 1;
                        if index < string_pool.len() {
                            string_pool[index].clone()
                        } else {
                            format!("str_{}", right)
                        }
                    } else {
                        right.to_string()
                    };
                    
                    // 连接字符串并添加到字符串池
                    let result = left_str + &right_str;
                    string_pool.push(result.clone());
                    let index = string_pool.len() as i32;
                    data.push(-index);
                } else {
                    // 普通整数加法
                    data.push(left + right);
                }
                pc += 1;
            }
            Instruction::Subtract => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left - right);
                pc += 1;
            }
            Instruction::Multiply => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left * right);
                pc += 1;
            }
            Instruction::Divide => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left / right);
                pc += 1;
            }
            Instruction::Modulo => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left % right);
                pc += 1;
            }
            Instruction::Equal => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left == right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::NotEqual => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left != right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::LessThan => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left < right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::LessThanOrEqual => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left <= right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::GreaterThan => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left > right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::GreaterThanOrEqual => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left >= right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::And => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left != 0 && right != 0 { 1 } else { 0 });
                pc += 1;
            }
            Instruction::Or => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left != 0 || right != 0 { 1 } else { 0 });
                pc += 1;
            }
            Instruction::Not => {
                let val = data.pop().unwrap();
                data.push(if val == 0 { 1 } else { 0 });
                pc += 1;
            }
            Instruction::StoreString(s) => {
                // 将字符串添加到字符串池，并存储其索引（负数）
                string_pool.push(s.clone());
                let index = string_pool.len() as i32;
                data.push(-index); // 使用负数表示字符串索引
                pc += 1;
            }
            Instruction::Store(n) => {
                data.push(*n);
                pc += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Instruction, Program, eval};

    #[test]
    fn test_print_1() {
        let pgrm = Program {
            syms: HashMap::new(),
            instructions: vec![
                Instruction::Store(1),
                Instruction::Call("print".to_string(), 1),
            ],
        };

        eval(pgrm)
    }

    #[test]
    fn test_if_print_1() {
        let pgrm = Program {
            syms: {
                let mut m = HashMap::new();
                m.insert(
                    "if_1".to_string(),
                    super::Symbol {
                        location: 3,
                        narguments: 1,
                        nlocals: 1,
                    },
                );
                m
            },
            instructions: vec![
                Instruction::Store(2),
                Instruction::Store(1),
                Instruction::LessThan,
                Instruction::JumpIfNotZero("if_1".to_string()),
                Instruction::Store(1),
                Instruction::Call("print".to_string(), 1),
            ],
        };

        eval(pgrm)
    }
}