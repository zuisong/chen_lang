# chen_lang

## 一个用 [Rust](https://www.rust-lang.org) 编写的小型、面向表达式的编程语言

`chen_lang` 是一个为学习和实验而设计的动态解释型编程语言。它拥有简洁的语法、基于表达式的控制流以及一等函数支持。

---

### ✨ 主要特性

*   **面向表达式**：`if/else` 块和代码块都是表达式，并且会返回值。
*   **动态类型**：支持整数 (Integer)、浮点数 (Float)、字符串 (String)、布尔值 (Boolean) 和空值 (Null)。
*   **函数**：支持一等函数和隐式返回。
*   **控制流**：`if/else` 表达式、`for` 循环、`break` 和 `continue`。
*   **对象系统**：支持对象字面量、属性访问、索引访问和元表 (Metatable)。
*   **模块系统**：支持导入标准库和自定义模块。
*   **作用域隔离**：支持块级作用域变量。


---

### 🚀 快速开始

#### 1. Hello World 与字符串操作
```python
let io = import("stdlib/io")
let name = "Chen Lang"
io.println("Hello, " + name + "!")
# 输出: Hello, Chen Lang!
```

#### 2. If/Else 作为表达式
在 `chen_lang` 中，`if/else` 是一个表达式，这意味着它会返回一个值。
```python
let a = 10
let result = if a > 5 {
    "大于"
} else {
    "小于"
}
println(result) 
# 输出: 大于
```

#### 3. 函数与隐式返回
函数会隐式返回最后一个表达式的值。
```python
def add(a, b) {
    a + b  # 隐式返回
}

let sum = add(3, 5)
println(sum)
# 输出: 8
```

#### 4. 递归斐波那契
```python
def fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

println(fib(10))
# 输出: 55
```

#### 5. 循环与逻辑
```python
# 打印乘法表
let i = 1
for i <= 9 {
    let j = 1
    for j <= i {
        let prod = i * j
        print(j + "x" + i + "=" + prod + " ")
        j = j + 1
    }
    println("")
    i = i + 1
}
```

#### 6. 对象系统
```python
# 对象字面量
let person = ${ 
    name: "张三", 
    age: 25 
}

# 属性访问
println(person.name)  # 输出: 张三

# 属性赋值
person.age = 26

# 索引访问
let key = "name"
println(person[key])  # 输出: 张三
```

---

### 🛠️ 语言参考

#### 数据类型
*   **整数 (Integer)**: `1`, `42`, `-10`
*   **浮点数 (Float)**: `3.14`, `-0.01`
*   **字符串 (String)**: `"Hello World"`
*   **布尔值 (Boolean)**: `true`, `false`
*   **空值 (Null)**: `null` (空块或没有 else 的 if 语句的隐式返回值)

#### 运算符
*   **算术**: `+`, `-`, `*`, `/`, `%`
*   **比较**: `>`, `>=`, `<`, `<=`, `==`, `!=`
*   **逻辑**: `&&`, `||`, `!`

#### 注释
```python
# 这是一个单行注释
```

---

### 📝 TODO / 路线图

*   [x] **核心**: 整数, 布尔值, 算术运算, 逻辑运算
*   [x] **控制流**: `if/else` (表达式), `for` 循环, `break`, `continue`
*   [x] **函数**: 定义, 调用, 递归, 隐式返回
*   [x] **类型**: 浮点数, 字符串
*   [x] **对象系统**: 对象字面量, 属性访问, 索引访问, 元表（可用作数组）
*   [x] **标准库**: 文件 I/O, 数学函数
*   [x] **错误处理**: Try/Catch 或 Result 类型
*   [x] **闭包**: 捕获外部作用域变量


---

### 📄 许可证

MIT License
