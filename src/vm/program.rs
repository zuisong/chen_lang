use indexmap::IndexMap;

use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub location: i32,
    pub narguments: usize,
    pub nlocals: usize,
    pub upvalues: Vec<(bool, usize)>, // (is_local, index)
}

/// 指令集 - 简化后的统一指令
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // 栈操作
    Push(Value), // 推入常量值
    Pop,         // 弹出栈顶
    Dup,         // 复制栈顶元素

    // 变量操作
    Load(String),  // 加载变量（全局变量或函数符号回退）
    Store(String), // 存储到变量（全局变量）

    // 运算操作（统一接口）
    Add,      // 加法
    Subtract, // 减法
    Multiply, // 乘法
    Divide,   // 除法
    Modulo,   // 取模

    // 比较操作
    Equal,              // 等于
    NotEqual,           // 不等于
    LessThan,           // 小于
    LessThanOrEqual,    // 小于等于
    GreaterThan,        // 大于
    GreaterThanOrEqual, // 大于等于

    // 逻辑操作
    And, // 逻辑与
    Or,  // 逻辑或
    Not, // 逻辑非

    // 控制流
    Jump(String),        // 无条件跳转到标签
    JumpIfFalse(String), // 条件跳转：栈顶为假时跳转到标签
    JumpIfTrue(String),  // 条件跳转：栈顶为真时跳转到标签

    // 函数调用
    Call(String, usize), // 调用函数（函数名，参数个数）
    Return,              // 返回：恢复调用帧并把返回值压栈

    // 标签（用于跳转目标）
    Label(String), // 标签定义（当前实现主要用 syms 映射，非必需）

    // New Scope-related Instructions
    DupPlusFP(i32),            // 复制 FP + offset 的值到栈顶（局部变量读取）
    MovePlusFP(usize),         // 将栈顶值写入 FP + offset（局部变量写入）
    GetUpvalue(usize),         // 读取闭包捕获的 upvalue
    SetUpvalue(usize),         // 写入闭包捕获的 upvalue
    Closure(String),           // 根据函数符号名创建闭包并压栈
    CloseUpvalue,              // 关闭栈顶相关的 upvalue（离开作用域）
    CloseUpvaluesAbove(usize), // 关闭所有位于 FP+offset 之上的 upvalue

    // Object operations
    NewObject,         // 创建空对象
    SetField(String),  // 设置对象字段：obj.field = value（弹出 value, obj）
    GetField(String),  // 获取对象字段：obj.field（弹出 obj，压入 value）
    GetMethod(String), // 获取方法：obj.method（弹出 obj，压入 func, obj）- 用于方法调用优化
    SetIndex,          // 设置对象索引：obj[index] = value（弹出 value, index, obj）
    GetIndex,          // 获取对象索引：obj[index]（弹出 index, obj，压入 value）

    // Call function from stack
    CallStack(usize), // 从栈顶调用函数（函数对象在 args 之下）

    // Array creation (Syntactic sugar for object with numeric keys)
    BuildArray(usize), // 从栈顶 n 个元素构建数组对象

    // Exception handling
    Throw,                        // 抛出异常（从栈顶取值）
    Import(String),               // 导入模块（stdlib 或文件）
    PushExceptionHandler(String), // 压入异常处理器（catch label）
    PopExceptionHandler,          // 弹出异常处理器
}

/// 程序表示
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub syms: IndexMap<String, Symbol>, // 符号表
    pub lines: IndexMap<usize, u32>,    // PC -> 行号映射 (Instruction Index -> Line Number)
}

impl Program {
    /// 添加指令
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}
