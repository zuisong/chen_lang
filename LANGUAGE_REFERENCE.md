# Chen Lang 语言参考手册

**版本**: 0.3.0  
**更新日期**: 2026-05-22  

---

## 📑 目录

1. [简介](#简介)
2. [基础语法](#基础语法)
3. [数据类型](#数据类型)
4. [变量和作用域](#变量和作用域)
5. [运算符](#运算符)
6. [控制流](#控制流)
7. [函数与异步](#函数与异步)
8. [对象与原型系统](#对象与原型系统)
9. [数组](#数组)
10. [异常处理](#异常处理)
11. [内置全局对象](#内置全局对象)
12. [Chen 运行时命名空间](#chen-运行时命名空间)
13. [示例程序](#示例程序)
14. [最佳实践与常见问题](#最佳实践与常见问题)

---

## 简介

**Chen Lang** 是一个简洁、采用 JS 风格的动态类型脚本语言。它由 Rust 实现，底层基于一个轻量高效的字节码虚拟机。

### 核心特性
- 🎯 **JS 语法风格**：绝大部分代码写法与 JavaScript 极其相似。
- 🔄 **动态类型**：变量在运行时绑定类型。
- 📦 **原型链继承**：支持基于 JavaScript 风格的对象和原型链，也保留了底层的高级元表（Metatable）定制能力。
- ⚡ **高精度数值**：Float 类型在底层使用 Decimal（高精度十进制数）表示，彻底避免了 `0.1 + 0.2 != 0.3` 的浮点误差。
- 🛡️ **异常处理**：完整的 `try-catch-finally` 机制。
- 🟢 **异步控制**：支持 `Promise`、`async/await`，基于轻量级 Fiber 协程调度。

### 运行环境与执行
Chen Lang 源文件推荐使用 `.chen.js` 作为后缀名。

```bash
# 启动交互式命令行（REPL）
cargo run --bin chen_lang -- repl

# 运行一个指定的代码文件
cargo run --bin chen_lang -- run demo.chen.js

# 从标准输入直接执行代码
echo 'console.log("Hello from stdin")' | cargo run --bin chen_lang -- run -
```

---

## 基础语法

### 注释
Chen Lang 仅支持 JS 风格的单行注释。旧版 `#` 注释不再支持：

```js
// 这是单行注释

// 可以连续多行
// 来编写多行说明
```

### 语句与分号
语句的分隔是**分号可选**的。你可以使用新行，或者显式使用分号 `;` 分隔：

```js
let x = 10
let y = 20; let z = x + y
```

### 代码块
使用花括号 `{}` 定义语句块：

```js
if (x > 0) {
    let msg = "positive"
    console.log(msg)
}
```

---

## 数据类型

Chen Lang 支持以下基础数据类型：

| 类型 | 说明 | 示例 |
| :--- | :--- | :--- |
| **Integer** | 32位带符号整数 | `42`, `-10`, `0` |
| **Float** | 基于 Decimal 的高精度十进制浮点数 | `3.14`, `-0.01`, `0.1` |
| **String** | 字符串类型，支持双引号与单引号 | `"Hello"`, `'World'` |
| **Boolean** | 布尔真/假值 | `true`, `false` |
| **Null** | 空值，表示没有值或未定义 | `null` |
| **Array** | 顺序数组列表 | `[1, 2, "three", null]` |
| **Object** | 键值对表 | `{ name: "Alice", age: 25 }` |
| **Function**| 可执行的函数（包含闭包与原生函数） | `function(a) { return a }` |

---

## 变量和作用域

### 变量声明与赋值
使用 `let` 关键字声明变量。声明后可以随时重新赋值：

```js
let x = 10
x = 20 // 重新赋值
```

### 块级作用域
变量具有严格的词法块级作用域。在 `{}` 内部声明的变量在块外无法访问：

```js
let outer = "global"

if (true) {
    let inner = "local"
    console.log(outer) // "global"
    console.log(inner) // "local"
}

// console.log(inner) // 错误！inner 在当前作用域未定义
```

---

## 运算符

### 算术运算符
支持基本的算术运算。所有浮点数运算均为高精度（Decimal）：

```js
let a = 10
let b = 3
console.log(a + b)  // 13
console.log(a - b)  // 7
console.log(a * b)  // 30
console.log(a / b)  // 3.3333333333333333333333333333
console.log(a % b)  // 1 (取余数)
```

### 比较运算符
```js
let x = 10
let y = 20
console.log(x == y)  // false
console.log(x != y)  // true
console.log(x < y)   // true
console.log(x >= y)  // false
```

### 逻辑运算符
`&&`（逻辑与）和 `||`（逻辑或）会返回操作数本身（短路逻辑）；`!`（逻辑非）始终返回布尔值：

```js
let a = "hello" && 123  // 返回 123 (真值)
let b = null || "fallback"  // 返回 "fallback"
let c = !0  // 返回 true
```

#### 真值与假值（Truthiness）
- **假值 (Falsey)**: `false`, `null`, 整数 `0`, 浮点数 `0.0`, 空字符串 `""`。
- **真值 (Truthy)**: 除上述假值外的所有其他值（如非空数组、非空对象、非空字符串等）。

### 字符串拼接
使用 `+` 运算符。如果其中一个操作数是字符串，Chen Lang 会自动将另一个操作数转换为字符串进行拼接：

```js
let text = "Score: " + 98.5 // "Score: 98.5"
```

---

## 控制流

### If-Else 条件分支
条件表达式**必须**用小括号 `()` 包裹：

```js
let score = 85

if (score >= 90) {
    console.log("A")
} else if (score >= 80) {
    console.log("B")
} else {
    console.log("C")
}
```

If 可以作为表达式使用（类似于三元运算符）：

```js
let status = if (age >= 18) { "adult" } else { "minor" }
```

### While 循环
条件表达式**必须**用小括号 `()` 包裹：

```js
let i = 0
while (i < 5) {
    console.log(i)
    i = i + 1
}
```

### For-Of 循环
用于遍历数组、对象（的值）以及字符串：

```js
let arr = ["A", "B", "C"]
for (let item of arr) {
    console.log(item) // "A", "B", "C"
}
```

### For Await-Of 循环
用于遍历异步迭代器（例如具有 `asyncIter()` 方法的对象），通常在 `async` 函数内部使用：

```js
let async_iterable = {
    asyncIter: function() {
        let i = 0
        return {
            next: async function() {
                if (i < 3) {
                    i = i + 1
                    return { value: i, done: false }
                }
                return { value: null, done: true }
            }
        }
    }
}

async function run() {
    for await (let x of async_iterable) {
        console.log(x) // 依次打印 1, 2, 3
    }
}
```

### Break 与 Continue
用于控制循环退出与跳过当前迭代：

```js
let i = 0
while (i < 10) {
    i = i + 1
    if (i == 5) {
        continue // 跳过本次
    }
    if (i == 8) {
        break // 终止循环
    }
    console.log(i)
}
```

---

## 函数与异步

### 函数定义
使用 `function` 关键字（旧版 `def` 也保留支持）定义函数：

```js
// 命名函数
function add(a, b) {
    return a + b
}

// 匿名函数表达式
let multiply = function(a, b) {
    return a * b
}
```

### 隐式返回
如果函数体内没有显式使用 `return` 语句，函数体内**最后一个表达式的值**将被自动作为返回值返回：

```js
function square(x) {
    x * x // 隐式返回 x * x 的结果
}
console.log(square(5)) // 25
```

### 异步函数 (`async/await`)
`async` 声明的函数会自动返回一个 `Promise`。在 `async` 函数内部，可以使用 `await` 暂停当前 Fiber 并等待 Promise 决议：

```js
async function fetchData() {
    // 异步等待定时器
    await Chen.timer.sleep(100)
    return "data"
}

async function main() {
    console.log("开始加载")
    let result = await fetchData()
    console.log("结果: " + result)
}

main()
```

---

## 对象与原型系统

### 对象字面量
使用键值对的 `{}` 声明对象。键可以是标识符、字符串或整数：

```js
let user = {
    name: "Alice",
    age: 30,
    "current city": "Beijing"
}

// 属性访问与修改
console.log(user.name) // "Alice"
user.age = 31
console.log(user["current city"]) // "Beijing"
```

### 原型继承
使用 `Object.create(proto)` 创建一个指定原型的新对象。当访问对象属性不存在时，会沿原型链向上查找：

```js
let proto = { name: "prototype_name" }
let obj = Object.create(proto)
obj.own_prop = "value"

console.log(obj.own_prop) // "value" (自有属性)
console.log(obj.name)     // "prototype_name" (继承自原型)
```

### 方法与 `this` 绑定
在对象上调用方法时，如果使用点号 `obj.method(...)`，那么在 `method` 函数内部，`this` 会自动绑定到调用对象 `obj`。  
普通函数直接调用不会绑定 `this`。在未绑定 `this` 的上下文中使用 `this` 关键字会抛出运行时错误：

```js
let dog = {
    name: "Buddy",
    bark: function() {
        return this.name + " says woof!"
    }
}

// 方法调用：this 绑定到 dog
console.log(dog.bark()) // "Buddy says woof!"

// 提取方法为普通函数：失去了 this 绑定
let barkFunc = dog.bark
// barkFunc() // 运行时报错：this unbound error
```

### 底层元表（Metatable）与元方法
对于高级开发，Chen Lang 提供了类似 Lua 的元表机制。可以使用 `Chen.setMeta(obj, meta)` 和 `Chen.getMeta(obj)`。

#### 支持的元方法：
- `__index`：属性查找拦截器。可以是一个包含属性的对象，或者是函数 `function(obj, key)`。
- `__newindex`：属性写入拦截器。必须是一个函数 `function(obj, key, value)`。
- `__add`：自定义加法行为 `+`。
- `__sub`：自定义减法行为 `-`。
- `__mul`：自定义乘法行为 `*`。

```js
let proto = {
    __index: function(obj, key) {
        return "fallback_" + key
    }
}
let target = {}
Chen.setMeta(target, proto)
console.log(target.anything) // "fallback_anything"
```

---

## 数组

数组在底层是绑定了数组原型的特殊对象，索引从 `0` 开始。

### 创建与修改
```js
let arr = [10, 20, 30]
console.log(arr[0]) // 10
arr[1] = 99
```

### 属性与方法
- **`.length`**: 获取数组的长度（成员个数）。
- **`arr.push(value)`**: 向数组末尾添加一个元素，返回新数组长度。
- **`arr.pop()`**: 弹出并返回数组最后一个元素。
- **`arr.iter()`**: 获取用于 `for-of` 遍历的只读迭代器。
- **`arr.entries()`**: 获取包含键值对 `{ key: index, value: val }` 的迭代器。

```js
let list = [1, 2]
console.log(list.length) // 2
list.push(3)
console.log(list.length) // 3
console.log(list.pop())    // 3
```

---

## 异常处理

### Try-Catch-Finally
任何类型的值均可被 `throw`。捕获变量在 `catch` 后必须加上小括号 `()`：

```js
try {
    throw { code: 500, message: "Internal Error" }
} catch (err) {
    console.log("Error code: " + err.code) // 500
} finally {
    console.log("清理工作完毕")
}
```

也可以不声明异常变量：

```js
try {
    throw "Oops"
} catch {
    console.log("发生了未知错误")
}
```

---

## 内置全局对象

无需特殊导入即可直接在全局作用域使用的对象：

### `console`

- `console.log(arg1, arg2, ...)`: 打印内容并自动换行。
- `console.info(arg1, arg2, ...)`: `console.log` 的别名。
- `console.warn(arg1, arg2, ...)`: `console.log` 的别名。
- `console.error(arg1, arg2, ...)`: `console.log` 的别名。
- `console.debug(arg1, arg2, ...)`: `console.log` 的别名。
- `console.print(arg1, arg2, ...)`: 打印内容，末尾不换行。
- `console.readLine()`: 同步阻塞读取控制台的一行输入（非标准，建议使用 `Chen.io.readline`）。

### `JSON`
数据序列化和反序列化：
- `JSON.stringify(value)`: 将 Chen Lang 值转换为 JSON 字符串。
- `JSON.parse(str)`: 将 JSON 字符串解析为 Chen Lang 原生对象/数组/值。

### `Object`
原型管理与反射：
- `Object.create(proto)`: 创建以 `proto` 为原型的新对象。
- `Object.keys(obj)`: 返回一个包含对象所有可枚举键名的数组。
- `Object.entries(obj)`: 返回一个包含对象所有 `[key, value]` 键值对的二维数组。

### `Promise`
异步决议与状态控制：
- **`Promise.new(executor)`**: 创建新 Promise。`executor` 是 `function(resolve, reject) {}`。
- **`Promise.resolve(value)`**: 返回一个已 Fulfilled 的 Promise。
- **`Promise.reject(reason)`**: 返回一个已 Rejected 的 Promise。
- **`Promise.all(array)`**: 并发等待数组中的所有 Promise 完成。全部成功返回结果数组，有一个失败则立即 Rejected。
- **`Promise.race(array)`**: 返回第一个完成（无论成功/失败）的 Promise 结果。
- **`Promise.allSettled(array)`**: 等待所有 Promise 敲定，返回的数组包含所有结果状态对象 `{ status: "fulfilled", value: val }` 或 `{ status: "rejected", reason: err }`。

#### 实例方法：
- **`promise.then(onFulfilled, onRejected)`**
- **`promise.catch(onRejected)`**
- **`promise.finally(onFinally)`**：无论成败都执行。

---

## Chen 运行时命名空间

Chen Lang 独有的核心模块与工具全部集中在全局的 `Chen` 命名空间下。不需要手动 `import`：

### `Chen.fs` (文件系统)
- `Chen.fs.readTextFile(path)`: 同步读取文本文件，返回字符串。
- `Chen.fs.writeTextFile(path, text)`: 同步写入文本内容到指定路径。
- `Chen.fs.readDir(path)`: 同步读取目录，返回文件/子目录名数组。
- `Chen.fs.exists(path)`: 判断路径是否存在，返回布尔值。
- `Chen.fs.remove(path)`: 移除文件或目录。

### `Chen.timer` (定时器)
- `Chen.timer.sleep(ms)`: 异步挂起当前 Fiber 指定的毫秒数。**必须在 async 函数中使用 await 调用**：
  ```js
  await Chen.timer.sleep(1000) // 睡眠 1 秒
  ```

### `Chen.date` (日期处理)
- `Chen.date.new(val?)`: 构造一个新的 Date 对象实例。`val` 可为空（获取当前时间）、ISO时间字符串，或毫秒时间戳。
- `Chen.date.now()`: 获取当前时间戳（Float类型毫秒数）。
- **Date 实例方法**:
  - `date.format(fmt)`: 格式化日期（如 `%Y-%m-%d %H:%M:%S`）。
  - `date.timestamp()`: 返回对应的毫秒时间戳。

### `Chen.process` (进程控制)
- `Chen.process.exit(code)`: 以指定退出码结束当前程序进程。
- `Chen.process.args()`: 获取命令行参数数组。
- `Chen.process.env()`: 获取当前进程的环境变量对象。

### `Chen.load(path)` (模块加载)
执行指定路径的 `.chen.js` 模块文件。被加载的模块在独立的沙箱作用域中运行，模块文件的**最后一个表达式的值**作为模块的导出对象返回。模块加载结果会被自动缓存：

```js
// math.chen.js
let utils = {
    add: function(a, b) { a + b }
}
utils // 作为最后一行表达式导出

// main.chen.js
let math = Chen.load("math.chen.js")
console.log(math.add(2, 3)) // 5
```

---

## 示例程序

### 1. 斐波那契数列 (递归实现)
```js
function fibonacci(n) {
    if (n <= 1) {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

let i = 0
while (i < 10) {
    console.log("fib(" + i + ") = " + fibonacci(i))
    i = i + 1
}
```

### 2. 九九乘法表
```js
let i = 1
while (i <= 9) {
    let j = 1
    while (j <= i) {
        console.print(j + " × " + i + " = " + (i * j) + "  ")
        j = j + 1
    }
    console.log("")
    i = i + 1
}
```

### 3. 基于原型的 Point 类与加法重载
```js
// 定义 Point 类的原型
let PointPrototype = {
    toString: function() {
        return "Point(" + this.x + ", " + this.y + ")"
    }
}

// 类元表，包含加法操作符重载
let PointMeta = {
    __index: PointPrototype,
    __add: function(a, b) {
        return newPoint(a.x + b.x, a.y + b.y)
    }
}

// 构造函数
function newPoint(x, y) {
    let instance = { x: x, y: y }
    Chen.setMeta(instance, PointMeta)
    return instance
}

let p1 = newPoint(10, 20)
let p2 = newPoint(5, 8)
let p3 = p1 + p2

console.log(p3.toString()) // "Point(15, 28)"
```

---

## 最佳实践与常见问题

### 1. 变量命名规范
- 局部变量和普通函数名建议使用小驼峰（`camelCase`）或蛇形命名（`snake_case`）。
- 构造函数建议使用大驼峰（`CamelCase`）或者 `new` 前缀（`newPoint`）。

### 2. this 指向与避免 unbound 错误
请确保在调用需要访问对象内部属性的方法时，使用 `obj.method()` 的形式。如果将 `obj.method` 赋值给另一个变量后再执行，该方法内部的 `this` 就会变为未绑定状态，导致抛出运行时错误。

### 3. Semicolon-free 与代码块换行
由于分号是可选的，请尽量保持大括号 `{` 紧跟在条件语句或函数定义行末，防止换行符被误解析为语句结束：

```js
// 推荐写法
if (x > 0) {
    console.log(x)
}

// 不推荐写法 (可能引发解析歧义)
if (x > 0)
{
    console.log(x)
}
```

### 4. 异步并发性能
在需要并发等待多个异步 I/O 操作时，优先选择 `Promise.all` 批量等待，而不是连续串行使用 `await`：

```js
// 推荐的并发做法
let results = await Promise.all([task1(), task2()])

// 较慢的串行做法
let res1 = await task1()
let res2 = await task2()
```

---
🎉 **祝你使用 Chen Lang 编程愉快！**
