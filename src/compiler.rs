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
}

impl Scope {
    fn new() -> Self {
        Self { locals: HashMap::new() }
    }
}
///
///
///
///
/// Compiler 结构体的核心工作是将抽象语法树（AST）递归遍历并转换为线性的字节码指令序列（Program）。
///
/// 以下是它如何利用各个字段将 AST 编译为指令的详细解释：
///
/// 1. 结构体字段的作用
///
///   * `program: Program`:
///       * 编译器的输出缓冲区。所有 Instruction（如 Push, Add, Jump）都会通过 emit 写入
///         `program.instructions`。
///       * `program.syms` 是符号表，用于记录函数入口地址和跳转标签的位置。
///       * `program.lines` 记录指令索引到源码位置的映射，便于运行时报错定位。
///   * `states: Vec<FunctionState>`:
///       * 函数编译状态栈。进入函数时 push，退出函数时 pop。
///       * 每个 FunctionState 代表一个函数边界，包含块级作用域、局部变量计数、循环栈、闭包 upvalue。
///   * `offset: usize`:
///       * 用于生成唯一 ID 的基数（配合 unique_id() 方法），防止不同编译单元生成的标签名冲突。
///   * `_raw: &[char]`:
///       * 当前源码字符切片，占位保留，便于未来与源码位置相关的优化或诊断。
///
/// FunctionState 里的关键字段：
///   * `scopes: Vec<Scope>`:
///       * 变量作用域栈。进入代码块（如 if/loop/函数体）时 begin_scope 推入 Scope。
///       * Scope 内部维护 `HashMap<String, i32>`，把变量名映射到当前栈帧的偏移量（FP + offset）。
///       * 变量解析顺序：本函数的内层 Scope -> 外层 Scope -> Upvalue -> Global。
///   * `locals_count: usize`:
///       * 记录当前函数内局部变量数量。每次 `let` 增加 1。
///       * 编译完成后写入 `Symbol.nlocals`，VM 调用函数时按此预分配栈空间。
///   * `loop_stack: Vec<LoopLabels>`:
///       * 处理 break / continue。循环开始时压入 (start, end)，退出时弹出。
///   * `upvalues: Vec<Upvalue>`:
///       * 记录闭包捕获的变量（来自外层函数的局部或 upvalue）。
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
///    1. 生成跳过函数体的指令 Jump(skip_label)，避免顺序执行进入函数体。
///    2. 进入新的 FunctionState（函数边界），为参数/局部变量建立新作用域栈。
///    3. 将参数名注册为局部变量（写入 Scope，并增加 locals_count）。
///    4. 编译函数体语句，保证最后有返回值（无显式 return 时自动压入 null）。
///    5. 发射 Return。
///    6. 退出 FunctionState，并把 nlocals/upvalues 写入函数符号表 `func_foo`。
///    7. 插入 skip_label 位置，使 Jump(skip_label) 落到函数定义之后。
///
///   E. 程序入口与函数定义顺序
///    * 编译时会先把“非函数声明语句”作为主逻辑编译到前面。
///    * 若存在函数声明，会插入 `Jump(program_end)` 跳过所有函数体定义。
///    * 所有函数体被编译到指令末尾，并用符号表记录入口地址。
///    * 最后在 `program_end` 处落地，确保主逻辑执行完后结束。
///
///   F. 闭包捕获（Upvalue）编译要点
///    * 变量解析顺序：Local -> Upvalue -> Global。
///    * 当内层函数引用外层变量时，`resolve_upvalue` 会先在外层函数的 local 中查找。
///      - 若找到，则在当前函数的 upvalues 中记录 `(is_local=true, index=local_idx)`。
///      - 若未找到，递归向更外层查找 upvalue，并记录 `(is_local=false, index=upvalue_idx)`。
///    * 编译函数值时发射 `Instruction::Closure(func_name)`，VM 根据符号表里的 upvalues
///      列表，捕获对应的栈槽或上层 upvalue，形成运行时闭包。
///
///   G. Block 表达式返回值规则
///    * Block 是表达式：最后一个语句若是表达式，则其结果作为 Block 的值留在栈顶。
///    * 最后一个语句若不是表达式，会自动 `Push Null` 作为 Block 值。
///    * 空 Block 直接返回 `Null`。
///    * `end_scope(loc, preserve_top=true)` 会关闭 upvalue 并保留栈顶返回值，
///      确保 block 结果不被作用域清理掉。
///
///   总结
///   编译器本质上是一个状态机，它一边遍历 AST，一边维护上下文（Scope, Loop Stack），将树状结构的逻辑“压平”成 VM
///   可以线性执行的指令流。局部变量通过编译时计算的栈偏移量（Stack
///   Offset）来访问，从而避免了运行时的哈希表查找（相比全局变量更快）。
///
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
        // Search scopes from inside out, BUT STOP AT FUNCTION BOUNDARY
        // Note: FunctionState represents ONE function context.
        // Its scopes are blocks within that function.
        // So we search all scopes in this state.
        // The is_function_boundary flag in Scope is actually redundant if we have FunctionState,
        // because FunctionState IS the boundary.
        // But for compatibility with existing logic, let's keep it or simplify.

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
        // Start with one global state.
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

    // Defines a variable in the current scope.
    // Returns its location (offset for local, name for global).
    fn define_variable(&mut self, name: String) -> VarLocation {
        // Check if we are in global scope (bottom of state stack, and bottom of scope stack)
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

        // 1. Check local in enclosing function
        if let Some(local_idx) = self.states[enclosing_idx].resolve_local(name) {
            return Some(self.add_upvalue(state_idx, local_idx as usize, true));
        }

        // 2. Recursive check upvalue in enclosing function
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

    // Resolves a variable: Local -> Upvalue -> Global
    fn resolve_variable(&mut self, name: &str) -> Option<VarLocation> {
        let state_idx = self.states.len() - 1;

        // 1. Local
        if let Some(index) = self.states[state_idx].resolve_local(name) {
            return Some(VarLocation::Local(index));
        }

        // 2. Upvalue
        if let Some(up_idx) = self.resolve_upvalue(state_idx, name) {
            return Some(VarLocation::Upvalue(up_idx));
        }

        // 3. Global
        Some(VarLocation::Global(name.to_string()))
    }

    // --- Helper for emitting instructions with loc numbers ---
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
            // Use 0 as loc number for implicit jumps
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
                // To access the loc number of expression 'e', we need to check its variant.
                // But e is moved into compile_expression.
                // We can't easily extract loc without matching.
                // However, compile_expression handles emission.
                // But we need to Pop the result.
                // We need the loc number for Pop.
                // We can extract loc number via a helper?
                // Or just use 0/approx loc.
                // Let's implement `get_line` for Expression.
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
            // Await removed
            Expression::MethodCall(mc) => mc.loc,
            Expression::Import { loc, .. } => *loc,
        }
    }

    fn compile_function_def(&mut self, fd: FunctionDeclaration) {
        let loc = fd.loc;
        let func_name = fd.name.clone().expect("Statement function must have a name");

        if false { // Async removed
            // removed
        } else {
            // Normal sync function
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
        }

        let var_location = self.define_variable(func_name.clone());
        self.emit(Instruction::Closure(format!("func_{}", func_name)), loc);
        match var_location {
            VarLocation::Local(offset) => self.emit(Instruction::MovePlusFP(offset as usize), loc),
            VarLocation::Global(name) => self.emit(Instruction::Store(name), loc),
            _ => panic!("Cannot define variable in Upvalue location"),
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
                    _ => panic!("Unsupported unary operator"),
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

                // Async removed
                if false {
                } else {
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
            // Await removed
            Expression::Import { path, loc } => {
                self.emit(Instruction::Import(path), loc);
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
                        // Block must return a value
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
            VarLocation::Upvalue(_) => panic!("Cannot define local variable as Upvalue"),
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
            Operator::Assign | Operator::Not => panic!("Unable to compile binary operation: {:?}", bop.operator),
        };
        self.emit(instruction, loc);
    }

    fn compile_function_call(&mut self, fc: FunctionCall) {
        let loc = fc.loc;
        let len = fc.arguments.len();
        let arguments = fc.arguments;
        let other_callee = *fc.callee;

        {
            // Optimized call
            let is_optimized_call = if let Expression::Identifier(ref name, _) = other_callee {
                match self.resolve_variable(name) {
                    Some(VarLocation::Local(_)) | Some(VarLocation::Upvalue(_)) => false,
                    _ => {
                        // Global or not found (function declaration)
                        true
                    }
                }
            } else {
                false
            };

            if is_optimized_call {
                if let Expression::Identifier(name, _) = other_callee {
                    for arg in arguments {
                        self.compile_expression(arg);
                    }
                    self.emit(Instruction::Call(name, len), loc);
                } else {
                    unreachable!();
                }
            } else {
                self.compile_expression(other_callee);
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

        self.emit(Instruction::CallStack(len + 1), loc);
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

        // Push new function state
        self.states.push(FunctionState::new());

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
        self.emit(Instruction::Return, loc); // Safety?

        // Pop state
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

        // 1. Compile iterable, call :iter() on it, and store it in a hidden local variable
        self.compile_expression(for_in.iterable);
        self.emit(Instruction::GetMethod("iter".to_string()), loc); // [iterable, iter_fn, iterable]
        self.emit(Instruction::CallStack(1), loc); // [coroutine]
        let iter_loc = self.define_variable(iter_var);
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            _ => unreachable!("Hidden iterator variable must be local"),
        }

        // loop_start label
        self.program.syms.insert(
            loop_start.clone(),
            Symbol {
                location: self.program.instructions.len() as i32,
                narguments: 0,
                nlocals: 0,
                upvalues: Vec::new(),
            },
        );

        // 2. Check status: coroutine.status(iterable)
        self.emit(Instruction::Load("coroutine".to_string()), loc); // [coroutine]
        self.emit(Instruction::GetField("status".to_string()), loc); // [status_fn]
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc); // [status_fn, iterable]
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc); // [status_val]
        self.emit(Instruction::Push(crate::value::Value::string("dead".to_string())), loc);
        self.emit(Instruction::Equal, loc);
        self.emit(Instruction::JumpIfTrue(loop_end.clone()), loc);

        self.emit(Instruction::Load("coroutine".to_string()), loc); // [resume_fn, iterable, coroutine]
        self.emit(Instruction::GetField("resume".to_string()), loc); // [resume_fn, iterable, resume_fn]
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc); // [resume_fn, iterable, resume_fn, iterable]
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc); // [resume_fn, iterable, yielded_val]

        // 3.5 Check status again after resume - if it just died, the returned value is the final result, not an iteration item.
        self.emit(Instruction::Load("coroutine".to_string()), loc); // [resume_fn, iterable, yielded_val, coroutine]
        self.emit(Instruction::GetField("status".to_string()), loc); // [resume_fn, iterable, yielded_val, status_fn]
        match iter_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::DupPlusFP(offset), loc); // [..., status_fn, iterable]
            }
            _ => unreachable!(),
        }
        self.emit(Instruction::CallStack(1), loc); // [resume_fn, iterable, yielded_val, status_val]
        self.emit(Instruction::Push(crate::value::Value::string("dead".to_string())), loc);
        self.emit(Instruction::Equal, loc);
        let continue_label = format!("for_in_continue_{}", unique_id);
        self.emit(Instruction::JumpIfFalse(continue_label.clone()), loc);
        self.emit(Instruction::Pop, loc); // Pop yielded_val since coroutine is dead
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

        // 4. Define loop variable and assign yielded value
        self.begin_scope();
        let var_loc = self.define_variable(for_in.var);
        match var_loc {
            VarLocation::Local(offset) => {
                self.emit(Instruction::MovePlusFP(offset as usize), loc);
            }
            VarLocation::Global(name) => {
                self.emit(Instruction::Store(name), loc);
            }
            VarLocation::Upvalue(_) => unreachable!(),
        }

        self.current_state().loop_stack.push(LoopLabels {
            start: loop_start.clone(),
            end: loop_end.clone(),
        });

        // 5. Body
        for stmt in for_in.body {
            self.compile_statement(stmt);
        }

        self.end_scope(loc, false);
        self.current_state().loop_stack.pop();

        self.emit(Instruction::Jump(loop_start.clone()), loc);

        // loop_end label
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

        // Set up exception handler
        self.emit(Instruction::PushExceptionHandler(catch_label.clone()), loc);

        // Compile try block
        self.begin_scope();
        for stmt in tc.try_body {
            self.compile_statement(stmt);
        }
        self.end_scope(loc, false);

        // Pop exception handler if no exception occurred
        self.emit(Instruction::PopExceptionHandler, loc);

        // Jump to finally or end
        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), loc);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), loc);
        }

        // Catch block
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

        // Define error variable if provided
        if let Some(error_name) = tc.error_name {
            let var_location = self.define_variable(error_name);
            match var_location {
                VarLocation::Local(offset) => {
                    self.emit(Instruction::MovePlusFP(offset as usize), loc);
                }
                VarLocation::Global(name) => {
                    self.emit(Instruction::Store(name), loc);
                }
                VarLocation::Upvalue(_) => panic!("Cannot define error variable as Upvalue"),
            }
        } else {
            // Pop the error value if no variable to store it
            self.emit(Instruction::Pop, loc);
        }

        // Compile catch block
        for stmt in tc.catch_body {
            self.compile_statement(stmt);
        }

        self.end_scope(loc, false);

        // Jump to finally or end after catch
        if tc.finally_body.is_some() {
            self.emit(Instruction::Jump(finally_label.clone()), loc);
        } else {
            self.emit(Instruction::Jump(end_label.clone()), loc);
        }

        // Finally block (if present)
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

        // End label
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
