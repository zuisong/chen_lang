//! 一个小的玩具语言
#![allow(soft_unstable)]
// #![deny(missing_docs)]
// #![deny(unused_imports)]
// #![deny(unused_parens)]
// #![deny(dead_code)]
// #![deny(unused_mut)]
// #![deny(unreachable_code)]

use std::fmt::{Debug, Display};

use anyhow::Result;
use tracing::debug;

/// 关键字   if for
/// 函数库   print println
/// 操作符  = +-*/  ==
/// 逻辑运算符  && || ！
/// 标识符   纯字母
///
// use crate::context::Context;
use crate::expression::*;
use crate::token::*;
use crate::value::Value;

/// 表达式模块
pub mod expression;
/// 语法分析模块
pub mod parse;

/// 测试模块
#[cfg(test)]
mod tests;
/// 词法分析模块
pub mod token;

/// 值系统模块
pub mod value;
/// 虚拟机模块
pub mod vm;

#[inline]
pub(crate) fn err_msg<M>(msg: M) -> anyhow::Error
where
    M: Display + Debug + Send + Sync + 'static,
{
    anyhow::Error::msg(msg)
}

/// 运行代码
#[unsafe(no_mangle)]
pub fn run(code: String) -> Result<()> {
    let tokens = tokenlizer(code)?;
    let mut lines: Vec<Box<[Token]>> = vec![];
    let mut temp = vec![];
    for x in tokens {
        if let Token::NewLine = x {
            if !temp.is_empty() {
                lines.push(temp.into_boxed_slice());
                temp = vec![];
            }
        } else {
            temp.push(x)
        }
    }
    let (_, ast) = parse::parse_block(lines.as_slice(), 0)?;
    debug!("ast => {:?}", &ast);

    // 编译为字节码并执行
    let program = compile_to_bytecode(ast)?;
    debug!("Generated instructions: {:?}", program.instructions);

    let mut vm = vm::VM::new();
    match vm.execute(&program) {
        vm::VMResult::Ok(result) => {
            debug!("Execution result: {:?}", result);
        }
        vm::VMResult::Error(error) => {
            return Err(anyhow::Error::msg(format!("Runtime error: {}", error)));
        }
    }

    Ok(())
}

/// 编译AST为字节码
fn compile_to_bytecode(ast: Ast) -> Result<vm::Program> {
    let mut program = vm::Program::default();
    let mut functions = Vec::new();

    // 分离函数定义和主程序语句
    let mut main_statements = Vec::new();
    for statement in ast {
        match statement {
            Statement::FunctionDeclaration(func_decl) => {
                functions.push(func_decl);
            }
            _ => {
                main_statements.push(statement);
            }
        }
    }

    // 编译主程序
    for statement in main_statements {
        compile_statement(&mut program, statement)?;
    }

    // 添加跳转指令跳过函数定义
    let has_functions = !functions.is_empty();
    if has_functions {
        program.add_instruction(vm::Instruction::Jump("program_end".to_string()));
    }

    // 编译所有函数定义
    for func_decl in functions {
        compile_function_declaration(&mut program, func_decl)?;
    }

    // 添加程序结束标签
    if has_functions {
        program.add_instruction(vm::Instruction::Label("program_end".to_string()));
    }

    program.resolve_labels();
    Ok(program)
}

/// 编译单个语句
fn compile_statement(program: &mut vm::Program, statement: Statement) -> Result<()> {
    match statement {
        Statement::Expression(expr) => {
            compile_expression(program, expr)?;
            // 表达式语句的结果会被丢弃
            program.add_instruction(vm::Instruction::Pop);
        }
        Statement::Local(local) => {
            compile_expression(program, local.expression)?;
            program.add_instruction(vm::Instruction::Store(local.name));
        }
        Statement::Assign(assign) => {
            compile_expression(program, *assign.expr)?;
            program.add_instruction(vm::Instruction::Store(assign.name));
        }
        Statement::If(if_stmt) => {
            compile_if_statement(program, if_stmt)?;
        }
        Statement::Loop(loop_stmt) => {
            compile_loop_statement(program, loop_stmt)?;
        }
        Statement::FunctionDeclaration(func_decl) => {
            compile_function_declaration(program, func_decl)?;
        }
        Statement::Return(ret) => {
            compile_expression(program, ret.expression)?;
            program.add_instruction(vm::Instruction::Return);
        }
        _ => {
            // 其他语句类型暂时不支持
            debug!("Unsupported statement type: {:?}", statement);
        }
    }

    Ok(())
}

/// 编译表达式
fn compile_expression(program: &mut vm::Program, expression: Expression) -> Result<()> {
    match expression {
        Expression::Literal(lit) => {
            match lit {
                Literal::Value(value) => {
                    program.add_instruction(vm::Instruction::Push(value));
                }
                Literal::Identifier(name) => {
                    program.add_instruction(vm::Instruction::Load(name));
                }
            }
        }
        Expression::BinaryOperation(binop) => {
            compile_expression(program, *binop.left)?;
            compile_expression(program, *binop.right)?;

            let instruction = match binop.operator {
                Operator::Add => vm::Instruction::Add,
                Operator::Subtract => vm::Instruction::Subtract,
                Operator::Multiply => vm::Instruction::Multiply,
                Operator::Divide => vm::Instruction::Divide,
                Operator::Mod => vm::Instruction::Modulo,
                Operator::Equals => vm::Instruction::Equal,
                Operator::NotEquals => vm::Instruction::NotEqual,
                Operator::Lt => vm::Instruction::LessThan,
                Operator::LtE => vm::Instruction::LessThanOrEqual,
                Operator::Gt => vm::Instruction::GreaterThan,
                Operator::GtE => vm::Instruction::GreaterThanOrEqual,
                Operator::And => vm::Instruction::And,
                Operator::Or => vm::Instruction::Or,
                _ => {
                    return Err(anyhow::Error::msg(format!("Unsupported operator: {:?}", binop.operator)));
                }
            };
            program.add_instruction(instruction);
        }
        Expression::FunctionCall(func_call) => {
            // 推入参数
            for arg in &func_call.arguments {
                compile_expression(program, arg.clone())?;
            }
            program.add_instruction(vm::Instruction::Call(func_call.name, func_call.arguments.len()));
        }
        Expression::NotStatement(not_stmt) => {
            compile_expression(program, *not_stmt.expr)?;
            program.add_instruction(vm::Instruction::Not);
        }
        _ => {
            debug!("Unsupported expression type: {:?}", expression);
        }
    }

    Ok(())
}

/// 编译if语句
fn compile_if_statement(program: &mut vm::Program, if_stmt: If) -> Result<()> {
    let else_label = format!("else_{}", program.instructions.len());
    let end_label = format!("end_if_{}", program.instructions.len());

    // 编译条件
    compile_expression(program, if_stmt.test)?;

    // 条件跳转
    program.add_instruction(vm::Instruction::JumpIfFalse(else_label.clone()));

    // 编译then分支
    for stmt in if_stmt.body {
        compile_statement(program, stmt)?;
    }

    // 跳转到结束
    program.add_instruction(vm::Instruction::Jump(end_label.clone()));

    // else标签
    program.add_instruction(vm::Instruction::Label(else_label));

    // 编译else分支
    for stmt in if_stmt.else_body {
        compile_statement(program, stmt)?;
    }

    // 结束标签
    program.add_instruction(vm::Instruction::Label(end_label));

    Ok(())
}

/// 编译循环语句
fn compile_loop_statement(program: &mut vm::Program, loop_stmt: Loop) -> Result<()> {
    let start_label = format!("loop_start_{}", program.instructions.len());
    let end_label = format!("loop_end_{}", program.instructions.len());

    // 开始标签
    program.add_instruction(vm::Instruction::Label(start_label.clone()));

    // 编译条件
    compile_expression(program, loop_stmt.test)?;

    // 条件跳转
    program.add_instruction(vm::Instruction::JumpIfFalse(end_label.clone()));

    // 编译循环体
    for stmt in loop_stmt.body {
        compile_statement(program, stmt)?;
    }

    // 跳转回开始
    program.add_instruction(vm::Instruction::Jump(start_label));

    // 结束标签
    program.add_instruction(vm::Instruction::Label(end_label));

    Ok(())
}

/// 编译函数声明
fn compile_function_declaration(program: &mut vm::Program, func_decl: FunctionDeclaration) -> Result<()> {
    // 为函数添加标签（函数体将在程序末尾编译）
    let func_label = format!("func_{}", func_decl.name);
    program.add_instruction(vm::Instruction::Label(func_label.clone()));

    // 检查函数是否有显式return语句
    let has_return = func_decl.body.iter().any(|stmt| matches!(stmt, Statement::Return(_)));

    // 编译函数体
    for stmt in func_decl.body {
        compile_statement(program, stmt)?;
    }

    // 如果函数没有显式return，添加默认返回值
    if !has_return {
        program.add_instruction(vm::Instruction::Push(Value::null()));
        program.add_instruction(vm::Instruction::Return);
    }

    Ok(())
}

// 运行
// fn evaluate(ast: Ast) -> Result<Value> {
//     let mut ctx = Context::default();
//     debug!("{:?}", &ast);
//     for cmd in ast.iter() {
//         cmd.evaluate(&mut ctx)?;
//     }
//
//     Ok(Value::Void)
// }
