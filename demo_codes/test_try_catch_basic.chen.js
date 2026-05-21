# Basic try-catch test
try {
    throw "Something went wrong!"
} catch (error) {
    console.log("Caught error: " + error)
}

console.log("Program continues...")
