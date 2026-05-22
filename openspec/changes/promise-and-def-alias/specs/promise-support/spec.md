## ADDED Requirements

### Requirement: Promise 实例方法链式调用与异常传播
Promise 实例必须提供 `.then(onFulfilled, onRejected)`、`.catch(onRejected)` 和 `.finally(onFinally)` 接口。
当 Promise 被解决（settled）时，应当触发相应的回调；异常应当能被正确捕获和向下传播；`.finally` 执行完毕后，应当保持并恢复先前的决议状态（除非 finally 回调本身执行抛出异常）。

#### Scenario: 成功链式调用与值传递
- **WHEN** 对 Resolved Promise 调用 `.then(def(v) { return v + 5 })` 后再调用 `.then(def(v) { console.print(v) })`
- **THEN** 控制台应当打印出累加后的结果值

#### Scenario: 异常被 catch 捕获
- **WHEN** 运行 Rejected Promise，在后续链条上调用 `.catch(def(err) { console.print(err) })`
- **THEN** 控制台应当打印出被拒绝的原因（Reason）

#### Scenario: finally 回调运行并恢复状态
- **WHEN** Promise 被解决，运行 `.finally(def() { console.print("finally") })`
- **THEN** 控制台打印 "finally"，且该链条的最终决议状态仍恢复为之前的 resolve 值

### Requirement: Promise 全局静态方法
`Promise` 对象必须提供 `Promise.new(executor)` 静态工厂方法创建 Promise，并提供 `Promise.all(iterable)`、`Promise.race(iterable)` 和 `Promise.allSettled(iterable)` 进行并发操作。

#### Scenario: Promise.new 异步决议
- **WHEN** 调用 `Promise.new(def(resolve, reject) { resolve("resolved_value") })` 并 await 该 Promise
- **THEN** 表达式应当正确决议为 "resolved_value"

#### Scenario: Promise.all 全部成功
- **WHEN** 传入多个 Promise 数组调用 `Promise.all` 并 await
- **THEN** 应当返回包含所有 Promise 决议值的数组

#### Scenario: Promise.race 竞争决议
- **WHEN** 传入多个不同延迟时间的 Promise 数组给 `Promise.race` 并 await
- **THEN** 应当决议为最快决议的那一个 Promise 的结果

#### Scenario: Promise.allSettled 敲定聚合
- **WHEN** 传入混合成功和失败的 Promise 数组给 `Promise.allSettled` 并 await
- **THEN** 返回一个包含每一项敲定状态 `{ status: "fulfilled", value: ... }` 或 `{ status: "rejected", reason: ... }` 的结果数组
