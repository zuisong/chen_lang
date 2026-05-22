// Nested try-catch
try {
    console.log("Outer try")

    try {
        console.log("Inner try")
        throw "Inner error"
    } catch (inner_error) {
        console.log("Inner catch: " + inner_error)
        throw "Outer error"
    }

    console.log("This should not print")
} catch (outer_error) {
    console.log("Outer catch: " + outer_error)
}

console.log("Done")
