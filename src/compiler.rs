use std::collections::HashMap;

use crate::expression::*;
use crate::tokenizer::{Location, Operator};
use crate::vm::{Instruction, Program, Symbol};

// A scope holds the local variables for a block or function.
struct Scope {
    locals: HashMap<String, i32>,
}

enum VarLocation {
    Local(i32),     // Offset from FP
    Upvalue(usize), // Index in closure's upvalue list
    Global(String), // Global variable name
    This,           // 'this' binding
}

impl Scope {
    fn new() -> Self {
        Self { locals: HashMap::new() }
    }
}

struct Compiler<'a> {
    _raw: &'a [char],
    program: Program,
    offset: usize,
    states: Vec<FunctionState>,
}

struct LoopLabels {
    start: String,
    end: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Upvalue {
    index: usize,
    is_local: bool,
}

struct FunctionState {
    scopes: Vec<Scope>,
    locals_count: usize,
    loop_stack: Vec<LoopLabels>,
    upvalues: Vec<Upvalue>,
}

impl FunctionState {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            locals_count: 0,
            loop_stack: Vec::new(),
            upvalues: Vec::new(),
        }
    }

    fn resolve_local(&self, name: &str) -> Option<i32> {
        for scope in self.scopes.iter().rev() {
            if let Some(index) = scope.locals.get(name) {
                return Some(*index);
            }
        }
        None
    }
}

// The main entry point for compilation.
pub fn compile(raw: &[char], ast: Ast) -> Program {
    let mut compiler = Compiler::new(raw, 0);
    compiler.compile_program(ast);
    compiler.program
}

pub fn compile_with_offset(raw: &[char], ast: Ast, offset: usize) -> Program {
    let mut compiler = Compiler::new(raw, offset);
    compiler.compile_program(ast);
    compiler.program
}

impl<'a> Compiler<'a> {
    fn new(raw: &'a [char], offset: usize) -> Self {
        let states = vec![FunctionState::new()];

        Self {
            _raw: raw,
            program: Program::default(),
            states,
            offset,
        }
    }

    fn unique_id(&self) -> usize {
        self.offset + self.program.instructions.len()
    }

    fn current_state(&mut self) -> &mut FunctionState {
        self.states.last_mut().expect("Compiler state stack empty")
    }

    // --- Scope Management ---

    fn begin_scope(&mut self) {
        self.current_state().scopes.push(Scope::new());
    }

    fn end_scope(&mut self, loc: Location, preserve_top: bool) {
        let (count, first_idx) = {
            let state = self.current_state();
            let scope = state.scopes.pop().expect("No scope to end");
            let c = scope.locals.len();
            let first = state.locals_count - c;
            state.locals_count -= c;
            (c, first)
        };

        if count > 0 {
            if preserve_top {
                self.emit(Instruction::CloseUpvaluesAbove(first_idx), loc);
                self.emit(Instruction::MovePlusFP(first_idx), loc);
                for _ in 0..count - 1 {
                    self.emit(Instruction::Pop, loc);
                }
            } else {
                for _ in 0..count {
                    self.emit(Instruction::CloseUpvalue, loc);
                }
            }
        }
    }

    fn define_variable(&mut self, name: String) -> VarLocation {
        let is_global = self.states.len() == 1 && self.states[0].scopes.len() == 1;

        if is_global {
            VarLocation::Global(name)
        } else {
            let state = self.current_state();
            let scope = state.scopes.last_mut().unwrap();
            let index = state.locals_count as i32;
            scope.locals.insert(name, index);
            state.locals_count += 1;
            VarLocation::Local(index)
        }
    }

    fn resolve_upvalue(&mut self, state_idx: usize, name: &str) -> Option<usize> {
        if state_idx == 0 {
            return None;
        }

        let enclosing_idx = state_idx - 1;

        if let Some(local_idx) = self.states[enclosing_idx].resolve_local(name) {
            return Some(self.add_upvalue(state_idx, local_idx as usize, true));
        }

        if let Some(up_idx) = self.resolve_upvalue(enclosing_idx, name) {
            return Some(self.add_upvalue(state_idx, up_idx, false));
        }

        None
    }

    fn add_upvalue(&mut self, state_idx: usize, index: usize, is_local: bool) -> usize {
        let state = &mut self.states[state_idx];
        for (i, up) in state.upvalues.iter().enumerate() {
            if up.index == index && up.is_local == is_local {
                return i;
            }
        }
        state.upvalues.push(Upvalue { index, is_local });
        state.upvalues.len() - 1
    }

    fn resolve_variable(&mut self, name: &str) -> Option<VarLocation> {
        let state_idx = self.states.len() - 1;

        if let Some(index) = self.states[state_idx].resolve_local(name) {
            return Some(VarLocation::Local(index));
        }

        if let Some(up_idx) = self.resolve_upvalue(state_idx, name) {
            return Some(VarLocation::Upvalue(up_idx));
        }

        if name == "this" {
            return Some(VarLocation::This);
        }

        Some(VarLocation::Global(name.to_string()))
    }

    fn emit(&mut self, instr: Instruction, loc: Location) {
        let idx = self.program.instructions.len();
        self.program.instructions.push(instr);
        self.program.lines.insert(idx, loc);
    }

    fn loc_from_line(line: u32) -> Location {
        Location { line, col: 1, index: 0 }
    }

    // --- Compilation Methods ---

    fn compile_program(&mut self, ast: Ast) {
        let mut function_declarations = Vec::new();
        let mut main_statements = Vec::new();

        for stmt in ast {
            if let Statement::FunctionDeclaration(fd) = stmt {
                function_declarations.push(fd);
            } else {
                main_statements.push(stmt);
            }
        }

        for stmt in main_statements {
            self.compile_statement(stmt);
        }

        if !function_declarations.is_empty() {
            let end_label = "program_end".to_string();
            self.emit(Instruction::Jump(end_label.clone()), Self::loc_from_line(0));

            for fd in function_declarations {
                self.compile_function_def(fd);
            }

            self.program.syms.insert(
                end_label,
                Symbol {
                    location: self.program.instructions.len() as i32,
                    narguments: 0,
                    nlocals: 0,
                    upvalues: Vec::new(),
                },
            );
        }
    }

    fn compile_statement(&mut self, stmt: Statement) {
        match stmt {
            Statement::FunctionDeclaration(fd) => self.compile_function_def(fd),
            Statement::Return(r) => self.compile_return(r),
            Statement::Local(loc) => self.compile_local(loc),
            Statement::Expression(e) => {
                let loc = self.get_expression_location(&e);
                self.compile_expression(e);
                self.emit(Instruction::Pop, loc);
            }
            Statement::Loop(e) => self.compile_loop(e),
            Statement::ForIn(e) => self.compile_for_in(e),
            Statement::Assign(e) => self.compile_assign(e),
            Statement::Break(loc) => {
                let end_label = self
                    .current_state()
                    .loop_stack
                    .last()
                    .expect("break outside of loop")
                    .end
                    .clone();
                self.emit(Instruction::Jump(end_label), loc);
            }
            Statement::Continue(loc) => {
                let start_label = self
                    .current_state()
                    .loop_stack
                    .last()
                    .expect("continue outside of loop")
                    .start
                    .clone();
                self.emit(Instruction::Jump(start_label), loc);
            }
            Statement::SetField {
                object,
                field,
                value,
                loc,
            } => {
                self.compile_expression(object);
                self.compile_expression(value);
                self.emit(Instruction::SetField(field), loc);
            }
            Statement::SetIndex {
                object,
                index,
                value,
                loc,
            } => {
                self.compile_expression(object);
                self.compile_expression(index);
                self.compile_expression(value);
                self.emit(Instruction::SetIndex, loc);
            }
            Statement::TryCatch(tc) => self.compile_try_catch(tc),
            Statement::Throw { value, loc } => {
                self.compile_expression(value);
                self.emit(Instruction::Throw, loc);
            }
        }
    }

    fn get_expression_location(&self, expr: &Expression) -> Location {
        match expr {
            Expression::FunctionCall(fc) => fc.loc,
            Expression::BinaryOperation(bin) => bin.loc,
            Expression::Literal(_, loc) => *loc,
            Expression::Unary(u) => u.loc,
            Expression::Identifier(_, loc) => *loc,
            Expression::Block(_, loc) => *loc,
            Expression::If(if_expr) => if_expr.loc,
            Expression::ObjectLiteral(_, loc) => *loc,
            Expression::ArrayLiteral(_, loc) => *loc,
            Expression::GetField { loc, .. } => *loc,
            Expression::Index { loc, .. } => *loc,
            Expression::Function(fd) => fd.loc,
            Expression::MethodCall(mc) => mc.loc,
        }
    }

    fn compile_function_def(&mut self, fd: FunctionDeclaration) {
        let loc = fd.loc;
        let func_name = fd.name.clone().expect("Statement function must have a name");

        let unique_id = self.unique_id();
        let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

        self.emit(Instruction::Jump(skip_label.clone()), loc);

        self.compile_declaration(fd);

        self.program.syms.insert(
            skip_label,
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        let var_location = self.define_variable(func_name.clone());
        self.emit(Instruction::Closure(format!("func_{}", func_name)), loc);
        match var_location {
            VarLocation::Local(offset) => self.emit(Instruction::MovePlusFP(offset as usize), loc),
            VarLocation::Global(name) => self.emit(Instruction::Store(name), loc),
            _ => panic!("Cannot define function in Upvalue or This location"),
        }
    }

    fn compile_expression(&mut self, exp: Expression) {
        match exp {
            Expression::BinaryOperation(bop) => self.compile_binary_operation(bop),
            Expression::FunctionCall(fc) => self.compile_function_call(fc),
            Expression::MethodCall(mc) => self.compile_method_call(mc),
            Expression::Literal(lit, loc) => self.compile_literal(lit, loc),
            Expression::Identifier(ident, loc) => {
                if let Some(var_location) = self.resolve_variable(&ident) {
                    match var_location {
                        VarLocation::Local(offset) => {
                            self.emit(Instruction::DupPlusFP(offset), loc);
                        }
                        VarLocation::Upvalue(index) => {
                            self.emit(Instruction::GetUpvalue(index), loc);
                        }
                        VarLocation::Global(name) => {
                            self.emit(Instruction::Load(name), loc);
                        }
                        VarLocation::This => {
                            self.emit(Instruction::LoadThis, loc);
                        }
                    }
                } else {
                    self.emit(Instruction::Load(ident), loc);
                }
            }
            Expression::Unary(unary) => {
                let loc = unary.loc;
                self.compile_expression(*unary.expr);
                match unary.operator {
                    Operator::Not => self.emit(Instruction::Not, loc),
                    Operator::Subtract => self.emit(Instruction::Neg, loc),
                    _ => panic!("Unsupported unary operator: {:?}", unary.operator),
                }
            }
            Expression::Block(stmts, loc) => self.compile_block_expression(stmts, loc),
            Expression::If(if_expr) => self.compile_if(if_expr),
            Expression::ObjectLiteral(fields, loc) => {
                self.emit(Instruction::NewObject, loc);
                for (key, val) in fields {
                    self.emit(Instruction::Dup, loc);
                    self.compile_expression(val);
                    self.emit(Instruction::SetField(key), loc);
                }
            }
            Expression::ArrayLiteral(elements, loc) => {
                let count = elements.len();
                for elem in elements {
                    self.compile_expression(elem);
                }
                self.emit(Instruction::BuildArray(count), loc);
            }
            Expression::GetField { object, field, loc } => {
                self.compile_expression(*object);
                self.emit(Instruction::GetField(field), loc);
            }
            Expression::Index { object, index, loc } => {
                self.compile_expression(*object);
                self.compile_expression(*index);
                self.emit(Instruction::GetIndex, loc);
            }
            Expression::Function(mut fd) => {
                let loc = fd.loc;
                let func_name = fd.name.take().unwrap_or_else(|| format!("anon_{}", self.unique_id()));
                fd.name = Some(func_name.clone());

                let unique_id = self.unique_id();
                let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

                self.emit(Instruction::Jump(skip_label.clone()), loc);
                self.compile_declaration(fd);

                self.program.syms.insert(
                    skip_label,
                    Symbol {
                        location: self.program.instructions.len() as i32,
                        narguments: 0,
                        nlocals: 0,
                        upvalues: Vec::new(),
                    },
                );

                self.emit(Instruction::Closure(format!("func_{}", func_name)), loc);
            }
        }
    }

    fn compile_block_expression(&mut self, stmts: Vec<Statement>, loc: Location) {
        self.begin_scope();
        let len = stmts.len();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if i == len - 1 {
                match stmt {
                    Statement::Expression(e) => self.compile_expression(e),
                    _ => {
                        self.compile_statement(stmt);
                        self.emit(Instruction::Push(crate::value::Value::Null), loc);
                    }
                }
            } else {
                self.compile_statement(stmt);
            }
        }
        if len == 0 {
            self.emit(Instruction::Push(crate::value::Value::Null), loc);
        }
        self.end_scope(loc, true);
    }

    fn compile_literal(&mut self, lit: Literal, loc: Location) {
        match lit {
            Literal::Value(val) => {
                self.emit(Instruction::Push(val), loc);
            }
        }
    }

    fn compile_local(&mut self, local: Local) {
        let loc = local.loc;
        self.compile_expression(local.expression);
        let var_location = self.define_variable(local.name);
        match var_location {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), loc);
            }
            _ => panic!("Cannot define local variable"),
        }
    }

    fn compile_assign(&mut self, assign: Assign) {
        let loc = assign.loc;
        self.compile_expression(*assign.expr);
        let var_location = self.resolve_variable(&assign.name).expect("Undefined variable");

        match var_location {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), loc);
            }
            VarLocation::Upvalue(index) => {
                self.emit(Instruction::SetUpvalue(index), loc);
            }
            VarLocation::This => panic!("Cannot assign to 'this'"),
        }
    }

    fn compile_binary_operation(&mut self, bop: BinaryOperation) {
        let loc = bop.loc;
        self.compile_expression(*bop.left);
        self.compile_expression(*bop.right);
        let instruction = match bop.operator {
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
            Operator::Assign | Operator::Not => panic!("Unable to compile binary operation"),
        };
        self.emit(instruction, loc);
    }

    fn compile_function_call(&mut self, fc: FunctionCall) {
        let loc = fc.loc;
        let len = fc.arguments.len();
        let arguments = fc.arguments;
        let callee = *fc.callee;

        if let Expression::GetField { object, field, .. } = callee {
            self.compile_expression(*object);
            self.emit(Instruction::GetMethod(field), loc);
            for arg in arguments {
                self.compile_expression(arg);
            }
            self.emit(Instruction::CallMethodStack(len), loc);
            return;
        }

        {
            let is_optimized_call = if let Expression::Identifier(ref name, _) = callee {
                match self.resolve_variable(name) {
                    Some(VarLocation::Local(_)) | Some(VarLocation::Upvalue(_)) => false,
                    _ => true,
                }
            } else {
                false
            };

            if is_optimized_call {
                if let Expression::Identifier(name, _) = callee {
                    for arg in arguments {
                        self.compile_expression(arg);
                    }
                    self.emit(Instruction::Call(name, len), loc);
                } else {
                    unreachable!();
                }
            } else {
                self.compile_expression(callee);
                for arg in arguments {
                    self.compile_expression(arg);
                }
                self.emit(Instruction::CallStack(len), loc);
            }
        }
    }

    fn compile_method_call(&mut self, mc: MethodCall) {
        let loc = mc.loc;
        self.compile_expression(*mc.object);
        self.emit(Instruction::GetMethod(mc.method), loc);

        let len = mc.arguments.len();
        for arg in mc.arguments {
            self.compile_expression(arg);
        }

        self.emit(Instruction::CallMethodStack(len), loc);
    }

    fn compile_return(&mut self, ret: Return) {
        let loc = ret.loc;
        self.compile_expression(ret.expression);
        self.emit(Instruction::Return, loc);
    }

    fn compile_declaration(&mut self, fd: FunctionDeclaration) {
        let loc = fd.loc;
        let function_index = self.program.instructions.len() as i32;
        let narguments = fd.parameters.len();

        self.states.push(FunctionState::new());

        for param in fd.parameters {
            self.define_variable(param.name);
        }

        let len = fd.body.len();
        if len > 0 {
            for (i, stmt) in fd.body.into_iter().enumerate() {
                if i == len - 1 {
                    match stmt {
                        Statement::Expression(expr) => {
                            self.compile_expression(expr);
                        }
                        _ => {
                            self.compile_statement(stmt);
                            self.emit(Instruction::Push(crate::value::Value::Null), loc);
                        }
                    }
                } else {
                    self.compile_statement(stmt);
                }
            }
        } else {
            self.emit(Instruction::Push(crate::value::Value::Null), loc);
        }

        self.emit(Instruction::Return, loc);
        self.emit(Instruction::Return, loc);

        let state = self.states.pop().expect("Popped global state");
        let nlocals = state.locals_count;
        let upvalues: Vec<(bool, usize)> = state.upvalues.into_iter().map(|u| (u.is_local, u.index)).collect();

        self.program.syms.insert(
            format!("func_{}", fd.name.as_ref().expect("Function must have a name")),
            Symbol {
                location: function_index,
                nlocals,
                narguments,
                upvalues,
            },
        );
    }

    fn compile_if(&mut self, if_stmt: If) {
        let loc = if_stmt.loc;
        self.compile_expression(*if_stmt.test);

        let unique_id = self.unique_id();
        let else_label = format!("else_{}", unique_id);
        let end_label = format!("end_{}", unique_id);

        self.emit(Instruction::JumpIfFalse(else_label.clone()), loc);

        self.compile_block_expression(if_stmt.body, loc);

        self.emit(Instruction::Jump(end_label.clone()), loc);

        self.program.syms.insert(
            else_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                nlocals: 0,
                narguments: 0,
                upvalues: Vec::new(),
            },
        );

        if !if_stmt.else_body.is_empty() {
            self.compile_block_expression(if_stmt.else_body, loc);
        } else {
            self.emit(Instruction::Push(crate::value::Value::Null), loc);
        }

        self.program.syms.insert(
            end_label,
            Symbol {
                location: self.program.instructions.len() as i32,
                nlocals: 0,
                narguments: 0,
                upvalues: Vec::new(),
            },
        );
    }

    fn compile_loop(&mut self, loop_: Loop) {
        let loc = loop_.loc;
        let unique_id = self.unique_id();
        let loop_start = format!("loop_start_{}", unique_id);
        let loop_end = format!("loop_end_{}", unique_id);

        self.program.syms.insert(
            loop_start.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        self.current_state().loop_stack.push(LoopLabels {
            start: loop_start.clone(),
            end: loop_end.clone(),
        });

        self.compile_expression(loop_.test);
        self.emit(Instruction::JumpIfFalse(loop_end.clone()), loc);

        self.begin_scope();
        for stmt in loop_.body {
            self.compile_statement(stmt);
        }
        self.end_scope(loc, false);
        self.current_state().loop_stack.pop();

        self.emit(Instruction::Jump(loop_start.clone()), loc);

        self.program.syms.insert(
            loop_end.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );
    }

    fn compile_for_in(&mut self, for_in: ForInLoop) {
        let loc = for_in.loc;
        let unique_id = self.unique_id();
        let loop_start = format!("for_in_start_{}", unique_id);
        let loop_end = format!("for_in_end_{}", unique_id);
        let iter_var = format!("@iter_{}", unique_id);

        self.begin_scope();

        self.compile_expression(for_in.iterable);
        self.emit(Instruction::GetMethod("iter".to_string()), loc);
        self.emit(Instruction::CallMethodStack(0), loc);
        let iter_loc = self.define_variable(iter_var);
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            _ => unreachable!(),
        }

        self.program.syms.insert(
            loop_start.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        self.emit(Instruction::Load("coroutine".to_string()), loc);
        self.emit(Instruction::GetField("status".to_string()), loc);
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc);
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc);
        self.emit(Instruction::Push(crate::value::Value::string("dead".to_string())), loc);
        self.emit(Instruction::Equal, loc);
        self.emit(Instruction::JumpIfTrue(loop_end.clone()), loc);

        self.emit(Instruction::Load("coroutine".to_string()), loc);
        self.emit(Instruction::GetField("resume".to_string()), loc);
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc);
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc);

        self.emit(Instruction::Load("coroutine".to_string()), loc);
        self.emit(Instruction::GetField("status".to_string()), loc);
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc);
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc);
        self.emit(Instruction::Push(crate::value::Value::string("dead".to_string())), loc);
        self.emit(Instruction::Equal, loc);
        let continue_label = format!("for_in_continue_{}", unique_id);
        self.emit(Instruction::JumpIfFalse(continue_label.clone()), loc);
        self.emit(Instruction::Pop, loc);
        self.emit(Instruction::Jump(loop_end.clone()), loc);

        self.program.syms.insert(
            continue_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        self.begin_scope();
        let var_loc = self.define_variable(for_in.var);
        match var_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), loc);
            }
            _ => unreachable!(),
        }

        self.current_state().loop_stack.push(LoopLabels {
            start: loop_start.clone(),
            end: loop_end.clone(),
        });

        for stmt in for_in.body {
            self.compile_statement(stmt);
        }

        self.end_scope(loc, false);
        self.current_state().loop_stack.pop();

        self.emit(Instruction::Jump(loop_start.clone()), loc);

        self.program.syms.insert(
            loop_end.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );
        self.end_scope(loc, false);
    }

    fn compile_try_catch(&mut self, tc: TryCatch) {
        let loc = tc.loc;
        let unique_id = self.unique_id();
        let catch_label = format!("catch_{}", unique_id);
        let finally_label = format!("finally_{}", unique_id);
        let end_label = format!("end_try_{}", unique_id);

        self.emit(Instruction::PushExceptionHandler(catch_label.clone()), loc);

        self.begin_scope();
        for stmt in tc.try_body {
            self.compile_statement(stmt);
        }
        self.end_scope(loc, false);

        self.emit(Instruction::PopExceptionHandler, loc);

        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), loc);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), loc);
        }

        self.program.syms.insert(
            catch_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        self.begin_scope();

        if let Some(error_name) = tc.error_name {
            let var_location = self.define_variable(error_name);
            match var_location {
                VarLocation::Local(offset) => {
                    self.emit(Instruction::MovePlusFP(offset as usize), loc);
                }
                VarLocation::Global(name) => {
                    self.emit(Instruction::Store(name), loc);
                }
                _ => panic!("Cannot define error variable"),
            }
        } else {
            self.emit(Instruction::Pop, loc);
        }

        for stmt in tc.catch_body {
            self.compile_statement(stmt);
        }

        self.end_scope(loc, false);

        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), loc);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), loc);
        }

        if let Some(finally_body) = tc.finally_body {
            self.program.syms.insert(
                finally_label.clone(),
                Symbol {
                    location: self.program.instructions.len() as i32,
                    narguments: 0,
                    nlocals: 0,
                    upvalues: Vec::new(),
                },
            );

            self.begin_scope();
            for stmt in finally_body {
                self.compile_statement(stmt);
            }
            self.end_scope(loc, false);
        }

        self.program.syms.insert(
            end_label,
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );
    }
}
