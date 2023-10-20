use std::collections::HashMap;
#[allow(dead_code)]
#[derive(Debug)]
enum Instruction {
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
    LessThan,
}

#[derive(Debug)]
struct Symbol {
    location: i32,
    narguments: usize,
    nlocals: usize,
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

    while pc < pgrm.instructions.len() as i32 {
        match &pgrm.instructions[pc as usize] {
            Instruction::DupPlusFP(i) => {
                data.push(data[(fp + i) as usize]);
                pc += 1;
            }
            Instruction::MoveMinusFP(local_offset, fp_offset) => {
                data[fp as usize + local_offset] = data[(fp - (fp_offset + 4)) as usize];
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
                if top == 0 {
                    pc = pgrm.syms[label].location;
                }
                pc += 1;
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
                if label == "print" {
                    for _ in 0..*narguments {
                        print!("{}", data.pop().unwrap());
                        print!(" ");
                    }
                    println!();
                    pc += 1;
                    continue;
                }

                data.push(fp);
                data.push(pc + 1);
                data.push(pgrm.syms[label].narguments as i32);
                pc = pgrm.syms[label].location;
                fp = data.len() as i32;

                // Set up space for all arguments/locals
                let mut nlocals = pgrm.syms[label].nlocals;
                while nlocals > 0 {
                    data.push(0);
                    nlocals -= 1;
                }
            }
            Instruction::Add => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left + right);
                pc += 1;
            }
            Instruction::Subtract => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left - right);
                pc += 1;
            }
            Instruction::LessThan => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left < right { 1 } else { 0 });
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

    use super::{eval, Instruction, Program};

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
