# StdLib: JSON Processing
let data = {
    name: "Chen Lang",
    features: ["Simple", "Dynamic", "Rust-based"],
    version: 0.1+2
}

let jsonStr = JSON.stringify(data)
console.log("Serialized JSON:")
console.log(jsonStr)

let parsed = JSON.parse(jsonStr)
console.log("Parsed JSON Name: " + parsed.name)
console.log("Parsed JSON Version: " + parsed.version)

# Test more decimal precision cases
let test_cases = {
    simple_add: 0.1 + 0.2,
    int_float_add: 1 + 0.5,
    complex: 3.14159 * 2
}

console.log("\nTest Cases:")
console.log(JSON.stringify(test_cases))
