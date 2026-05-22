## 背景 (Context)

在将协程模型移除并全量迁移到基于 Promise 的异步机制后，Chen Lang 目前仅有基本的 Promise 实例化等骨架，缺乏对高级 Promise 链式操作和并行操作的完整支持。我们需要为运行时和词法/语法层提供高质量的设计方案，以便使 Promise 相关机制及 `def` 关键字别名能健壮地运行。

## 目标与非目标 (Goals / Non-Goals)

**目标：**
- 提供 `.then`、`.catch`、`.finally` 实例链式调用支持。
- 提供 `Promise.new`、`Promise.all`、`Promise.race`、`Promise.allSettled` 静态接口。
- 支持 `def` 关键字，在 Pest 模式和手写解析器模式下无缝工作。

**非目标：**
- 引入类似 JavaScript 的 `new Promise` 构造器语法（我们只提供 `Promise.new(executor)` 静态方法）。
- 改变 Chen Lang 的核心 VM 线程模型或引入第三方异步库。

## 技术决策 (Decisions)

### 1. Promise 实例方法的实现与 Reaction 类型扩展
- **设计选择**：由于 Promise 敲定（Settled）后需要调度回调，我们通过 `src/promise.rs` 中定义的 `Reaction` 进行注册。为了支持 `.finally()`，在 `Reaction` 中新增 `Finally` 变体，以捕获 finally 状态并保证能返回前一个 Promise 的 settled 状态。
- **Fiber 状态扩展**：在 `Fiber` 结构中新增 `finally_initial_state` 字段（保留 finally 之前的决议状态），并在 Fiber 正常退出时自动恢复此状态（除非 finally 运行中途抛出异常）；新增 `reject_on_error_promise` 字段，用于在 Fiber 出现未捕获运行时错误时，自动 reject 对应的 Promise，从而保证错误能够在链式调用中被正确捕获和向下传播。

### 2. 全局 Promise 静态方法的注册与控制
- **`Promise.new`**：通过构造原生的 `resolve` 和 `reject` 的 native 闭包，并将其传入 `executor`。如果是用户自定义函数（`Value::Fn`），则为其生成一个 Fiber 异步调度执行；如果是 Native 函数，则在主 VM 中同步执行。
- **可迭代对象聚合（`all`, `race`, `allSettled`）**：在 Rust 侧读取数组中的每一个元素，并利用 `.then` / `.catch` 为每一个 Promise 元素注册 Reaction 观察回调。利用一个计数器上下文和局部结果数组进行同步。

### 3. `def` 关键字的别名解析
- **词法层面拦截**：在手写分词器和 Winnow 分词器匹配字符串时，如果匹配到 `"def"`，直接将其转换为 `Token::Keyword(Keyword::FUNCTION)`。这种设计在词法层面就把 `def` 转换成了和 `function` 相同的 token，从而保证手写的语法分析器（`src/parser/handwritten.rs`）无需做任何代码修改。
- **Pest 语法更新**：在 `src/chen.pest` 中将 `FUNCTION` 规则更新为 `FUNCTION = @{ ("function" | "def") ~ !(ASCII_ALPHANUMERIC | "_") }`，使得 Pest AST 能够统一识别并将它们解析成同一语法树节点。

## 风险与权衡 (Risks / Trade-offs)

- **[风险] `finally` 异常覆盖** → 如果 `.finally()` 里的回调自身执行成功，根据标准应该丢弃其返回值并恢复之前的 settled 结果；但如果它执行中途报错，应该以新的错误来 reject。
  * *缓解措施*：通过 `Fiber::finally_initial_state` 机制实现精确的退出状态拦截。
- **[风险] 多个 `def` 关键字在错误栈提示中的展示** → 由于词法层面完全等价为 `FUNCTION`，在错误栈或调试信息中可能只显示 `function`。
  * *权衡*：这为解析器提供了最简、最稳健的改动，且不影响运行时实际语义。
