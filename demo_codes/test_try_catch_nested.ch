let io = import("stdlib/io")
let print = io.print
let println = io.println

# Nested try-catch
try {
    println("Outer try")

    try {
        println("Inner try")
        throw "Inner error"
    } catch inner_error {
        println("Inner catch: " + inner_error)
        throw "Outer error"
    }

    println("This should not print")
} catch outer_error {
    println("Outer catch: " + outer_error)
}

println("Done")
