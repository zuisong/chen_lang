
# chen_lang

## A tiny, expression-oriented programming language written in [Rust](https://www.rust-lang.org)

[🇨🇳 中文版](./readme_cn.md)

`chen_lang` is a dynamic, interpreted programming language designed for learning and experimentation. It features a clean syntax, expression-based control flow, and first-class functions.

---

### ✨ Key Features

*   **Expression-Oriented**: `if/else` blocks and code blocks are expressions that return values.
*   **Dynamic Typing**: Supports Integers, Floats, Strings, Booleans, and Null.
*   **Functions**: First-class functions with implicit returns.
*   **Control Flow**: `if/else` expressions, `for` loops, `break`, and `continue`.
*   **Object System**: Supports object literals, property access, indexing, and metatables.
*   **Module System**: Supports importing standard library and custom modules.
*   **Scope Isolation**: Block-scoped variables.


---

### 🚀 Quick Start

#### 1. Hello World & String Operations
```python
let io = import "stdlib/io"
let name = "Chen Lang"
io.println("Hello, " + name + "!")
# Output: Hello, Chen Lang!
```

#### 2. If/Else as Expression
In `chen_lang`, `if/else` is an expression, meaning it returns a value.
```python
let a = 10
let result = if a > 5 {
    "Greater"
} else {
    "Smaller"
}
println(result) 
# Output: Greater
```

#### 3. Functions & Implicit Return
Functions implicitly return the value of the last expression.
```python
def add(a, b) {
    a + b  # Implicit return
}

let sum = add(3, 5)
println(sum)
# Output: 8
```

#### 4. Recursive Fibonacci
```python
def fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

println(fib(10))
# Output: 55
```

#### 5. Loops & Logic
```python
# Print multiplication table
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

#### 6. Object System
```python
# Object literal
let person = ${ 
    name: "Alice", 
    age: 25 
}

# Property access
println(person.name)  # Output: Alice

# Property assignment
person.age = 26

# Index access
let key = "name"
println(person[key])  # Output: Alice
```

---

### 🛠️ Language Reference

#### Data Types
*   **Integer**: `1`, `42`, `-10`
*   **Float**: `3.14`, `-0.01`
*   **String**: `"Hello World"`
*   **Boolean**: `true`, `false`
*   **Null**: `null` (implicit return for empty blocks or if without else)

#### Operators
*   **Arithmetic**: `+`, `-`, `*`, `/`, `%`
*   **Comparison**: `>`, `>=`, `<`, `<=`, `==`, `!=`
*   **Logical**: `&&`, `||`, `!`

#### Comments
```python
# This is a single-line comment
```

---

### 📝 TODO / Roadmap

*   [x] **Core**: Integers, Booleans, Arithmetic, Logic
*   [x] **Control Flow**: `if/else` (expression), `for` loops, `break`, `continue`
*   [x] **Functions**: Definition, Call, Recursion, Implicit Return
*   [x] **Types**: Floats, Strings
*   [x] **Object System**: Object literals, property access, indexing, metatables (can be used as arrays)
*   [x] **Standard Library**: File I/O, Math functions
*   [x] **Error Handling**: Try/Catch or Result type
*   [ ] **Closures**: Capture variables from outer scopes


---

### 📄 License

MIT License
