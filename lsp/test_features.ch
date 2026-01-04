# Chen Lang LSP 测试文件
# 用于测试各种 LSP 功能

# 测试 1: 变量定义和引用
let x = 10
let y = 20
let result = x + y  # x 和 y 应该可以跳转到定义

# 测试 2: 函数定义和调用
def add(a, b) {
    a + b
}

def multiply(a, b) {
    a * b
}

# 测试函数调用 - 应该可以跳转到函数定义
let sum = add(x, y)
let product = multiply(x, y)

# 测试 3: 递归函数
def factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let fact5 = factorial(5)

# 测试 4: 闭包
def make_counter() {
    let count = 0
    def increment() {
        count = count + 1
        count
    }
    increment
}

let counter = make_counter()

# 测试 5: 对象
let person = ${
    name: "张三",
    age: 25,
    greet: def() {
        println("你好，我是" + person.name)
    }
}

# 测试 6: 循环
def print_numbers(n) {
    let i = 1
    for i <= n {
        println(i)
        i = i + 1
    }
}

# 测试 7: 错误处理
def safe_divide(a, b) {
    try {
        a / b
    } catch error {
        println("错误: " + error)
        0
    }
}

# 测试 8: 模块导入
let io = import "stdlib/io"
io.println("Hello from LSP test!")

# 测试 9: 多次使用同一变量（测试查找引用）
let test_var = 100
let test_result1 = test_var + 1
let test_result2 = test_var * 2
let test_result3 = test_var - 50

# 测试 10: 嵌套函数
def outer(x) {
    def inner(y) {
        x + y
    }
    inner
}

let add5 = outer(5)
let result10 = add5(3)

# 故意的语法错误（用于测试诊断功能）
# 取消下面的注释来测试错误检测
# def broken_function( {
#     let x =
# }
