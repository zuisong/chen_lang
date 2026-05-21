# Function with exception
function divide(a, b) {
    if (b == 0) {
        throw "Division by zero"
    }
    return a / b
}

try {
    let result = divide(10, 2)
    console.log("Result: " + result)

    let bad_result = divide(10, 0)
    console.log("This should not print")
} catch (error) {
    console.log("Caught: " + error)
}

console.log("Program completed")
