local io = require("stdlib/io")
local print = io.print
local println = io.println
local JSON = require("stdlib/json")

-- StdLib: JSON Processing
local data = {
    name = "Chen Lang",
    features = ["Simple", "Dynamic", "Rust-based"],
    version = 0.1+2
}

local jsonStr = JSON.stringify(data)
println("Serialized JSON:")
println(jsonStr)

local parsed = JSON.parse(jsonStr)
println("Parsed JSON Name: " + parsed.name)
println("Parsed JSON Version: " + parsed.version)

-- Test more decimal precision cases
local test_cases = {
    simple_add = 0.1 + 0.2,
    int_float_add = 1 + 0.5,
    complex = 3.14159 * 2
}

println("\nTest Cases:")
println(JSON.stringify(test_cases))
