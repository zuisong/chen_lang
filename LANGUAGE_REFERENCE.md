**版本**: 0.2.0

**更新日期**: 2026-02-01

---

## 📑 目录

1. [简介](#简介)
2. [基础语法](#基础语法)
3. [数据类型](#数据类型)
4. [变量和作用域](#变量和作用域)
5. [运算符](#运算符)
6. [控制流](#控制流)
7. [函数](#函数)
8. [对象和元表](#对象和元表)
9. [数组](#数组)
10. [异常处理](#异常处理)
11. [标准库](#标准库)
12. [示例程序](#示例程序)

---

## 简介

Chen Lang 是一个简洁、动态类型的编程语言,具有以下特点:

- 🎯 **简洁语法** - 易于学习和使用
- 🔄 **动态类型** - 灵活的类型系统
- 📦 **对象系统** - 基于原型的对象模型
- ⚡ **高精度数值** - 使用 Decimal 类型避免浮点误差
- 🛡️ **异常处理** - 完整的 try-catch-finally 机制
- 🚀 **快速执行** - 基于字节码的虚拟机

### 运行示例

```bash
# 运行demo文件
cargo run --bin chen_lang -- run demo_codes/fibonacci.ch

# 从标准输入运行程序
echo 'let io = import("stdlib/io"); io.println("Hello from stdin")' | cargo run --bin chen_lang -- run -

# 直接运行代码
echo 'let x = 5; let y = 3; print(x + y)' | cargo run --bin chen_lang -- run -
```

---

## 基础语法

### 注释

```python
# 这是单行注释

# 多行注释需要每行都用 # 开头
# 第二行注释
# 第三行注释
```

### 语句分隔

Chen Lang 使用换行符作为语句分隔符:

```python
let x = 10
let y = 20
let z = x + y
```

### 代码块

使用花括号 `{}` 定义代码块:

```python
if x > 0 {
    println("Positive")
}

for i < 10 {
    println(i)
    i = i + 1
}
```

---

## 数据类型

Chen Lang 支持以下数据类型:

### 1. 整数 (Integer)

```python
let age = 25
let negative = -100
let zero = 0
```

### 2. 浮点数 (Float)

使用高精度 Decimal 类型,避免浮点误差:

```python
let price = 19.99
let pi = 3.14159
let result = 0.1 + 0.2  # 结果是精确的 0.3
```

### 3. 字符串 (String)

使用双引号或单引号:

```python
let name = "Chen Lang"
let message = 'Hello, World!'

# 字符串拼接
let greeting = "Hello, " + name
```

### 4. 布尔值 (Boolean)

```python
let is_valid = true
let is_empty = false
```

### 5. 空值 (Null)

```python
let empty = null
```

### 6. 对象 (Object)

使用 `${}` 创建对象:

```python
let person = ${
    name: "Alice",
    age: 30,
    city: "Beijing"
}
```

### 7. 数组 (Array)

使用 `[]` 创建数组:

```python
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "two", true, null]
```

### 8. 函数 (Function)

函数是一等公民:

```python
let add = def(a, b) {
    a + b
}
```

---

## 变量和作用域

### 变量声明

使用 `let` 关键字声明变量:

```python
let x = 10
let name = "Chen"
let is_valid = true
```

### 变量赋值

```python
let x = 10
x = 20  # 重新赋值
```

### 作用域

Chen Lang 使用词法作用域:

```python
let global_var = "global"

def my_function() {
    let local_var = "local"
    # println 需要导入，这里假设已导入
    # println(global_var)  # 可以访问全局变量
    # println(local_var)   # 可以访问局部变量
}

# println(local_var)  # 错误!无法访问局部变量
```

### 块级作用域

```python
let x = 10

if true {
    let y = 20
    println(x)  # 10
    println(y)  # 20
}

# println(y)  # 错误!y 在块外不可见
```

---

## 运算符

### 算术运算符

```python
let a = 10
let b = 3

let sum = a + b        # 13
let diff = a - b       # 7
let product = a * b    # 30
let quotient = a / b   # 3.333...
let remainder = a % b  # 1
```

### 比较运算符

```python
let x = 10
let y = 20

x == y   # false (等于)
x != y   # true  (不等于)
x < y    # true  (小于)
x <= y   # true  (小于等于)
x > y    # false (大于)
x >= y   # false (大于等于)
```

### 逻辑运算符

```python
let a = true
let b = false

a && b   # false (逻辑与)
a || b   # true  (逻辑或)
!a       # false (逻辑非)
```

### 字符串拼接

```python
let first = "Hello"
let second = "World"
let result = first + " " + second  # "Hello World"
```

### 运算符优先级

从高到低:

1. `!` (逻辑非), `-` (负号)
2. `*`, `/`, `%`
3. `+`, `-`
4. `<`, `<=`, `>`, `>=`
5. `==`, `!=`
6. `&&`
7. `||`

---

## 控制流

### If-Else 语句

```python
let score = 85

if score >= 90 {
    println("A")
} else if score >= 80 {
    println("B")
} else {
    println("C")
}
```

### If 表达式

If 可以作为表达式使用:

```python
let status = if age >= 18 { "adult" } else { "minor" }
```

### For 循环

Chen Lang 的 `for` 循环非常灵活, 支持条件循环、无限循环以及集合迭代。

#### 1. 条件循环 (Go 风格)

```python
let i = 0
for i < 10 {
    println(i)
    i = i + 1
}
```

#### 2. 无限循环

```python
for {
    if some_condition() {
        break
    }
}
```

#### 3. 集合迭代 (For-In)

可以使用 `for...in` 语法遍历数组、对象、字符串以及协程。它会自动调用集合的
`:iter()` 方法。

```python
# 遍历数组值
let arr = ["A", "B", "C"]
for x in arr {
    println(x)
}

# 遍历对象值
let obj = ${ a: 1, b: 2 }
for v in obj {
    println(v)
}

# 遍历字符串字符
for char in "Hello" {
    println(char)
}
```

#### 4. 键值对迭代 (Entries)

如果需要同时获取索引/键和值，可以使用 `:entries()` 方法。它会返回一个包含 `key`
和 `value` 属性的对象。

```python
for e in arr:entries() {
    println("Index: " + e.key + ", Value: " + e.value)
}

for e in obj:entries() {
    println("Key: " + e.key + ", Value: " + e.value)
}
```

### Break 和 Continue

```python
let i = 0
for i < 10 {
    if i == 5 {
        break  # 退出循环
    }
    if i % 2 == 0 {
        i = i + 1
        continue  # 跳过本次迭代
    }
    println(i)
    i = i + 1
}
```

---

## 函数

### 函数定义

```python
def greet(name) {
    println("Hello, " + name + "!")
}

greet("Alice")  # 输出: Hello, Alice!
```

### 带返回值的函数

```python
def add(a, b) {
    return a + b
}

let result = add(10, 20)  # 30
```

### 隐式返回

函数的最后一个表达式会自动返回:

```python
def multiply(a, b) {
    a * b  # 隐式返回
}

let result = multiply(5, 6)  # 30
```

### 匿名函数

```python
let square = def(x) {
    x * x
}

println(square(5))  # 25
```

### 递归函数

```python
def fibonacci(n) {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

println(fibonacci(10))  # 55
```

### 嵌套函数

```python
def outer() {
    def inner() {
        println("Inner function")
    }
    inner()
}

outer()  # 输出: Inner function
```

### 函数作为参数

```python
def apply(func, value) {
    func(value)
}

def double(x) {
    x * 2
}

let result = apply(double, 10)  # 20
```

---

## 对象和元表

### 创建对象

```python
let person = ${
    name: "Alice",
    age: 30,
    city: "Beijing"
}
```

### 访问属性

```python
println(person.name)  # "Alice"
println(person.age)   # 30
```

### 修改属性

```python
person.age = 31
person.email = "alice@example.com"  # 添加新属性
```

### 对象方法

调用对象方法时，如果方法需要访问对象本身（即 `self`），请使用冒号 `:`
语法。冒号语法会自动将调用者作为第一个参数传递给方法。如果使用点号 `.`
调用，对象**不会**作为参数传递。

```python
let calculator = ${
    value: 0,
    add: def(self, n) {
        self.value = self.value + n
    },
    get: def(self) {
        self.value
    }
}

# 使用冒号调用（自动传递 self）
calculator:add(10)
calculator:add(5)
println(calculator:get())  # 15
```

### 元表 (Metatable)

元表用于实现高级特性,如运算符重载和方法查找:

```python
# 定义 Point 原型
let Point = ${
    __index: ${
        to_string: def(self) {
            "Point(" + self.x + ", " + self.y + ")"
        }
    },
    __add: def(a, b) {
        new_Point(a.x + b.x, a.y + b.y)
    }
}

# 构造函数
def new_Point(x, y) {
    let instance = ${ x: x, y: y }
    set_meta(instance, Point)
    return instance
}

# 使用
let p1 = new_Point(10, 20)
let p2 = new_Point(5, 10)
let p3 = p1 + p2  # 使用重载的 + 运算符

println(p3:to_string())  # "Point(15, 30)"
```

### 元方法

支持的元方法:

- `__add` - 加法 (+)
- `__sub` - 减法 (-)
- `__mul` - 乘法 (*)
- `__index` - 属性查找拦截。当访问不存在的属性时触发。它可以是一个对象（在该对象中继续查找），也可以是一个函数 `def(obj, key)`，用于动态返回默认值。
- `__newindex` - 属性赋值拦截。当给不存在的属性赋值时触发。它必须是一个函数 `def(obj, key, value)`，用于动态拦截和处理赋值行为。

#### 动态元方法示例

```python
let io = import("stdlib/io")

let proto = ${
    # 当查找不存在的属性时触发
    __index: def(obj, key) {
        return "fallback_" + key
    },
    # 当给不存在的属性赋值时触发
    __newindex: def(obj, key, value) {
        io.println("拦截到赋值: " + key + " = " + value)
        # 注意: 如果这里直接给 obj[key] 赋值会触发死循环
    }
}

let person = ${ name: "Alice" }
set_meta(person, proto)

# 触发 __index
println(person.age)  # 输出: fallback_age

# 触发 __newindex
person.city = "Beijing"  # 输出: 拦截到赋值: city = Beijing
```

---

## 数组

### 创建数组

```python
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "two", true, null]
let empty = []
```

### 访问元素

```python
let first = numbers[0]   # 1
let second = numbers[1]  # 2
```

### 修改元素

```python
numbers[0] = 10
numbers[5] = 6  # 添加新元素
```

### 数组方法

```python
let arr = [1, 2, 3]

# 获取长度
let length = arr:len()  # 3

# 添加元素
arr:push(4)  # 返回新长度 4, arr 变为 [1, 2, 3, 4]

# 弹出元素
let last = arr:pop()  # 返回 4, arr 变为 [1, 2, 3]

# 获取类型
println(arr.__type)  # "Array"
```

### 遍历数组

```python
let arr = [10, 20, 30]
let i = 0
for i < arr:len() {
    println(arr[i])
    i = i + 1
}
```

---

## 异常处理

### Try-Catch

```python
try {
    throw "Something went wrong!"
} catch error {
    println("Caught error: " + error)
}
```

### Try-Catch-Finally

```python
try {
    throw "Error"
} catch error {
    println("Error: " + error)
} finally {
    println("Cleanup")  # 总是执行
}
```

### 不带错误变量的 Catch

```python
try {
    throw "Error"
} catch {
    println("An error occurred")
}
```

### 函数中的异常

```python
def divide(a, b) {
    if b == 0 {
        throw "Division by zero"
    }
    a / b
}

try {
    let result = divide(10, 0)
} catch error {
    println("Error: " + error)
}
```

### 嵌套异常处理

```python
try {
    try {
        throw "Inner error"
    } catch e {
        println("Inner catch: " + e)
        throw "Outer error"
    }
} catch e {
    println("Outer catch: " + e)
}
```

---

## 模块系统与标准库

Chen Lang 采用显式导入机制。除了极少数核心功能（如 `null`,
`coroutine`）外，大部分标准库功能（即所谓的“标准库”）都需要通过 `import`
语句显式引入。

### 导入语法

```python
let <变量名> = import("<模块路径>")
```

示例:

```python
let JSON = import("stdlib/json")
let io = import("stdlib/io")
```

### 核心设计原则

1. **按需引入**: 减少全局命名空间污染，提高加载性能。
2. **显式依赖**: 从代码中可以清晰看到使用了哪些外部模块。

### 常用标准库模块

| 模块路径         | 返回对象包含的成员                | 说明              |
| :--------------- | :-------------------------------- | :---------------- |
| `stdlib/io`      | `print`, `println`, `readline`    | 标准输入输出      |
| `stdlib/json`    | `stringify`, `parse`              | JSON 序列化与解析 |
| `stdlib/date`    | `new`, `now`, `parse`             | 日期时间处理      |
| `stdlib/fs`      | `read_to_string`, `write_file` 等 | 文件系统操作      |
| `stdlib/http`    | `get`, `post` 等                  | HTTP 客户端       |
| `stdlib/process` | `exit`, `args`, `env` 等          | 进程与环境信息    |

### 自定义模块

你可以创建自己的 `.ch`
文件并导入它们。被导入的文件会作为独立的模块执行，最后一行表达式的值将作为模块的返回值。

例如，创建 `math_utils.ch`:

```python
# math_utils.ch
${
    add: def(a, b) { a + b },
    sub: def(a, b) { a - b }
}
```

在主程序中导入:

```python
let math_utils = import("math_utils.ch")

let result = math_utils.add(10, 20)
let io = import("stdlib/io")
io.println(result)  # 30
```

注意：导入路径是相对于当前执行目录或绝对路径。模块会被缓存，重复导入不会重新执行。

### 示例程序

```python
let io = import("stdlib/io")
let JSON = import("stdlib/json")

let score = 85
let level = if score >= 90 { "A" } else if score >= 60 { "P" } else { "F" }

let result = ${
    score: score,
    level: level
}

# 必须导入 stdlib/io 才能使用 println
io.println("Final Result: " + JSON.stringify(result))
```

### 输出函数

```python
# 打印(不换行)
print("Hello")
print(" World")  # 输出: Hello World

# 打印(换行)
println("Hello")
println("World")
# 输出:
# Hello
# World
```

### Date 对象

```python
# 创建当前时间
let now = Date:new()

# 获取类型
println(now.__type)  # "Date"

# 格式化日期
let formatted = now:format('%Y-%m-%d %H:%M:%S')
println(formatted)  # 例如: 2025-12-10 22:40:00

# 常用格式符号:
# %Y - 年份 (2025)
# %m - 月份 (01-12)
# %d - 日期 (01-31)
# %H - 小时 (00-23)
# %M - 分钟 (00-59)
# %S - 秒 (00-59)
```

### JSON 对象

```python
# 序列化为 JSON
let data = ${
    name: "Alice",
    age: 30,
    hobbies: ["reading", "coding"]
}
let json_str = JSON.stringify(data)
io.println(json_str)
# 输出: {"name":"Alice","age":30,"hobbies":["reading","coding"]}

# 解析 JSON
let parsed = JSON.parse(json_str)
println(parsed.name)  # "Alice"
```

### 字符串方法

```python
let text = "Hello, World!"

# 获取长度
let length = text:len()  # 13

# 转大写
let upper = text:upper()  # "HELLO, WORLD!"

# 转小写
let lower = text:lower()  # "hello, world!"

# 去除空白
let trimmed = "  hello  ":trim()  # "hello"

# 获取类型
println(text.__type)  # "String"
```

### 对象方法

```python
let obj = ${
    name: "Alice",
    age: 30,
    city: "Beijing"
}

# 获取所有键
let keys = obj:keys()  # ["name", "age", "city"]

# 获取迭代器 (仅返回值)
let it = obj:iter()

# 获取键值对迭代器
let entries = obj:entries()
# 返回的对象结构为: ${ key: "name", value: "Alice" }
```

### 元表函数

```python
# 设置元表
set_meta(object, metatable)

# 获取元表
let mt = get_meta(object)
```

---

## 示例程序

### 1. 斐波那契数列

```python
def fibonacci(n) {
    if n <= 1 {
        return n
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

let i = 0
for i < 10 {
    println("fib(" + i + ") = " + fibonacci(i))
    i = i + 1
}
```

### 2. 九九乘法表

```python
let i = 1
for i <= 9 {
    let j = 1
    for j <= i {
        print(j + " × " + i + " = " + (i * j) + "  ")
        j = j + 1
    }
    println("")
    i = i + 1
}
```

### 3. 计算器对象

```python
let calculator = ${
    value: 0,
    add: def(self, n) {
        self.value = self.value + n
        self
    },
    subtract: def(self, n) {
        self.value = self.value - n
        self
    },
    multiply: def(self, n) {
        self.value = self.value * n
        self
    },
    divide: def(self, n) {
        if n == 0 {
            throw "Division by zero"
        }
        self.value = self.value / n
        self
    },
    result: def(self) {
        self.value
    }
}

try {
    let result = calculator.add(10).multiply(5).subtract(20).result()
    println("Result: " + result)  # 30
} catch error {
    println("Error: " + error)
}
```

### 4. Point 类

```python
# Point 原型
let Point = ${
    __index: ${
        to_string: def(self) {
            "Point(" + self.x + ", " + self.y + ")"
        },
        move_by: def(self, dx, dy) {
            self.x = self.x + dx
            self.y = self.y + dy
        }
    },
    __add: def(a, b) {
        new_Point(a.x + b.x, a.y + b.y)
    },
    __sub: def(a, b) {
        new_Point(a.x - b.x, a.y - b.y)
    }
}

def new_Point(x, y) {
    let instance = ${ x: x, y: y }
    set_meta(instance, Point)
    return instance
}

# 使用
let p1 = new_Point(10, 20)
let p2 = new_Point(5, 10)

println(p1.to_string())  # "Point(10, 20)"
println(p2.to_string())  # "Point(5, 10)"

let p3 = p1 + p2
println(p3.to_string())  # "Point(15, 30)"

p1.move_by(5, -10)
println(p1.to_string())  # "Point(15, 10)"
```

### 5. 安全除法函数

```python
def safe_divide(a, b) {
    try {
        if b == 0 {
            throw "Division by zero"
        }
        return a / b
    } catch error {
        println("Error: " + error)
        return null
    }
}

println(safe_divide(10, 2))   # 5
println(safe_divide(10, 0))   # Error: Division by zero, 然后输出 null
```

---

## 最佳实践

### 1. 命名约定

```python
# 变量和函数使用 snake_case
let user_name = "Alice"
def calculate_total() { }

# 构造函数推荐使用驼峰或 new_ 前缀
def new_Point(x, y) { }
def NewPoint(x, y) { }

# 常量使用大写
let MAX_SIZE = 100
```

### 2. 代码组织

```python
# 将相关功能组织在一起
let MathUtils = ${
    PI: 3.14159,
    square: def(x) { x * x },
    cube: def(x) { x * x * x }
}
```

### 3. 错误处理

```python
# 对可能失败的操作使用 try-catch
try {
    risky_operation()
} catch error {
    println("Error: " + error)
}
```

### 4. 使用 Finally 清理资源

```python
try {
    # 执行操作
    process_data()
} catch error {
    println("Error: " + error)
} finally {
    # 总是清理资源
    println("Cleanup done")
}
```

---

## 常见问题

### Q: Chen Lang 是静态类型还是动态类型?

A: Chen Lang 是动态类型语言,变量的类型在运行时确定。

### Q: 如何处理浮点数精度问题?

A: Chen Lang 使用 Decimal 类型存储浮点数,避免了常见的浮点精度问题。例如
`0.1 + 0.2` 的结果是精确的 `0.3`。

### Q: 支持类和继承吗?

A: Chen Lang 使用基于原型的对象系统,通过元表的 `__index` 实现类似继承的功能。

### Q: 如何调试程序?

A: 使用 `println()` 输出调试信息,查看错误消息中的行号定位问题。

### Q: 如何遍历数组?

A: 推荐使用 `for...in` 语法直接遍历:

```python
let arr = [1, 2, 3]
for x in arr {
    println(x)
}
```

如果需要索引，请使用 `:entries()`:

```python
for e in arr:entries() {
    println(e.key + ": " + e.value)
}
```

传统的索引遍历依然有效:

```python
let i = 0
for i < arr:len() {
    println(arr[i])
    i = i + 1
}
```

---

## 附录

### 关键字列表

| 关键字     | 说明           |
| ---------- | -------------- |
| `let`      | 变量声明       |
| `def`      | 函数定义       |
| `if`       | 条件语句       |
| `else`     | 否则分支       |
| `for`      | 循环           |
| `return`   | 返回值         |
| `break`    | 退出循环       |
| `continue` | 继续下一次迭代 |
| `try`      | 异常处理       |
| `catch`    | 捕获异常       |
| `finally`  | 最终执行       |
| `throw`    | 抛出异常       |
| `true`     | 布尔真值       |
| `false`    | 布尔假值       |
| `null`     | 空值           |

### 内置函数

| 函数                  | 说明           |
| --------------------- | -------------- |
| `print(...)`          | 打印(不换行)   |
| `println(...)`        | 打印(换行)     |
| `set_meta(obj, meta)` | 设置对象的元表 |
| `get_meta(obj)`       | 获取对象的元表 |

### 内置对象

| 对象   | 说明                                             |
| ------ | ------------------------------------------------ |
| `Date` | 日期时间对象,使用 `Date.new()` 创建              |
| `JSON` | JSON 序列化,提供 `stringify()` 和 `parse()` 方法 |

### 数组方法

| 方法              | 说明                                     |
| ----------------- | ---------------------------------------- |
| `arr:len()`       | 返回数组长度                             |
| `arr:push(value)` | 添加元素到末尾, 返回新长度               |
| `arr:pop()`       | 移除并返回最后一个元素                   |
| `arr:iter()`      | 返回一个仅产生值的迭代器 (用于 `for-in`) |
| `arr:entries()`   | 返回一个产生 `{key, value}` 对象的迭代器 |

### 字符串方法

| 方法          | 说明                                         |
| ------------- | -------------------------------------------- |
| `str:len()`   | 返回字符串长度                               |
| `str:upper()` | 转换为大写                                   |
| `str:lower()` | 转换为小写                                   |
| `str:trim()`  | 去除首尾空白                                 |
| `str:iter()`  | 返回一个产生每个字符的迭代器 (用于 `for-in`) |

### 对象方法

| 方法            | 说明                                     |
| --------------- | ---------------------------------------- |
| `obj:keys()`    | 返回对象所有键名组成的数组               |
| `obj:iter()`    | 返回一个仅产生值的迭代器 (用于 `for-in`) |
| `obj:entries()` | 返回一个产生 `{key, value}` 对象的迭代器 |

### 协程 (Coroutine)

协程是 Chen Lang 处理异步和迭代的核心。

| 方法          | 说明                                          |
| ------------- | --------------------------------------------- |
| `co:resume()` | 恢复运行协程                                  |
| `co:status()` | 返回协程状态 ("suspended", "running", "dead") |
| `co:iter()`   | 返回协程自身, 以便直接在 `for-in` 中使用      |

---

## 当前限制

以下功能目前尚未支持:

- ❌ **闭包** - 内部函数无法捕获外部作用域的变量

---

**祝你学习愉快!** 🎉

如有问题,请参考示例代码或查看项目文档。
