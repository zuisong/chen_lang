# Chen Lang - AI Assistant Guide

## 项目概述

Chen Lang 是一个用 Rust 实现的简单编程语言解释器，支持基本的编程语言特性。

## 核心架构

### 1. 词法分析器 (`token.rs`)
- 负责将源代码转换为 Token 流
- 支持关键字、标识符、数字、字符串、操作符等
- 特殊支持：浮点数、注释（以 `#` 开头）

### 2. 语法分析器 (`parse.rs`)
- 将 Token 流转换为 AST（抽象语法树）
- 支持表达式、语句、函数定义等语法结构

### 3. 编译器 (`lib.rs`)
- 将 AST 编译为字节码指令
- 支持函数定义、循环、条件语句等
- 实现变量作用域和类型检查

### 4. 虚拟机 (`vm.rs`)
- 执行字节码指令
- 栈式虚拟机，支持函数调用和返回
- 内置函数：`print`, `println`

### 5. 值系统 (`value.rs`)
- 统一的值类型系统
- 支持：整数、浮点数、布尔值、字符串、空值
- 自动类型提升和转换

## 语言特性

### 数据类型
- `int`: 32位整数
- `float`: 32位浮点数
- `bool`: 布尔值
- `string`: 字符串
- `null`: 空值

### 运算符
- 算术：`+`, `-`, `*`, `/`, `%`
- 比较：`==`, `!=`, `<`, `<=`, `>`, `>=`
- 逻辑：`&&`, `||`, `!`
- 字符串连接：`+` (自动类型转换)

### 控制流
- 条件语句：`if`/`else`
- 循环：`for`
- 函数定义：`def function_name() { ... }`
- 函数调用：`function_name()`

### 变量
- 声明：`let variable_name = value`
- 赋值：`variable_name = value`
- 作用域：块级作用域

## 开发指南

### 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定测试套件
cargo test --test value_tests
cargo test --test function_tests
cargo test --test complex_tests

# 运行单元测试
cargo test --lib
```

### 运行示例
```bash
# 运行demo文件
cargo run --bin chen_lang -- run demo_codes/fibonacci.ch
cargo run --bin chen_lang -- run demo_codes/9x9.ch

# 直接运行代码
echo 'let x = 5; let y = 3; print(x + y)' | cargo run --bin chen_lang -- run -
```

### 添加新特性

#### 1. 添加新数据类型
1. 在 `value.rs` 的 `Value` 枚举中添加新变体
2. 实现相应的运算方法
3. 更新 `Display` trait 实现
4. 添加类型检查逻辑

#### 2. 添加新运算符
1. 在 `token.rs` 中添加 Token 定义
2. 在 `parse.rs` 中添加解析逻辑
3. 在 `lib.rs` 中添加编译逻辑
4. 在 `vm.rs` 中添加执行逻辑
5. 在 `value.rs` 中添加运算实现

#### 3. 添加新语法结构
1. 在 `expression.rs` 中定义 AST 节点
2. 在 `parse.rs` 中添加解析函数
3. 在 `lib.rs` 中添加编译逻辑
4. 在 `vm.rs` 中添加执行指令

### 代码规范

#### 错误处理
- 使用 `anyhow::Result` 进行错误处理
- 运行时错误使用 `RuntimeError` 枚举
- 编译时错误使用 `CompileError` 枚举

#### 测试
- 单元测试放在相应模块的 `tests` 模块中
- 集成测试放在 `tests/` 目录下
- 每个新特性都需要相应的测试覆盖

#### 调试
- 使用 `tracing` 进行日志记录
- 默认日志级别为 `INFO`
- 调试时可设置为 `DEBUG` 或 `TRACE`

## 项目结构

```
chen_lang/
├── src/
│   ├── bin/chen_lang.rs    # CLI入口
│   ├── lib.rs              # 编译器主逻辑
│   ├── token.rs            # 词法分析器
│   ├── parse.rs            # 语法分析器
│   ├── expression.rs       # AST定义
│   ├── value.rs            # 值系统
│   ├── vm.rs               # 虚拟机
│   └── tests/              # 单元测试
├── tests/                  # 集成测试
│   ├── value_tests.rs      # 值系统测试
│   ├── function_tests.rs   # 函数测试
│   ├── complex_tests.rs    # 复杂功能测试
│   └── ...
├── demo_codes/             # 示例代码
└── Cargo.toml
```

## 重要提醒

### 每次修改后必须运行完整测试
```bash
cargo test
```

### 类型安全
- 所有类型转换都是安全的
- 运行时进行类型检查
- 编译时进行基本的类型推断

### 性能考虑
- 栈式虚拟机，执行效率高
- 字符串使用引用计数，避免不必要的复制
- 指令集设计简洁，易于优化

### 扩展性
- 模块化设计，易于添加新特性
- 统一的指令集，便于VM优化
- 类型系统支持扩展新的数据类型

## 常见问题

### Q: 如何添加新的内置函数？
A: 在 `vm.rs` 的 `Call` 指令处理中添加新的匹配分支。

### Q: 如何支持新的运算符优先级？
A: 在 `parse.rs` 的运算符优先级定义中添加新的优先级。

### Q: 如何调试编译过程？
A: 设置 `RUST_LOG=debug` 环境变量查看详细日志。

### Q: 如何添加新的语法糖？
A: 在语法分析阶段将语法糖转换为等价的AST节点。

## 贡献指南

1. Fork 项目
2. 创建特性分支
3. 编写测试
4. 实现功能
5. 确保所有测试通过
6. 提交 Pull Request

---

*此文档由 AI 助手维护，请及时更新以反映最新代码变更。*