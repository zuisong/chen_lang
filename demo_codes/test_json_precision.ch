let io = import "stdlib/io"
let print = io.print
let println = io.println
let JSON = import "stdlib/json"

# StdLib: JSON Processing
let data = ${
    name: "Chen Lang",
    features: ["Simple", "Dynamic", "Rust-based"],
    version: 0.1+2
}

let jsonStr = JSON.stringify(data)
println("Serialized JSON:")
println(jsonStr)

let parsed = JSON.parse(jsonStr)
println("Parsed JSON Name: " + parsed.name)
println("Parsed JSON Version: " + parsed.version)

# Test more decimal precision cases
let test_cases = ${
    simple_add: 0.1 + 0.2,
    int_float_add: 1 + 0.5,
    complex: 3.14159 * 2
}

println("\nTest Cases:")
println(JSON.stringify(test_cases))
