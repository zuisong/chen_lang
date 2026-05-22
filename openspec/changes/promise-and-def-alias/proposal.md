## 为什么 (Why)

目前 Chen Lang 的异步执行模型已经从纤程/协程（fiber/coroutine）全面迁移到了基于 Promise 的异步执行模型。然而，Promise 的实例方法（`.then`、`.catch`、`.finally`）和全局静态方法（`Promise.new`、`Promise.all`、`Promise.race`、`Promise.allSettled`）尚未完整定义并实现在运行时中。此外，需要支持将 `def` 关键字作为 `function` 关键字的别名，以符合项目设计指南及用户习惯。

## 变更内容 (What Changes)

- **Promise 实例方法**：在 `Value::Promise` 上实现 `.then`、`.catch` 和 `.finally`。
- **Promise 静态方法**：在全局 `Promise` 对象上实现 `Promise.new`、`Promise.all`、`Promise.race` 和 `Promise.allSettled` 方法。
- **分词器（Tokenizer）更新**：更新 Winnow 分词器与手写分词器，支持将 `def` 关键字映射为 `Token::Keyword(Keyword::FUNCTION)`（即表示函数定义）。
- **Pest 解析器语法更新**：更新 `src/chen.pest` 语法定义文件，使 `FUNCTION` 词法规则同时接受 `def` 和 `function` 关键字。

## 能力需求 (Capabilities)

### 新增能力 (New Capabilities)
- `promise-support`: 实现用于异步操作的 Promise 实例方法和静态方法。
- `def-keyword-alias`: 支持将 `def` 关键字作为 `function` 关键字的直接别名。

### 修改能力 (Modified Capabilities)
<!-- 无 -->

## 影响范围 (Impact)

- `src/value.rs`：`Value::Promise` 的字段匹配和原生函数绑定。
- `src/promise.rs`：`Reaction` 状态枚举。
- `src/vm/fiber.rs`：`Fiber` 属性字段与初始化。
- `src/vm/interpreter.rs`：Fiber 执行清理、finally 覆盖以及错误捕获与 reject 传播。
- `src/vm.rs`：全局静态 `Promise` 对象方法注册和 Reaction 任务调度。
- `src/tokenizer.rs`：关键字的分词匹配逻辑（包含手写分词器与 Winnow 分词器）。
- `src/chen.pest`：Pest 语法解析定义。
