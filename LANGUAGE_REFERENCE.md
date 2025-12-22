# Chen Lang 语言参考

**版本**: 0.1.0  
**更新日期**: 2025-12-10

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

### 快速开始

```python
# Hello World
println("Hello, Chen Lang!")

# 简单计算
let result = 10 + 20
println("Result: " + result)
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

使用 `#{}` 创建对象:

```python
let person = #{
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
    println(global_var)  # 可以访问全局变量
    println(local_var)   # 可以访问局部变量
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

Chen Lang 的 for 循环是条件循环:

```python
let i = 0
for i < 10 {
    println(i)
    i = i + 1
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
let person = #{
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

```python
let calculator = #{
    value: 0,
    add: def(self, n) {
        self.value = self.value + n
    },
    get: def(self) {
        self.value
    }
}

calculator.add(10)
calculator.add(5)
println(calculator.get())  # 15
```

### 元表 (Metatable)

元表用于实现高级特性,如运算符重载和方法查找:

```python
# 定义 Point 原型
let Point = #{
    __index: #{
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
    let instance = #{ x: x, y: y }
    set_meta(instance, Point)
    return instance
}

# 使用
let p1 = new_Point(10, 20)
let p2 = new_Point(5, 10)
let p3 = p1 + p2  # 使用重载的 + 运算符

println(p3.to_string())  # "Point(15, 30)"
```

### 元方法

支持的元方法:
- `__add` - 加法 (+)
- `__sub` - 减法 (-)
- `__mul` - 乘法 (*)
- `__index` - 属性查找

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
let length = arr.len()  # 3

# 添加元素
arr.push(4)  # 返回新长度 4, arr 变为 [1, 2, 3, 4]

# 弹出元素
let last = arr.pop()  # 返回 4, arr 变为 [1, 2, 3]

# 获取类型
println(arr.__type)  # "Array"
```

### 遍历数组

```python
let arr = [10, 20, 30]
let i = 0
for i < arr.len() {
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

## 标准库

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
let now = Date.new()

# 获取类型
println(now.__type)  # "Date"

# 格式化日期
let formatted = now.format('%Y-%m-%d %H:%M:%S')
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
let data = #{
    name: "Alice",
    age: 30,
    hobbies: ["reading", "coding"]
}
let json_str = JSON.stringify(data)
println(json_str)
# 输出: {"name":"Alice","age":30,"hobbies":["reading","coding"]}

# 解析 JSON
let parsed = JSON.parse(json_str)
println(parsed.name)  # "Alice"
```

### 字符串方法

```python
let text = "Hello, World!"

# 获取长度
let length = text.len()  # 13

# 转大写
let upper = text.upper()  # "HELLO, WORLD!"

# 转小写
let lower = text.lower()  # "hello, world!"

# 去除空白
let trimmed = "  hello  ".trim()  # "hello"

# 获取类型
println(text.__type)  # "String"
```

### 对象方法

```python
let obj = #{
    name: "Alice",
    age: 30,
    city: "Beijing"
}

# 获取所有键
let keys = obj.keys()  # ["name", "age", "city"]
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
let calculator = #{
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
let Point = #{
    __index: #{
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
    let instance = #{ x: x, y: y }
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
let MathUtils = #{
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
A: Chen Lang 使用 Decimal 类型存储浮点数,避免了常见的浮点精度问题。例如 `0.1 + 0.2` 的结果是精确的 `0.3`。

### Q: 支持类和继承吗?
A: Chen Lang 使用基于原型的对象系统,通过元表的 `__index` 实现类似继承的功能。

### Q: 如何调试程序?
A: 使用 `println()` 输出调试信息,查看错误消息中的行号定位问题。

### Q: 如何遍历数组?
A: 使用 for 循环配合 `len()` 方法:
```python
let arr = [1, 2, 3]
let i = 0
for i < arr.len() {
    println(arr[i])
    i = i + 1
}
```

---

## 附录

### 关键字列表

| 关键字 | 说明 |
|--------|------|
| `let` | 变量声明 |
| `def` | 函数定义 |
| `if` | 条件语句 |
| `else` | 否则分支 |
| `for` | 循环 |
| `return` | 返回值 |
| `break` | 退出循环 |
| `continue` | 继续下一次迭代 |
| `try` | 异常处理 |
| `catch` | 捕获异常 |
| `finally` | 最终执行 |
| `throw` | 抛出异常 |
| `true` | 布尔真值 |
| `false` | 布尔假值 |
| `null` | 空值 |

### 内置函数

| 函数 | 说明 |
|------|------|
| `print(...)` | 打印(不换行) |
| `println(...)` | 打印(换行) |
| `set_meta(obj, meta)` | 设置对象的元表 |
| `get_meta(obj)` | 获取对象的元表 |

### 内置对象

| 对象 | 说明 |
|------|------|
| `Date` | 日期时间对象,使用 `Date.new()` 创建 |
| `JSON` | JSON 序列化,提供 `stringify()` 和 `parse()` 方法 |

### 数组方法

| 方法 | 说明 |
|------|------|
| `arr.len()` | 返回数组长度 |
| `arr.push(value)` | 添加元素到末尾,返回新长度 |
| `arr.pop()` | 移除并返回最后一个元素 |

### 字符串方法

| 方法 | 说明 |
|------|------|
| `str.len()` | 返回字符串长度 |
| `str.upper()` | 转换为大写 |
| `str.lower()` | 转换为小写 |
| `str.trim()` | 去除首尾空白 |

---

## 当前限制

以下功能目前尚未支持:

- ❌ **闭包** - 内部函数无法捕获外部作用域的变量
- ❌ **模块系统** - 无法导入外部文件
- ❌ **标准输入** - 无法读取用户输入

---

**祝你学习愉快!** 🎉

如有问题,请参考示例代码或查看项目文档。
