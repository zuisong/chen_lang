use std::collections::HashMap;

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
        Operator::ADD => {
            pgrm.instructions.push(Instruction::Add);
        }
        Operator::Subtract => {
            pgrm.instructions.push(Instruction::Subtract);
        }

        Operator::LT => {
            pgrm.instructions.push(Instruction::LessThan);
        }
        _ => {
            // panic!(
            //     "{}",
            //     bop.operator
            //         .loc
            //         .debug(raw, "Unable to compile binary operation:")
            // )
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
            if let Value::Int(n) = i {
                pgrm.instructions.push(Instruction::Store(n));
            } else {
                todo!()
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
        Expression::NotStatement(_) => todo!(),
    }
}

fn compile_declaration(
    pgrm: &mut Program,
    raw: &[char],
    _: &mut HashMap<String, i32>,
    fd: FunctionDeclaration,
) {
    // Jump to end of function to guard top-level
    let done_label = format!("function_done_{}", pgrm.instructions.len());
    pgrm.instructions
        .push(Instruction::Jump(done_label.clone()));

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

    // Overwrite function lookup with total number of locals
    pgrm.syms.insert(
        fd.name,
        Symbol {
            location: function_index as i32,
            narguments,
            nlocals: new_locals.keys().len(),
        },
    );

    pgrm.syms.insert(
        done_label,
        Symbol {
            location: pgrm.instructions.len() as i32,
            narguments: 0,
            nlocals: 0,
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
    let done_label = format!("if_else_{}", pgrm.instructions.len());
    pgrm.instructions
        .push(Instruction::JumpIfNotZero(done_label.clone()));
    for stmt in if_.body {
        compile_statement(pgrm, raw, locals, stmt);
    }
    pgrm.syms.insert(
        done_label,
        Symbol {
            location: pgrm.instructions.len() as i32 - 1,
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
        Statement::Loop(_) => {
            todo!()
        }
        Statement::Assign(_) => {
            todo!()
        }
    }
}

pub fn compile(raw: &[char], ast: Ast) -> Program {
    let mut locals: HashMap<String, i32> = HashMap::new();
    let mut pgrm = Program {
        syms: HashMap::new(),
        instructions: Vec::new(),
    };
    for stmt in ast {
        compile_statement(&mut pgrm, raw, &mut locals, stmt);
    }

    pgrm
}
