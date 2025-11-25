use std::collections::HashMap;
use tracing::debug;

use crate::expression::*;
use crate::token::Operator;
use crate::vm::{Instruction, Program, Symbol};

fn compile_binary_operation(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    bop: BinaryOperation,
) {
    compile_expression(pgrm, raw, locals, *bop.left);
    compile_expression(pgrm, raw, locals, *bop.right);
    match bop.operator {
        Operator::Add => {
            // 直接使用 Add 指令，VM 会处理字符串连接
            pgrm.instructions.push(Instruction::Add);
        }
        Operator::Subtract => {
            pgrm.instructions.push(Instruction::Subtract);
        }
        Operator::Multiply => {
            pgrm.instructions.push(Instruction::Multiply);
        }
        Operator::Divide => {
            pgrm.instructions.push(Instruction::Divide);
        }
        Operator::Mod => {
            pgrm.instructions.push(Instruction::Modulo);
        }
        Operator::Equals => {
            pgrm.instructions.push(Instruction::Equal);
        }
        Operator::NotEquals => {
            pgrm.instructions.push(Instruction::NotEqual);
        }
        Operator::Lt => {
            pgrm.instructions.push(Instruction::LessThan);
        }
        Operator::LtE => {
            pgrm.instructions.push(Instruction::LessThanOrEqual);
        }
        Operator::Gt => {
            pgrm.instructions.push(Instruction::GreaterThan);
        }
        Operator::GtE => {
            pgrm.instructions.push(Instruction::GreaterThanOrEqual);
        }
        Operator::And => {
            pgrm.instructions.push(Instruction::And);
        }
        Operator::Or => {
            pgrm.instructions.push(Instruction::Or);
        }
        _ => {
            panic!("Unable to compile binary operation: {:?}", bop.operator)
        }
    }
}

fn compile_function_call(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    fc: FunctionCall,
) {
    let len = fc.arguments.len();
    for arg in fc.arguments {
        compile_expression(pgrm, raw, locals, arg);
    }

    pgrm.instructions.push(Instruction::Call(fc.name, len));
}

fn compile_literal(
    pgrm: &mut Program,
    _: &[char],
    locals: &mut HashMap<String, i32>,
    lit: Literal,
) {
    match lit {
        Literal::Value(i) => {
            match i {
                Value::Int(n) => {
                    pgrm.instructions.push(Instruction::Store(n));
                }
                Value::Bool(b) => {
                    pgrm.instructions.push(Instruction::Store(if b { 1 } else { 0 }));
                }
                Value::Str(s) => {
                    // 直接存储字符串
                    pgrm.instructions.push(Instruction::StoreString(s.clone()));
                }
                _ => {
                    todo!()
                }
            }
        }
        Literal::Identifier(ident) => {
            pgrm.instructions
                .push(Instruction::DupPlusFP(locals[&ident]));
        }
    }
}

fn compile_expression(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    exp: Expression,
) {
    match exp {
        Expression::BinaryOperation(bop) => {
            compile_binary_operation(pgrm, raw, locals, bop);
        }
        Expression::FunctionCall(fc) => {
            compile_function_call(pgrm, raw, locals, fc);
        }
        Expression::Literal(lit) => {
            compile_literal(pgrm, raw, locals, lit);
        }
        Expression::NotStatement(not_stmt) => {
            compile_expression(pgrm, raw, locals, *not_stmt.expr);
            pgrm.instructions.push(Instruction::Not);
        }
    }
}

fn compile_declaration(
    pgrm: &mut Program,
    raw: &[char],
    _: &mut HashMap<String, i32>,
    fd: FunctionDeclaration,
) {
    // 函数定义不应该在主程序执行流程中，所以不需要Jump指令
    // 直接将函数体编译到指令序列的末尾

    let mut new_locals = HashMap::<String, i32>::new();

    let function_index = pgrm.instructions.len() as i32;
    let narguments = fd.parameters.len();
    for (i, param) in fd.parameters.iter().enumerate() {
        pgrm.instructions.push(Instruction::MoveMinusFP(
            i,
            narguments as i32 - (i as i32 + 1),
        ));
        new_locals.insert(param.clone(), i as i32);
    }

    for stmt in fd.body {
        compile_statement(pgrm, raw, &mut new_locals, stmt);
    }

    // 确保函数有返回指令（如果没有显式返回）
    // 注意：这里不添加Return指令，因为函数体应该有自己的Return语句

    // Overwrite function lookup with total number of locals
    pgrm.syms.insert(
        fd.name,
        Symbol {
            location: function_index,
            narguments,
            nlocals: new_locals.keys().len(),
        },
    );
}

fn compile_return(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    ret: Return,
) {
    compile_expression(pgrm, raw, locals, ret.expression);
    pgrm.instructions.push(Instruction::Return);
}

fn compile_if(pgrm: &mut Program, raw: &[char], locals: &mut HashMap<String, i32>, if_: If) {
    compile_expression(pgrm, raw, locals, if_.test);
    let else_label = format!("if_else_{}", pgrm.instructions.len());
    let end_label = format!("if_end_{}", pgrm.instructions.len());
    
    // If condition is false, jump to else
    pgrm.instructions
        .push(Instruction::JumpIfZero(else_label.clone()));
    
    // Compile then branch
    for stmt in if_.body {
        compile_statement(pgrm, raw, locals, stmt);
    }
    
    // Jump to end
    pgrm.instructions.push(Instruction::Jump(end_label.clone()));
    
    // Else label
    pgrm.syms.insert(
        else_label,
        Symbol {
            location: pgrm.instructions.len() as i32,
            nlocals: 0,
            narguments: 0,
        },
    );
    
    // Compile else branch
    for stmt in if_.else_body {
        compile_statement(pgrm, raw, locals, stmt);
    }
    
    // End label
    pgrm.syms.insert(
        end_label,
        Symbol {
            location: pgrm.instructions.len() as i32,
            nlocals: 0,
            narguments: 0,
        },
    );
}

fn compile_local(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    local: Local,
) {
    let index = locals.keys().len();
    locals.insert(local.name, index as i32);
    compile_expression(pgrm, raw, locals, local.expression);
    pgrm.instructions.push(Instruction::MovePlusFP(index));
}

fn compile_statement(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    stmt: Statement,
) {
    match stmt {
        Statement::FunctionDeclaration(fd) => compile_declaration(pgrm, raw, locals, fd),
        Statement::Return(r) => compile_return(pgrm, raw, locals, r),
        Statement::If(if_) => compile_if(pgrm, raw, locals, if_),
        Statement::Local(loc) => compile_local(pgrm, raw, locals, loc),
        Statement::Expression(e) => compile_expression(pgrm, raw, locals, e),
        Statement::Loop(e) => compile_loop(pgrm, raw, locals, e),
        Statement::Assign(e) => compile_assign(pgrm, raw, locals, e),
    }
}

fn compile_assign(
    pgrm: &mut Program,
    raw: &[char],
    locals: &mut HashMap<String, i32>,
    assign: Assign,
) {
    // 编译右侧表达式
    compile_expression(pgrm, raw, locals, *assign.expr);

    // 生成 MovePlusFP 指令
    let offset = locals[&assign.name];
    pgrm.instructions.push(Instruction::MovePlusFP(offset as usize));
}

fn compile_loop(pgrm: &mut Program, raw: &[char], locals: &mut HashMap<String, i32>, loop_: Loop) {
    // 循环开始的标签
    let loop_start = format!("loop_start_{}", pgrm.instructions.len());

    // 循环结束的标签
    let loop_end = format!("loop_end_{}", pgrm.instructions.len());

    // 插入循环开始标签
    pgrm.syms.insert(
        loop_start.clone(),
        Symbol {
            location: pgrm.instructions.len() as i32,
            narguments: 0,
            nlocals: 0,
        },
    );

    // 编译循环条件表达式
    compile_expression(pgrm, raw, locals, loop_.test);

    // 如果条件不满足,跳转到循环结束标签
    pgrm.instructions
        .push(Instruction::JumpIfZero(loop_end.clone()));

    // 编译循环体语句
    for stmt in loop_.body {
        compile_statement(pgrm, raw, locals, stmt);
    }

    // 跳转回循环开始标签,形成循环
    pgrm.instructions
        .push(Instruction::Jump(loop_start.clone()));

    // 插入循环结束标签
    pgrm.syms.insert(
        loop_end.clone(),
        Symbol {
            location: pgrm.instructions.len() as i32,
            narguments: 0,
            nlocals: 0,
        },
    );
}

pub fn compile(raw: &[char], ast: Ast) -> Program {
    let mut locals: HashMap<String, i32> = HashMap::new();
    let mut pgrm = Program {
        syms: HashMap::new(),
        instructions: Vec::new(),
    };
    
    // 首先收集所有函数定义，但不编译它们
    let mut function_declarations = Vec::new();
    let mut main_statements = Vec::new();
    
    for stmt in ast {
        match stmt {
            Statement::FunctionDeclaration(fd) => {
                debug!("Found function declaration: {}", fd.name);
                function_declarations.push(fd);
            }
            _ => {
                debug!("Found main statement: {:?}", stmt);
                main_statements.push(stmt);
            }
        }
    }
    
    debug!("Function declarations: {}", function_declarations.len());
    debug!("Main statements: {}", main_statements.len());
    
    // 记录主程序开始的位置
    let _main_start = pgrm.instructions.len();
    
    // 编译主程序
    for stmt in main_statements {
        debug!("Compiling main statement: {:?}", stmt);
        compile_statement(&mut pgrm, raw, &mut locals, stmt);
        debug!("Instructions after main statement: {:?}", pgrm.instructions);
    }
    
    // 在主程序结束后添加跳转到程序结尾的指令
    let end_label = "program_end".to_string();
    pgrm.instructions.push(Instruction::Jump(end_label.clone()));
    
    // 在指令序列末尾编译所有函数
    for fd in function_declarations {
        debug!("Compiling function: {}", fd.name);
        debug!("Before compilation, instructions len: {}", pgrm.instructions.len());
        compile_declaration(&mut pgrm, raw, &mut locals, fd);
        debug!("After compilation, instructions len: {}", pgrm.instructions.len());
    }
    
    // 添加程序结束标签
    pgrm.syms.insert(
        end_label,
        Symbol {
            location: pgrm.instructions.len() as i32,
            narguments: 0,
            nlocals: 0,
        },
    );

    pgrm
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_vm_simple() {
        use std::collections::HashMap;
        use crate::vm::{Instruction, Program, eval};
        
        let pgrm = Program {
            syms: HashMap::new(),
            instructions: vec![
                Instruction::Store(5),
                Instruction::Store(3),
                Instruction::Add,
                Instruction::Call("print".to_string(), 1),
            ],
        };

        eval(pgrm);
    }
}