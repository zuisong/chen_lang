# Basic try-catch test
try {
    throw "Something went wrong!"
} catch error {
    println("Caught error: " + error)
}

println("Program continues...")
