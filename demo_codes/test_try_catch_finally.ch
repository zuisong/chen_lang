import stdlib/io
let print = io.print
let println = io.println

# Try-catch with finally
let cleanup_called = false

try {
    println("In try block")
    throw "Error occurred"
} catch error {
    println("In catch block: " + error)
} finally {
    println("In finally block")
    cleanup_called = true
}

println("Cleanup called: " + cleanup_called)
