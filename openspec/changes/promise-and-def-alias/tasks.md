## 1. 分词与语法解析器更新

- [x] 1.1 更新 Pest 语法定义文件 `src/chen.pest`，使 `FUNCTION` 规则支持 `("function" | "def")`
- [x] 1.2 更新 `src/tokenizer.rs` 中的 Winnow 分词器匹配逻辑，将 `"def"` 映射为 `Token::Keyword(Keyword::FUNCTION)`
- [x] 1.3 更新 `src/tokenizer.rs` 中的手写分词器匹配逻辑，将 `"def"` 映射为 `Token::Keyword(Keyword::FUNCTION)`

## 2. Promise 实例方法实现

- [x] 2.1 在 `src/promise.rs` 的 `Reaction` 枚举中新增 `Finally` 变体
- [x] 2.2 在 `src/vm/fiber.rs` 的 `Fiber` 结构体中添加 `finally_initial_state` 和 `reject_on_error_promise` 字段并完成初始化
- [x] 2.3 在 `src/vm/interpreter.rs` 的 Fiber 正常/异常退出分支中，处理 `finally` 状态的重置与异常的自动 Promise reject 决议
- [x] 2.4 在 `src/value.rs` 中为 `Value::Promise` 实例字段获取绑定 `.then`、`.catch`、`.finally` 方法的 Native 函数

## 3. Promise 静态接口支持与事件调度

- [x] 3.1 更新 `src/vm.rs` 以支持 `Reaction::Finally` 的调度，并修改 `spawn_callback_fiber` 支持传入 `finally_initial_state`
- [x] 3.2 在全局 `Promise` 对象中实现并注册静态方法：`Promise.new`
- [x] 3.3 在全局 `Promise` 对象中实现并注册静态方法：`Promise.all`、`Promise.race` 和 `Promise.allSettled`

## 4. 测试与验证

- [x] 4.1 在 `tests/chen_lang_tests/language/async_await_tests.rs` 中使用 `def` 定义回调函数并编写全面的集成测试覆盖链式调用及静态接口
- [x] 4.2 运行完整自动化测试套件 `cargo test` 确保 165 个测试全量通过
