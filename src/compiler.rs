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
        let scopes = vec![Scope::new()];

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
            Statement::FunctionDeclaration(fd) => {
                let func_name = fd
                    .name
                    .clone()
                    .expect("Statement function must have a name");
                let unique_id = self.program.instructions.len();
                let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

                // 1. Jump over the function definition
                self.program
                    .instructions
                    .push(Instruction::Jump(skip_label.clone()));

                // 2. Compile the function body
                self.compile_declaration(fd);

                // 3. Define label target
                self.program.syms.insert(
                    skip_label,
                    Symbol {
                        location: self.program.instructions.len() as i32,
                        narguments: 0,
                        nlocals: 0,
                    },
                );

                // 4. Define local variable for the function
                let index = self.define_variable(func_name.clone());
                self.program
                    .instructions
                    .push(Instruction::Push(crate::value::Value::Function(func_name)));
                self.program
                    .instructions
                    .push(Instruction::MovePlusFP(index as usize));
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
            Statement::SetField {
                object,
                field,
                value,
            } => {
                self.compile_expression(object);
                self.compile_expression(value);
                self.program.instructions.push(Instruction::SetField(field));
            }
            Statement::SetIndex {
                object,
                index,
                value,
            } => {
                self.compile_expression(object);
                self.compile_expression(index);
                self.compile_expression(value);
                self.program.instructions.push(Instruction::SetIndex);
            }
        }
    }

    fn compile_expression(&mut self, exp: Expression) {
        match exp {
            Expression::BinaryOperation(bop) => self.compile_binary_operation(bop),
            Expression::FunctionCall(fc) => self.compile_function_call(fc),
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Identifier(ident) => {
                if let Some(offset) = self.resolve_variable(&ident) {
                    self.program
                        .instructions
                        .push(Instruction::DupPlusFP(offset));
                } else {
                    self.program.instructions.push(Instruction::Load(ident));
                }
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
            Expression::ObjectLiteral(fields) => {
                // 创建空对象
                self.program.instructions.push(Instruction::NewObject);
                // 为每个字段设置值
                for (key, val) in fields {
                    self.program.instructions.push(Instruction::Dup); // 复制对象引用
                    self.compile_expression(val); // 编译值
                    self.program.instructions.push(Instruction::SetField(key)); // 设置字段
                }
            }
            Expression::ArrayLiteral(elements) => {
                let count = elements.len();
                for elem in elements {
                    self.compile_expression(elem);
                }
                self.program
                    .instructions
                    .push(Instruction::BuildArray(count));
            }
            Expression::GetField { object, field } => {
                self.compile_expression(*object);
                self.program.instructions.push(Instruction::GetField(field));
            }
            Expression::Index { object, index } => {
                self.compile_expression(*object);
                self.compile_expression(*index);
                self.program.instructions.push(Instruction::GetIndex);
            }
            Expression::Function(mut fd) => {
                let func_name = fd
                    .name
                    .take()
                    .unwrap_or_else(|| format!("anon_{}", self.program.instructions.len()));
                fd.name = Some(func_name.clone());

                let unique_id = self.program.instructions.len();
                let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

                self.program
                    .instructions
                    .push(Instruction::Jump(skip_label.clone()));
                self.compile_declaration(fd);

                self.program.syms.insert(
                    skip_label,
                    Symbol {
                        location: self.program.instructions.len() as i32,
                        narguments: 0,
                        nlocals: 0,
                    },
                );

                self.program
                    .instructions
                    .push(Instruction::Push(crate::value::Value::Function(func_name)));
            }
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
        let arguments = fc.arguments;
        let callee = *fc.callee;

        match callee {
            Expression::GetField { object, field } => {
                // Method Call Optimization: obj.method(...) -> GetMethod -> CallStack(len+1)
                self.compile_expression(*object);
                self.program
                    .instructions
                    .push(Instruction::GetMethod(field));

                for arg in arguments {
                    self.compile_expression(arg);
                }

                // +1 argument for 'self'
                self.program
                    .instructions
                    .push(Instruction::CallStack(len + 1));
            }
            other_callee => {
                // Standard function call

                // Check if we can optimize to direct Call (Identifier referring to Global/Builtin)
                let is_optimized_call = if let Expression::Identifier(ref name) = other_callee {
                    self.resolve_variable(name).is_none()
                } else {
                    false
                };

                if is_optimized_call {
                    if let Expression::Identifier(name) = other_callee {
                        for arg in arguments {
                            self.compile_expression(arg);
                        }
                        self.program.instructions.push(Instruction::Call(name, len));
                    } else {
                        unreachable!("Logic error: is_optimized_call implies Identifier");
                    }
                } else {
                    self.compile_expression(other_callee);
                    for arg in arguments {
                        self.compile_expression(arg);
                    }
                    self.program.instructions.push(Instruction::CallStack(len));
                }
            }
        }
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

        self.program.instructions.push(Instruction::Return);

        self.program.syms.insert(
            format!(
                "func_{}",
                fd.name.as_ref().expect("Function must have a name")
            ),
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
