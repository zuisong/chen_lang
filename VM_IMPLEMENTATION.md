# VM 字节码编译实现总结

## 完成的功能

### 1. VM 指令集扩展
- **栈操作**: `Push`, `Pop`, `Dup`, `DupPlusFP`, `MovePlusFP`
- **变量操作**: `Store` (全局), `Load` (全局/内置)
- **控制流**: `Jump`, `JumpIfFalse`, `JumpIfTrue`, `Call` (直接调用), `CallStack` (栈上函数调用), `Return`
- **算术运算**: `Add`, `Subtract`, `Multiply`, `Divide`, `Modulo` (支持元方法重载)
- **比较运算**: `Equal`, `NotEqual`, `LessThan`, `LessThanOrEqual`, `GreaterThan`, `GreaterThanOrEqual`
- **逻辑运算**: `And`, `Or`, `Not`
- **对象操作**: 
  - `NewObject`: 创建新对象
  - `SetField`, `GetField`: 字段读写 (支持 `__index` 原型链)
  - `GetMethod`: 方法调用优化
  - `SetIndex`, `GetIndex`: 索引读写
- **数组操作**: `BuildArray`

### 2. 编译器实现
- **AST 转字节码**: 完整支持表达式、语句、控制流。
- **作用域管理**: 
  - 正确区分 **局部变量** (栈偏移, `MovePlusFP`/`DupPlusFP`) 和 **全局变量** (名称查找, `Store`/`Load`)。
  - 修复了之前将全局变量错误识别为局部变量的 bug。
- **函数调用优化**:
  - 对于局部变量函数，使用 `Load` + `CallStack`。
  - 对于全局/内置函数（如 `set_meta`, `print`），使用 `Call` 指令，由 VM 在运行时解析。

### 3. 对象与元系统 (Object System)
- **Table 结构**: 统一的数据结构 `Value::Object` (基于 `Rc<RefCell<Table>>`)。
- **Metatable (元表)**: 支持通过 `set_meta` 为对象设置元表。
- **原型继承**: `GetField` 操作会自动查找元表中的 `__index`。
- **运算符重载**:
  - `Add`, `Subtract`, `Multiply` 指令已更新。
  - 当操作数是对象且具有对应元方法 (`__add`, `__sub`, `__mul`) 时，VM 会自动发起函数调用。
  - 修复了元方法调用返回后指令流控制的 bug (避免跳过下一条指令)。

### 4. 集成与测试
- **主流程**: `lib.rs` 的 `run` 函数集成了 VM 执行。
- **测试覆盖**:
  - 单元测试覆盖了基础运算。
  - 集成测试 (`tests/`) 覆盖了函数、闭包、对象、继承、运算符重载等复杂场景。
  - 新增 `demo_codes/point_objects.ch` 演示了完整的对象系统用法。

### 5. 错误报告 (Error Reporting)
- **Source Mapping**: 实现了源代码行号映射。
- **运行时错误**: 当 VM 抛出错误时，现在会报告具体的源代码行号（例如 `Runtime error at line 4: UndefinedVariable("x")`）。
- **AST增强**: 修改了 AST 节点以携带行号信息。

## 运行示例

### 对象与运算符重载
```bash
# 运行演示代码
cargo run --bin chen_lang -- run demo_codes/point_objects.ch
```

### 基础运算
```bash
echo 'let i = 1
let j = 2
print(i + j)' | cargo run --bin chen_lang -- run -
# 输出: 3
```

## 技术架构

### 编译流程
1. **词法分析**: 源代码 → Token 流
2. **语法分析**: Token 流 → AST
3. **编译器**: AST → `Program` (包含指令集和符号表)
   - *Fix*: 增强了 `resolve_variable` 以正确处理作用域层级。
4. **VM 执行**: `Program` → 运行结果
   - *Feat*: `call_value` 支持 NativeFunction 和用户定义 Function 的统一调用接口。

### VM 架构
- **Stack-based**: 基于栈的虚拟机。
- **Frames**: 使用 `CallStack` 和 `Return` 管理调用栈帧 (`fp` 指针)。
- **Global Store**: `variables` HashMap 存储全局变量。
- **Native Interop**: 支持 Rust 原生函数注册到 VM 全局变量中 (如 `print`, `set_meta`)。

## 后续改进方向

1. **更多元方法**: 支持 `__tostring` (用于 `print`), `__eq` 等。
3. **性能优化**: 
   - 引入常量池。
   - 优化 `GetField` 的字符串查找 (String interning)。
4. **垃圾回收**: 目前依赖 Rust `Rc` (引用计数)，可能存在循环引用导致内存泄漏的问题 (例如对象原型环)。需要引入简单的 GC 或循环检测。
