use std::collections::HashMap;

use crate::expression::*;
use crate::token::Operator;
use crate::vm::{Instruction, Program, Symbol};

// A scope holds the local variables for a block or function.
struct Scope {
    locals: HashMap<String, i32>,
}

impl Scope {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }
}

// The Compiler struct holds the state of the compilation process.
struct Compiler<'a> {
    raw: &'a [char],
    program: Program,
    scopes: Vec<Scope>,
    locals_count: usize,
    loop_stack: Vec<LoopLabels>,
}

struct LoopLabels {
    start: String,
    end: String,
}

// The main entry point for compilation.
pub fn compile(raw: &[char], ast: Ast) -> Program {
    let mut compiler = Compiler::new(raw);
    compiler.compile_program(ast);
    compiler.program
}

impl<'a> Compiler<'a> {
    fn new(raw: &'a [char]) -> Self {
        // Start with one scope for the global-like top-level script.
        let mut scopes = Vec::new();
        scopes.push(Scope::new());

        Self {
            raw,
            program: Program::default(),
            scopes,
            locals_count: 0,
            loop_stack: Vec::new(),
        }
    }

    // --- Scope Management ---

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    // Defines a variable in the current scope.
    fn define_variable(&mut self, name: String) -> i32 {
        let scope = self.scopes.last_mut().unwrap();
        let index = self.locals_count as i32;
        scope.locals.insert(name, index);
        self.locals_count += 1;
        index
    }

    // Resolves a variable by searching from the innermost to outermost scope.
    fn resolve_variable(&self, name: &str) -> Option<i32> {
        for scope in self.scopes.iter().rev() {
            if let Some(index) = scope.locals.get(name) {
                return Some(*index);
            }
        }
        None
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
            self.program
                .instructions
                .push(Instruction::Jump(end_label.clone()));

            for fd in function_declarations {
                self.compile_declaration(fd);
            }

            self.program.syms.insert(
                end_label,
                Symbol {
                    location: self.program.instructions.len() as i32,
                    narguments: 0,
                    nlocals: 0,
                },
            );
        }
    }

    fn compile_statement(&mut self, stmt: Statement) {
        match stmt {
            Statement::FunctionDeclaration(_) => {
                // This is handled in compile_program, so we can ignore it here.
            }
            Statement::Return(r) => self.compile_return(r),
            Statement::Local(loc) => self.compile_local(loc),
            Statement::Expression(e) => {
                self.compile_expression(e);
                self.program.instructions.push(Instruction::Pop);
            }
            Statement::Loop(e) => self.compile_loop(e),
            Statement::Assign(e) => self.compile_assign(e),
            Statement::Break => {
                let labels = self.loop_stack.last().expect("break outside of loop");
                self.program
                    .instructions
                    .push(Instruction::Jump(labels.end.clone()));
            }
            Statement::Continue => {
                let labels = self.loop_stack.last().expect("continue outside of loop");
                self.program
                    .instructions
                    .push(Instruction::Jump(labels.start.clone()));
            }
        }
    }

    fn compile_expression(&mut self, exp: Expression) {
        match exp {
            Expression::BinaryOperation(bop) => self.compile_binary_operation(bop),
            Expression::FunctionCall(fc) => self.compile_function_call(fc),
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Identifier(ident) => {
                let offset = self.resolve_variable(&ident).expect("Undefined variable");
                self.program
                    .instructions
                    .push(Instruction::DupPlusFP(offset));
            }
            Expression::Unary(unary) => {
                self.compile_expression(*unary.expr);
                match unary.operator {
                    Operator::Not => self.program.instructions.push(Instruction::Not),
                    _ => panic!("Unsupported unary operator"),
                }
            }
            Expression::Block(stmts) => self.compile_block_expression(stmts),
            Expression::If(if_expr) => self.compile_if(if_expr),
        }
    }

    fn compile_block_expression(&mut self, stmts: Vec<Statement>) {
        self.begin_scope();
        let len = stmts.len();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if i == len - 1 {
                match stmt {
                    Statement::Expression(e) => self.compile_expression(e),
                    _ => {
                        self.compile_statement(stmt);
                        // Block must return a value, so push Null if the last statement is not an expression
                        self.program
                            .instructions
                            .push(Instruction::Push(crate::value::Value::Null));
                    }
                }
            } else {
                self.compile_statement(stmt);
            }
        }
        if len == 0 {
            self.program
                .instructions
                .push(Instruction::Push(crate::value::Value::Null));
        }
        self.end_scope();
    }

    fn compile_literal(&mut self, lit: Literal) {
        match lit {
            Literal::Value(val) => {
                self.program.instructions.push(Instruction::Push(val));
            }
        }
    }

    fn compile_local(&mut self, local: Local) {
        self.compile_expression(local.expression);
        let index = self.define_variable(local.name);
        self.program
            .instructions
            .push(Instruction::MovePlusFP(index as usize));
    }

    fn compile_assign(&mut self, assign: Assign) {
        self.compile_expression(*assign.expr);
        let offset = self
            .resolve_variable(&assign.name)
            .expect("Undefined variable");
        self.program
            .instructions
            .push(Instruction::MovePlusFP(offset as usize));
    }

    fn compile_binary_operation(&mut self, bop: BinaryOperation) {
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
            _ => panic!("Unable to compile binary operation: {:?}", bop.operator),
        };
        self.program.instructions.push(instruction);
    }

    fn compile_function_call(&mut self, fc: FunctionCall) {
        let len = fc.arguments.len();
        for arg in fc.arguments {
            self.compile_expression(arg);
        }
        self.program
            .instructions
            .push(Instruction::Call(fc.name, len));
    }

    fn compile_return(&mut self, ret: Return) {
        self.compile_expression(ret.expression);
        self.program.instructions.push(Instruction::Return);
    }

    fn compile_declaration(&mut self, fd: FunctionDeclaration) {
        self.begin_scope();
        let function_index = self.program.instructions.len() as i32;
        let narguments = fd.parameters.len();

        let old_locals_count = self.locals_count;
        self.locals_count = 0;

        for param in fd.parameters {
            self.define_variable(param);
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
                            self.program
                                .instructions
                                .push(Instruction::Push(crate::value::Value::Null));
                        }
                    }
                } else {
                    self.compile_statement(stmt);
                }
            }
        } else {
            self.program
                .instructions
                .push(Instruction::Push(crate::value::Value::Null));
        }

        let nlocals = self.locals_count;
        self.end_scope();
        self.locals_count = old_locals_count;

        // Implicit return: return the value on top of the stack
        self.program.instructions.push(Instruction::Return);

        self.program.syms.insert(
            format!("func_{}", fd.name),
            Symbol {
                location: function_index,
                nlocals,
                narguments,
            },
        );
    }

    fn compile_if(&mut self, if_stmt: If) {
        self.compile_expression(*if_stmt.test);

        let unique_id = self.program.instructions.len();
        let else_label = format!("else_{}", unique_id);
        let end_label = format!("end_{}", unique_id);

        self.program
            .instructions
            .push(Instruction::JumpIfFalse(else_label.clone()));

        self.compile_block_expression(if_stmt.body);

        self.program
            .instructions
            .push(Instruction::Jump(end_label.clone()));

        // Else label location
        self.program.syms.insert(
            else_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                nlocals: 0,
                narguments: 0,
            },
        );

        if !if_stmt.else_body.is_empty() {
            self.compile_block_expression(if_stmt.else_body);
        } else {
            // If expression with no else branch must return Null
            self.program
                .instructions
                .push(Instruction::Push(crate::value::Value::Null));
        }

        // End label location
        self.program.syms.insert(
            end_label,
            Symbol {
                location: self.program.instructions.len() as i32,
                nlocals: 0,
                narguments: 0,
            },
        );
    }

    fn compile_loop(&mut self, loop_: Loop) {
        let loop_start = format!("loop_start_{}", self.program.instructions.len());
        let loop_end = format!("loop_end_{}", self.program.instructions.len());

        self.program.syms.insert(
            loop_start.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
            },
        );

        self.loop_stack.push(LoopLabels {
            start: loop_start.clone(),
            end: loop_end.clone(),
        });

        self.compile_expression(loop_.test);
        self.program
            .instructions
            .push(Instruction::JumpIfFalse(loop_end.clone()));

        self.begin_scope();
        for stmt in loop_.body {
            self.compile_statement(stmt);
        }
        self.end_scope();
        self.loop_stack.pop();

        self.program
            .instructions
            .push(Instruction::Jump(loop_start.clone()));

        self.program.syms.insert(
            loop_end.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
            },
        );
    }
}
