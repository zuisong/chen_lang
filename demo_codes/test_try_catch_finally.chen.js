# Try-catch with finally
let cleanup_called = false

try {
    console.log("In try block")
    throw "Error occurred"
} catch (error) {
    console.log("In catch block: " + error)
} finally {
    console.log("In finally block")
    cleanup_called = true
}

console.log("Cleanup called: " + cleanup_called)
