import stdlib/io
let print = io.print
let println = io.println

# Function with exception
def divide(a, b) {
    if b == 0 {
        throw "Division by zero"
    }
    a / b
}

try {
    let result = divide(10, 2)
    println("Result: " + result)
    
    let bad_result = divide(10, 0)
    println("This should not print")
} catch error {
    println("Caught: " + error)
}

println("Program completed")
