use std::collections::HashMap;

use thiserror::Error;

use crate::expression::{
    Ast, BinaryOperation, Expression, FunctionCall, If, Literal, Loop, Statement,
};
use crate::tokenizer::{Location, Operator};
use crate::value::Value;
use crate::vm::{Instruction, Program, Symbol};

#[derive(Debug, Error)]
pub enum TinyCompileError {
    #[error(transparent)]
    Parser(#[from] crate::parser::ParserError),
    #[error(transparent)]
    Runtime(#[from] crate::vm::RuntimeErrorWithContext),
    #[error("Unsupported syntax: {0}")]
    Unsupported(String),
    #[error("Compile error: {0}")]
    Compile(String),
}

#[derive(Default)]
struct Scope {
    locals: HashMap<String, i32>,
}

enum VarLocation {
    Local(i32),
    Global(String),
}

struct LoopLabels {
    start: String,
    end: String,
}

pub struct TinyCompiler {
    program: Program,
    scopes: Vec<Scope>,
    locals_count: usize,
    loop_stack: Vec<LoopLabels>,
}

pub fn compile_ast(ast: &Ast) -> Result<Program, TinyCompileError> {
    let mut compiler = TinyCompiler::new();
    compiler.compile_program(ast)?;
    Ok(compiler.program)
}

impl TinyCompiler {
    fn new() -> Self {
        Self {
            program: Program::default(),
            scopes: vec![Scope::default()],
            locals_count: 0,
            loop_stack: Vec::new(),
        }
    }

    fn emit(&mut self, instr: Instruction, loc: Location) {
        let idx = self.program.instructions.len();
        self.program.instructions.push(instr);
        self.program.lines.insert(idx, loc);
    }

    fn define_label(&mut self, name: String) {
        self.program.syms.insert(
            name,
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );
    }

    fn unique_id(&self) -> usize {
        self.program.instructions.len()
    }

    fn begin_scope(&mut self) -> usize {
        self.scopes.push(Scope::default());
        self.locals_count
    }

    fn end_scope(&mut self, base_locals: usize, loc: Location, preserve_top: bool) {
        let scope = self.scopes.pop().expect("scope stack underflow");
        let count = scope.locals.len();
        self.locals_count = base_locals;

        if count == 0 {
            return;
        }

        if preserve_top {
            self.emit(Instruction::MovePlusFP(base_locals), loc);
            for _ in 0..count.saturating_sub(1) {
                self.emit(Instruction::Pop, loc);
            }
        } else {
            for _ in 0..count {
                self.emit(Instruction::Pop, loc);
            }
        }
    }

    fn define_variable(&mut self, name: String) -> VarLocation {
        let is_global = self.scopes.len() == 1;
        if is_global {
            VarLocation::Global(name)
        } else {
            let scope = self.scopes.last_mut().expect("no active scope");
            let index = self.locals_count as i32;
            scope.locals.insert(name, index);
            self.locals_count += 1;
            VarLocation::Local(index)
        }
    }

    fn resolve_variable(&self, name: &str) -> VarLocation {
        for scope in self.scopes.iter().rev() {
            if let Some(index) = scope.locals.get(name) {
                return VarLocation::Local(*index);
            }
        }
        VarLocation::Global(name.to_string())
    }

    fn compile_program(&mut self, ast: &Ast) -> Result<(), TinyCompileError> {
        for stmt in ast {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), TinyCompileError> {
        match stmt {
            Statement::Local(local) => {
                self.compile_expression(&local.expression)?;
                match self.define_variable(local.name.clone()) {
                    VarLocation::Local(offset) => {
                        self.emit(Instruction::MovePlusFP(offset as usize), local.loc);
                    }
                    VarLocation::Global(name) => {
                        self.emit(Instruction::Store(name), local.loc);
                    }
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                let should_pop = match expr {
                    Expression::If(if_expr) => {
                        self.compile_if(if_expr, false)?;
                        false
                    }
                    _ => {
                        self.compile_expression(expr)?;
                        true
                    }
                };
                if should_pop {
                    self.emit(Instruction::Pop, self.expr_loc(expr));
                }
                Ok(())
            }
            Statement::Loop(loop_stmt) => self.compile_loop(loop_stmt),
            Statement::Break(loc) => {
                let end_label = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| TinyCompileError::Compile("break outside loop".into()))?
                    .end
                    .clone();
                self.emit(Instruction::Jump(end_label), *loc);
                Ok(())
            }
            Statement::Continue(loc) => {
                let start_label = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| TinyCompileError::Compile("continue outside loop".into()))?
                    .start
                    .clone();
                self.emit(Instruction::Jump(start_label), *loc);
                Ok(())
            }
            Statement::FunctionDeclaration(_) => Err(TinyCompileError::Unsupported(
                "function declaration".to_string(),
            )),
            Statement::Return(_) => Err(TinyCompileError::Unsupported("return".to_string())),
            Statement::Assign(_) => Err(TinyCompileError::Unsupported("assignment".to_string())),
            Statement::SetField { .. } => Err(TinyCompileError::Unsupported("set field".to_string())),
            Statement::SetIndex { .. } => Err(TinyCompileError::Unsupported("set index".to_string())),
            Statement::TryCatch(_) => Err(TinyCompileError::Unsupported("try/catch".to_string())),
            Statement::Throw { .. } => Err(TinyCompileError::Unsupported("throw".to_string())),
        }
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), TinyCompileError> {
        match expr {
            Expression::Literal(lit, loc) => self.compile_literal(lit, *loc),
            Expression::Identifier(name, loc) => {
                match self.resolve_variable(name) {
                    VarLocation::Local(offset) => self.emit(Instruction::DupPlusFP(offset), *loc),
                    VarLocation::Global(name) => self.emit(Instruction::Load(name), *loc),
                }
                Ok(())
            }
            Expression::BinaryOperation(bop) => self.compile_binary(bop),
            Expression::FunctionCall(fc) => self.compile_call(fc),
            Expression::If(if_expr) => self.compile_if(if_expr, true),
            Expression::Block(stmts, loc) => self.compile_block(stmts, *loc, true),
            Expression::Unary(unary) => {
                if unary.operator == Operator::Not {
                    self.compile_expression(&unary.expr)?;
                    self.emit(Instruction::Not, unary.loc);
                    Ok(())
                } else {
                    Err(TinyCompileError::Unsupported("unary operator".to_string()))
                }
            }
            Expression::MethodCall(_) => Err(TinyCompileError::Unsupported("method call".to_string())),
            Expression::ObjectLiteral(_, _) => Err(TinyCompileError::Unsupported("object literal".to_string())),
            Expression::ArrayLiteral(_, _) => Err(TinyCompileError::Unsupported("array literal".to_string())),
            Expression::GetField { .. } => Err(TinyCompileError::Unsupported("get field".to_string())),
            Expression::Index { .. } => Err(TinyCompileError::Unsupported("index".to_string())),
            Expression::Function(_) => Err(TinyCompileError::Unsupported("function expression".to_string())),
            Expression::Import { .. } => Err(TinyCompileError::Unsupported("import".to_string())),
        }
    }

    fn compile_literal(&mut self, lit: &Literal, loc: Location) -> Result<(), TinyCompileError> {
        match lit {
            Literal::Value(value) => {
                self.emit(Instruction::Push(value.clone()), loc);
                Ok(())
            }
        }
    }

    fn compile_binary(&mut self, bop: &BinaryOperation) -> Result<(), TinyCompileError> {
        self.compile_expression(&bop.left)?;
        self.compile_expression(&bop.right)?;
        let instr = match bop.operator {
            Operator::Add => Instruction::Add,
            Operator::Subtract => Instruction::Subtract,
            Operator::Multiply => Instruction::Multiply,
            Operator::Divide => Instruction::Divide,
            Operator::Mod => Instruction::Modulo,
            Operator::Equals => Instruction::Equal,
            Operator::NotEquals => Instruction::NotEqual,
            Operator::Lt => Instruction::LessThan,
            Operator::LtE => Instruction::LessThanOrEqual,
            Operator::Gt => Instruction::GreaterThan,
            Operator::GtE => Instruction::GreaterThanOrEqual,
            Operator::And => Instruction::And,
            Operator::Or => Instruction::Or,
            Operator::Assign | Operator::Not => {
                return Err(TinyCompileError::Unsupported("binary operator".to_string()));
            }
        };
        self.emit(instr, bop.loc);
        Ok(())
    }

    fn compile_call(&mut self, fc: &FunctionCall) -> Result<(), TinyCompileError> {
        let argc = fc.arguments.len();
        if let Expression::Identifier(name, _) = &*fc.callee {
            let is_local = matches!(self.resolve_variable(name), VarLocation::Local(_));
            if !is_local {
                for arg in &fc.arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(Instruction::Call(name.clone(), argc), fc.loc);
                return Ok(());
            }
        }

        self.compile_expression(&fc.callee)?;
        for arg in &fc.arguments {
            self.compile_expression(arg)?;
        }
        self.emit(Instruction::CallStack(argc), fc.loc);
        Ok(())
    }

    fn compile_if(&mut self, if_expr: &If, as_expression: bool) -> Result<(), TinyCompileError> {
        self.compile_expression(&if_expr.test)?;

        let unique_id = self.unique_id();
        let else_label = format!("tiny_else_{}", unique_id);
        let end_label = format!("tiny_end_{}", unique_id);

        self.emit(Instruction::JumpIfFalse(else_label.clone()), if_expr.loc);
        self.compile_block(&if_expr.body, if_expr.loc, as_expression)?;
        self.emit(Instruction::Jump(end_label.clone()), if_expr.loc);

        self.define_label(else_label);

        if !if_expr.else_body.is_empty() {
            self.compile_block(&if_expr.else_body, if_expr.loc, as_expression)?;
        } else if as_expression {
            self.emit(Instruction::Push(Value::Null), if_expr.loc);
        }

        self.define_label(end_label);
        Ok(())
    }

    fn compile_loop(&mut self, loop_stmt: &Loop) -> Result<(), TinyCompileError> {
        let unique_id = self.unique_id();
        let start_label = format!("tiny_loop_start_{}", unique_id);
        let end_label = format!("tiny_loop_end_{}", unique_id);

        self.define_label(start_label.clone());
        self.loop_stack.push(LoopLabels {
            start: start_label.clone(),
            end: end_label.clone(),
        });

        self.compile_expression(&loop_stmt.test)?;
        self.emit(Instruction::JumpIfFalse(end_label.clone()), loop_stmt.loc);

        self.compile_block(&loop_stmt.body, loop_stmt.loc, false)?;
        self.loop_stack.pop();

        self.emit(Instruction::Jump(start_label), loop_stmt.loc);
        self.define_label(end_label);
        Ok(())
    }

    fn compile_block(
        &mut self,
        stmts: &[Statement],
        loc: Location,
        preserve_top: bool,
    ) -> Result<(), TinyCompileError> {
        let base_locals = self.begin_scope();
        if stmts.is_empty() && preserve_top {
            self.emit(Instruction::Push(Value::Null), loc);
        }

        for (idx, stmt) in stmts.iter().enumerate() {
            let is_last = idx == stmts.len().saturating_sub(1);
            if preserve_top && is_last {
                match stmt {
                    Statement::Expression(expr) => {
                        self.compile_expression(expr)?;
                    }
                    _ => {
                        self.compile_statement(stmt)?;
                        self.emit(Instruction::Push(Value::Null), loc);
                    }
                }
            } else {
                self.compile_statement(stmt)?;
            }
        }

        self.end_scope(base_locals, loc, preserve_top);
        Ok(())
    }

    fn expr_loc(&self, expr: &Expression) -> Location {
        match expr {
            Expression::FunctionCall(fc) => fc.loc,
            Expression::MethodCall(mc) => mc.loc,
            Expression::BinaryOperation(bop) => bop.loc,
            Expression::Literal(_, loc) => *loc,
            Expression::Unary(unary) => unary.loc,
            Expression::Identifier(_, loc) => *loc,
            Expression::Block(_, loc) => *loc,
            Expression::If(if_expr) => if_expr.loc,
            Expression::ObjectLiteral(_, loc) => *loc,
            Expression::ArrayLiteral(_, loc) => *loc,
            Expression::GetField { loc, .. } => *loc,
            Expression::Index { loc, .. } => *loc,
            Expression::Function(func) => func.loc,
            Expression::Import { loc, .. } => *loc,
        }
    }
}
