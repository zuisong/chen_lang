use std::collections::HashMap;

use crate::expression::*;
use crate::tokenizer::Operator;
use crate::vm::{Instruction, Program, Symbol};

// A scope holds the local variables for a block or function.
struct Scope {
    locals: HashMap<String, i32>,
    is_global_scope: bool, // true for the outermost scope
}

enum VarLocation {
    Local(i32),     // Offset from FP
    Global(String), // Global variable name
}

impl Scope {
    fn new(is_global: bool) -> Self {
        Self {
            locals: HashMap::new(),
            is_global_scope: is_global,
        }
    }
}
///
///
///
///
/// Compiler 结构体的核心工作是将抽象语法树（AST）递归遍历并转换为线性的字节码指令序列（Program）。
///
///   以下是它如何利用各个字段将 AST 编译为指令的详细解释：
///
///   1. 结构体字段的作用
///
///    * `program: Program`:
///        * 这是编译器的输出缓冲区。所有的 Instruction（如 Push, Add, Jump）都会被 emit 函数推入这个结构体中的
///          instructions 向量。
///        * 它还包含 syms（符号表），用于记录函数入口地址、跳转标签（Label）的位置。
///    * `scopes: Vec<Scope>`:
///        * 这是变量作用域栈。每当进入一个新的代码块（如函数体、if 块、loop 块），编译器就会调用 begin_scope
///          推入一个新的 Scope。
///        * 作用：它决定了变量是局部变量还是全局变量。
///            * Scope 内部维护了一个 HashMap<String, i32>，将变量名映射到当前栈帧的偏移量（Offset）。
///            * 编译器在编译 x = 1 时，会先从内层向外层查找
///              scopes。如果在某层找到，说明是局部变量；如果直到最外层（Global
///              Scope）都没找到，或者是在最外层定义的，就是全局变量。
///    * `locals_count: usize`:
///        * 作用：统计当前正在编译的函数内声明了多少个局部变量。
///        * 编译过程：每当遇到 let x = ...，如果是在函数内，locals_count 就加 1。这个计数值最终会作为 nlocals
///          存入函数的符号表信息中。VM 在调用该函数时，会根据这个值预分配栈空间（stack.resize）。
///    * `loop_stack: Vec<LoopLabels>`:
///        * 作用：处理 break 和 continue。
///        * 编译过程：每当编译一个 loop（或 while/for），编译器会生成两个标签：start_label 和
///          end_label，并将它们推入 loop_stack。
///        * 当遇到 break 时，编译器查看栈顶的 end_label 并生成 Jump(end_label)。
///        * 当遇到 continue 时，编译器查看栈顶的 start_label 并生成 Jump(start_label)。
///    * `offset: usize`:
///        * 这似乎是一个用于生成唯一 ID 的基数（配合 unique_id() 方法），防止不同编译单元生成的标签名冲突（例如
///          label_1, label_2）。
///
///   ---
///
///   2. 编译流程举例
///
///   A. 算术表达式 (1 + 2)
///   编译器调用 compile_expression：
///    1. 递归编译左子树 1 -> 发射 Push(1)。
///    2. 递归编译右子树 2 -> 发射 Push(2)。
///    3. 根据操作符 + -> 发射 Instruction::Add。
///        * VM 执行时：弹出 2，弹出 1，相加，结果 3 入栈。
///
///   B. 变量声明与使用 (let x = 10; print(x))
///    1. 声明 (`let x = 10`):
///        * 编译右值 10 -> 发射 Push(10)。
///        * 调用 define_variable("x")。
///        * 如果是局部变量：scopes 记录 x -> offset (比如 0)。发射 MovePlusFP(0)（把栈顶的 10 移动到 FP+0
///          的位置）。
///        * 如果是全局变量：发射 Store("x")。
///    2. 使用 (`print(x)`):
///        * 编译参数 x -> 调用 resolve_variable("x")。
///        * 局部：查表得到 offset，发射 DupPlusFP(offset)（把 FP+0 的值复制到栈顶）。
///        * 全局：发射 Load("x")。
///        * 发射 Call("print", 1)。
///
///   C. 控制流 (if true { ... } else { ... })
///    1. 编译条件 true -> 发射 Push(true)。
///    2. 生成两个标签：else_label 和 end_label。
///    3. 发射 JumpIfFalse(else_label)（如果条件为假，跳去 else）。
///    4. 编译 If 块的代码。
///    5. 发射 Jump(end_label)（执行完 If 块，跳过 else 块）。
///    6. 插入标签位置：在 program.syms 中记录 else_label 指向当前指令索引。
///    7. 编译 Else 块的代码。
///    8. 插入标签位置：在 program.syms 中记录 end_label 指向当前指令索引。
///
///   D. 函数定义 (def foo() { ... })
///    1. 生成跳过函数体的指令 Jump(skip_label)（防止程序顺序执行时误入函数定义）。
///    2. 记录函数入口：在当前位置记录函数名 func_foo 到符号表。
///    3. 调用 begin_scope()，重置 locals_count = 0。
///    4. 将参数名注册到 Scope 中（作为最早的局部变量）。
///    5. 编译函数体内的所有语句。
///    6. 记录 nlocals（局部变量总数）。
///    7. 发射 Return。
///    8. 调用 end_scope()。
///    9. 插入标签位置：记录 skip_label，让之前的 Jump 能跳到函数定义之后。
///
///   总结
///   编译器本质上是一个状态机，它一边遍历 AST，一边维护上下文（Scope, Loop Stack），将树状结构的逻辑“压平”成 VM
///   可以线性执行的指令流。局部变量通过编译时计算的栈偏移量（Stack
///   Offset）来访问，从而避免了运行时的哈希表查找（相比全局变量更快）。
///
///
struct Compiler<'a> {
    _raw: &'a [char],
    program: Program,
    scopes: Vec<Scope>,
    locals_count: usize, // For current function's local stack frame
    loop_stack: Vec<LoopLabels>,
    offset: usize,
}

struct LoopLabels {
    start: String,
    end: String,
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
        // Start with one global scope.
        let scopes = vec![Scope::new(true)];

        Self {
            _raw: raw,
            program: Program::default(),
            scopes,
            locals_count: 0,
            loop_stack: Vec::new(),
            offset,
        }
    }

    fn unique_id(&self) -> usize {
        self.offset + self.program.instructions.len()
    }

    // --- Scope Management ---

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new(false));
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    // Defines a variable in the current scope.
    // Returns its location (offset for local, name for global).
    fn define_variable(&mut self, name: String) -> VarLocation {
        let current_scope_idx = self.scopes.len() - 1;
        let is_global = self.scopes[current_scope_idx].is_global_scope;

        if is_global {
            VarLocation::Global(name)
        } else {
            let scope = self.scopes.last_mut().unwrap();
            let index = self.locals_count as i32;
            scope.locals.insert(name, index);
            self.locals_count += 1;
            VarLocation::Local(index)
        }
    }

    // Resolves a variable by searching from the innermost to outermost scope.
    fn resolve_variable(&self, name: &str) -> Option<VarLocation> {
        for scope in self.scopes.iter().rev() {
            if let Some(index) = scope.locals.get(name) {
                // If found in a local scope
                return Some(VarLocation::Local(*index));
            }
            if scope.is_global_scope {
                // If reached global scope and not found locally, it's a global variable
                return Some(VarLocation::Global(name.to_string()));
            }
        }
        None // Not found in any scope
    }

    // --- Helper for emitting instructions with line numbers ---
    fn emit(&mut self, instr: Instruction, line: u32) {
        let idx = self.program.instructions.len();
        self.program.instructions.push(instr);
        self.program.lines.insert(idx, line);
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
            // Use 0 as line number for implicit jumps
            self.emit(Instruction::Jump(end_label.clone()), 0);

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
                let line = fd.line;
                let func_name = fd.name.clone().expect("Statement function must have a name");
                let unique_id = self.unique_id();
                let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

                self.emit(Instruction::Jump(skip_label.clone()), line);

                self.compile_declaration(fd);

                self.program.syms.insert(
                    skip_label,
                    Symbol {
                        location: self.program.instructions.len() as i32,
                        narguments: 0,
                        nlocals: 0,
                    },
                );

                let var_location = self.define_variable(func_name.clone());
                self.emit(Instruction::Push(crate::value::Value::Function(func_name)), line);
                match var_location {
                    VarLocation::Local(offset) => self.emit(Instruction::MovePlusFP(offset as usize), line),
                    VarLocation::Global(name) => self.emit(Instruction::Store(name), line),
                }
            }
            Statement::Return(r) => self.compile_return(r),
            Statement::Local(loc) => self.compile_local(loc),
            Statement::Expression(e) => {
                // To access the line number of expression 'e', we need to check its variant.
                // But e is moved into compile_expression.
                // We can't easily extract line without matching.
                // However, compile_expression handles emission.
                // But we need to Pop the result.
                // We need the line number for Pop.
                // We can extract line number via a helper?
                // Or just use 0/approx line.
                // Let's implement `get_line` for Expression.
                let line = self.get_expression_line(&e);
                self.compile_expression(e);
                self.emit(Instruction::Pop, line);
            }
            Statement::Loop(e) => self.compile_loop(e),
            Statement::Assign(e) => self.compile_assign(e),
            Statement::Break(line) => {
                let labels = self.loop_stack.last().expect("break outside of loop");
                self.emit(Instruction::Jump(labels.end.clone()), line);
            }
            Statement::Continue(line) => {
                let labels = self.loop_stack.last().expect("continue outside of loop");
                self.emit(Instruction::Jump(labels.start.clone()), line);
            }
            Statement::SetField {
                object,
                field,
                value,
                line,
            } => {
                self.compile_expression(object);
                self.compile_expression(value);
                self.emit(Instruction::SetField(field), line);
            }
            Statement::SetIndex {
                object,
                index,
                value,
                line,
            } => {
                self.compile_expression(object);
                self.compile_expression(index);
                self.compile_expression(value);
                self.emit(Instruction::SetIndex, line);
            }
            Statement::TryCatch(tc) => self.compile_try_catch(tc),
            Statement::Throw { value, line } => {
                self.compile_expression(value);
                self.emit(Instruction::Throw, line);
            }
        }
    }

    fn get_expression_line(&self, expr: &Expression) -> u32 {
        match expr {
            Expression::FunctionCall(fc) => fc.line,
            Expression::BinaryOperation(bin) => bin.line,
            Expression::Literal(_, line) => *line,
            Expression::Unary(u) => u.line,
            Expression::Identifier(_, line) => *line,
            Expression::Block(_, line) => *line,
            Expression::If(if_expr) => if_expr.line,
            Expression::ObjectLiteral(_, line) => *line,
            Expression::ArrayLiteral(_, line) => *line,
            Expression::GetField { line, .. } => *line,
            Expression::Index { line, .. } => *line,
            Expression::Function(fd) => fd.line,
        }
    }

    fn compile_expression(&mut self, exp: Expression) {
        match exp {
            Expression::BinaryOperation(bop) => self.compile_binary_operation(bop),
            Expression::FunctionCall(fc) => self.compile_function_call(fc),
            Expression::Literal(lit, line) => self.compile_literal(lit, line),
            Expression::Identifier(ident, line) => {
                if let Some(var_location) = self.resolve_variable(&ident) {
                    match var_location {
                        VarLocation::Local(offset) => {
                            self.emit(Instruction::DupPlusFP(offset), line);
                        }
                        VarLocation::Global(name) => {
                            self.emit(Instruction::Load(name), line);
                        }
                    }
                } else {
                    self.emit(Instruction::Load(ident), line);
                }
            }
            Expression::Unary(unary) => {
                let line = unary.line;
                self.compile_expression(*unary.expr);
                match unary.operator {
                    Operator::Not => self.emit(Instruction::Not, line),
                    _ => panic!("Unsupported unary operator"),
                }
            }
            Expression::Block(stmts, line) => self.compile_block_expression(stmts, line),
            Expression::If(if_expr) => self.compile_if(if_expr),
            Expression::ObjectLiteral(fields, line) => {
                self.emit(Instruction::NewObject, line);
                for (key, val) in fields {
                    self.emit(Instruction::Dup, line);
                    self.compile_expression(val);
                    self.emit(Instruction::SetField(key), line);
                }
            }
            Expression::ArrayLiteral(elements, line) => {
                let count = elements.len();
                for elem in elements {
                    self.compile_expression(elem);
                }
                self.emit(Instruction::BuildArray(count), line);
            }
            Expression::GetField { object, field, line } => {
                self.compile_expression(*object);
                self.emit(Instruction::GetField(field), line);
            }
            Expression::Index { object, index, line } => {
                self.compile_expression(*object);
                self.compile_expression(*index);
                self.emit(Instruction::GetIndex, line);
            }
            Expression::Function(mut fd) => {
                let line = fd.line;
                let func_name = fd.name.take().unwrap_or_else(|| format!("anon_{}", self.unique_id()));
                fd.name = Some(func_name.clone());

                let unique_id = self.unique_id();
                let skip_label = format!("skip_func_{}_{}", func_name, unique_id);

                self.emit(Instruction::Jump(skip_label.clone()), line);
                self.compile_declaration(fd);

                self.program.syms.insert(
                    skip_label,
                    Symbol {
                        location: self.program.instructions.len() as i32,
                        narguments: 0,
                        nlocals: 0,
                    },
                );

                self.emit(Instruction::Push(crate::value::Value::Function(func_name)), line);
            }
        }
    }

    fn compile_block_expression(&mut self, stmts: Vec<Statement>, line: u32) {
        self.begin_scope();
        let len = stmts.len();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if i == len - 1 {
                match stmt {
                    Statement::Expression(e) => self.compile_expression(e),
                    _ => {
                        self.compile_statement(stmt);
                        // Block must return a value
                        self.emit(Instruction::Push(crate::value::Value::Null), line);
                    }
                }
            } else {
                self.compile_statement(stmt);
            }
        }
        if len == 0 {
            self.emit(Instruction::Push(crate::value::Value::Null), line);
        }
        self.end_scope();
    }

    fn compile_literal(&mut self, lit: Literal, line: u32) {
        match lit {
            Literal::Value(val) => {
                self.emit(Instruction::Push(val), line);
            }
        }
    }

    fn compile_local(&mut self, local: Local) {
        let line = local.line;
        self.compile_expression(local.expression);
        let var_location = self.define_variable(local.name);
        match var_location {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), line);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), line);
            }
        }
    }

    fn compile_assign(&mut self, assign: Assign) {
        let line = assign.line;
        self.compile_expression(*assign.expr);
        let var_location = self.resolve_variable(&assign.name).expect("Undefined variable");

        match var_location {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), line);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), line);
            }
        }
    }

    fn compile_binary_operation(&mut self, bop: BinaryOperation) {
        let line = bop.line;
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
        self.emit(instruction, line);
    }

    fn compile_function_call(&mut self, fc: FunctionCall) {
        let line = fc.line;
        let len = fc.arguments.len();
        let arguments = fc.arguments;
        let callee = *fc.callee;

        match callee {
            Expression::GetField { object, field, .. } => {
                self.compile_expression(*object);
                self.emit(Instruction::GetMethod(field), line);

                for arg in arguments {
                    self.compile_expression(arg);
                }

                self.emit(Instruction::CallStack(len + 1), line);
            }
            other_callee => {
                // Optimized call
                let is_optimized_call = if let Expression::Identifier(ref name, _) = other_callee {
                    match self.resolve_variable(name) {
                        Some(VarLocation::Local(_)) => false,
                        _ => true,
                    }
                } else {
                    false
                };

                if is_optimized_call {
                    if let Expression::Identifier(name, _) = other_callee {
                        for arg in arguments {
                            self.compile_expression(arg);
                        }
                        self.emit(Instruction::Call(name, len), line);
                    } else {
                        unreachable!();
                    }
                } else {
                    self.compile_expression(other_callee);
                    for arg in arguments {
                        self.compile_expression(arg);
                    }
                    self.emit(Instruction::CallStack(len), line);
                }
            }
        }
    }

    fn compile_return(&mut self, ret: Return) {
        let line = ret.line;
        self.compile_expression(ret.expression);
        self.emit(Instruction::Return, line);
    }

    fn compile_declaration(&mut self, fd: FunctionDeclaration) {
        let line = fd.line;
        self.begin_scope();
        let function_index = self.program.instructions.len() as i32;
        let narguments = fd.parameters.len();

        let old_locals_count = self.locals_count;
        self.locals_count = 0;

        for param in fd.parameters {
            if let VarLocation::Local(_) = self.define_variable(param) {
                // ok
            } else {
                panic!("Function parameter cannot be global");
            }
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
                            self.emit(Instruction::Push(crate::value::Value::Null), line);
                        }
                    }
                } else {
                    self.compile_statement(stmt);
                }
            }
        } else {
            self.emit(Instruction::Push(crate::value::Value::Null), line);
        }

        let nlocals = self.locals_count;
        self.end_scope();
        self.locals_count = old_locals_count;

        self.emit(Instruction::Return, line);
        self.emit(Instruction::Return, line); // Safety?

        self.program.syms.insert(
            format!("func_{}", fd.name.as_ref().expect("Function must have a name")),
            Symbol {
                location: function_index,
                nlocals,
                narguments,
            },
        );
    }

    fn compile_if(&mut self, if_stmt: If) {
        let line = if_stmt.line;
        self.compile_expression(*if_stmt.test);

        let unique_id = self.unique_id();
        let else_label = format!("else_{}", unique_id);
        let end_label = format!("end_{}", unique_id);

        self.emit(Instruction::JumpIfFalse(else_label.clone()), line);

        self.compile_block_expression(if_stmt.body, line);

        self.emit(Instruction::Jump(end_label.clone()), line);

        self.program.syms.insert(
            else_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                nlocals: 0,
                narguments: 0,
            },
        );

        if !if_stmt.else_body.is_empty() {
            self.compile_block_expression(if_stmt.else_body, line);
        } else {
            self.emit(Instruction::Push(crate::value::Value::Null), line);
        }

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
        let line = loop_.line;
        let unique_id = self.unique_id();
        let loop_start = format!("loop_start_{}", unique_id);
        let loop_end = format!("loop_end_{}", unique_id);

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
        self.emit(Instruction::JumpIfFalse(loop_end.clone()), line);

        self.begin_scope();
        for stmt in loop_.body {
            self.compile_statement(stmt);
        }
        self.end_scope();
        self.loop_stack.pop();

        self.emit(Instruction::Jump(loop_start.clone()), line);

        self.program.syms.insert(
            loop_end.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
            },
        );
    }

    fn compile_try_catch(&mut self, tc: TryCatch) {
        let line = tc.line;
        let unique_id = self.unique_id();
        let catch_label = format!("catch_{}", unique_id);
        let finally_label = format!("finally_{}", unique_id);
        let end_label = format!("end_try_{}", unique_id);

        // Set up exception handler
        self.emit(Instruction::PushExceptionHandler(catch_label.clone()), line);

        // Compile try block
        self.begin_scope();
        for stmt in tc.try_body {
            self.compile_statement(stmt);
        }
        self.end_scope();

        // Pop exception handler if no exception occurred
        self.emit(Instruction::PopExceptionHandler, line);

        // Jump to finally or end
        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), line);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), line);
        }

        // Catch block
        self.program.syms.insert(
            catch_label.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
            },
        );

        self.begin_scope();

        // Define error variable if provided
        if let Some(error_name) = tc.error_name {
            let var_location = self.define_variable(error_name);
            match var_location {
                VarLocation::Local(offset) => {
                    self.emit(Instruction::MovePlusFP(offset as usize), line);
                }
                VarLocation::Global(name) => {
                    self.emit(Instruction::Store(name), line);
                }
            }
        } else {
            // Pop the error value if no variable to store it
            self.emit(Instruction::Pop, line);
        }

        // Compile catch block
        for stmt in tc.catch_body {
            self.compile_statement(stmt);
        }

        self.end_scope();

        // Jump to finally or end after catch
        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), line);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), line);
        }

        // Finally block (if present)
        if let Some(finally_body) = tc.finally_body {
            self.program.syms.insert(
                finally_label.clone(),
                Symbol {
                    location: self.program.instructions.len() as i32,
                    narguments: 0,
                    nlocals: 0,
                },
            );

            self.begin_scope();
            for stmt in finally_body {
                self.compile_statement(stmt);
            }
            self.end_scope();
        }

        // End label
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
