# Async/Await 与 Coroutine 实现总结

本文档总结了 Chen 语言中 `async/await` 的实现机制、与 Lua/JS 的对比，以及调度器的设计思路。

## 1. 核心机制 (Stackful Coroutine)

Chen 语言目前的 `async/await` 是基于 **栈式协程 (Stackful Coroutines)** 的语法糖，底层复用了类似 Lua 的 `coroutine` 原语。

### 1.1 语法糖编译映射

*   **`async def func(...)`**:
    会被编译器自动拆分为两个部分：
    1.  **实现函数 (`func_impl`)**: 包含原有的函数体逻辑。
    2.  **包装函数 (`func`)**: 创建并返回一个 Fiber。
    ```javascript
    // 源代码
    async def foo(x) { return x + 1 }

    // 编译后等价逻辑
    def foo_impl(x) { return x + 1 }
    def foo(x) { return coroutine.create(foo_impl, x) }
    ```

*   **`await expr`**:
    会被编译为直接的 `yield` 调用。
    ```javascript
    // 源代码
    let v = await x
    
    // 编译后等价逻辑
    let v = coroutine.yield(x)
    ```

### 1.2 原语支持

VM 提供了以下 Native 函数（类似 Lua）：
*   `coroutine.create(fn, ...args)`: 创建协程，支持传入初始参数。
*   `coroutine.resume(co, val)`: 恢复协程执行，`val` 会成为协程内部 `yield` 的返回值。
*   `coroutine.yield(val)`: 挂起协程，`val` 会成为外部 `resume` 的返回值。
*   `coroutine.status(co)`: 查询状态 (`dead`, `suspended`, `running`)。

---

## 2. 与其他语言的对比

| 特性 | Chen (当前) / Lua | JavaScript (Promise) | Go (Goroutine) |
| :--- | :--- | :--- | :--- |
| **底层模型** | **Stackful** (有独立栈) | **Stackless** (状态机/回调) | **Stackful** (M:N 线程复用) |
| **调度方式** | **协作式/手动** (需显式 resume) | **自动** (Event Loop) | **自动** (Runtime Scheduler) |
| **await含义**| Yield (把值扔给调度器) | Unwrap (等待 Promise 结果) | (无此关键字，直接阻塞写法) |
| **IO 模型** | 需配合调度器实现非阻塞 | 内置非阻塞 | 内置非阻塞 (Poller) |

### 核心区别
*   **Chen**: `await` 只负责“暂停并产出值”，**不负责**“等待结果”。等待结果的逻辑由外部调度器决定。
*   **JS**: `await` 隐含了“注册回调并等待”的语义。

---

## 3. 调度器与异步 IO 设计

由于 Chen 只提供原语，实现完整的异步 I/O (如 `read_file`) 需要在标准库或用户层实现一个**调度器 (Scheduler)**。

### 3.1 调度器模型 (Wheel 模式)

```javascript
class Scheduler {
   tasks = [] // 待运行的任务队列 (Fiber)

   def run() {
       while !tasks.is_empty() {
           let fiber = tasks.pop()
           let status = coroutine.status(fiber)
           if status == "dead" { continue }

           // 执行一步
           let yield_val = coroutine.resume(fiber)
           
           // 根据 yield 出来的值决定下一步
           if is_io_token(yield_val) {
               // 这是一个 IO 请求，交给 Poller，暂不放回 tasks
               Poller.register(yield_val, fiber)
           } else if is_fiber(yield_val) {
               // await 了另一个 async 函数
               tasks.push(fiber)     // 自己稍后再跑
               tasks.push(yield_val) // 先跑子任务
           } else {
               // 普通 yield，继续排队
               if coroutine.status(fiber) != "dead" {
                   tasks.push(fiber)
               }
           }
       }
   }
}
```

### 3.2 统一抽象问题

**问题**：`async func` 返回的是 Fiber，但 `io.read` 可能返回一个 Token/Handle，类型不一致怎么办？

**方案 A：协议化 (Protocol/Interface)**
不需要强制类型一致。调度器只要能识别不同的 yield 值即可（Duck Typing）。
*   Yield `Fiber` -> 优先执行子任务。
*   Yield `IoToken` -> 注册到 Epoll/Kqueue。
*   Yield `Int/String` -> 可能是用户自定义逻辑，忽略或透传。

**方案 B：Promise 化 (JS 风格)**
如果不喜欢异构类型，可以引入 `Promise` 类：
*   让 `async func` 返回 `Promise` (内部包装 Fiber)。
*   让 `io.read` 也返回 `Promise` (内部包装 IO Handle)。
*   但这需要更复杂的 VM 运行时支持。

**结论**：目前 Chen 采用方案 A，保持内核精简，灵活性类似于 Lua。

---

## 4. 当前状态

*   [x] 关键字 `async/await` 支持
*   [x] 编译器变换 (`def` -> `wrapper + impl`)
*   [x] VM 协程原语 (`create`, `resume`, `yield`)
*   [x] 参数传递与返回值修正
*   [x] 协程状态查询 (`status`)
*   [ ] 标准库调度器 (待实现)
*   [ ] 异步 IO 接口 (待实现)
