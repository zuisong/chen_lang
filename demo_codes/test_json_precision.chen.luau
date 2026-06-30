local JSON = require("stdlib/json")

-- StdLib: JSON Processing
local data = {
    name = "Chen Lang",
    features = ["Simple", "Dynamic", "Rust-based"],
    version = 0.1+2
}

local jsonStr = JSON.stringify(data)
print("Serialized JSON:")
print(jsonStr)

local parsed = JSON.parse(jsonStr)
print("Parsed JSON Name: " + parsed.name)
print("Parsed JSON Version: " + parsed.version)

-- Test more decimal precision cases
local test_cases = {
    simple_add = 0.1 + 0.2,
    int_float_add = 1 + 0.5,
    complex = 3.14159 * 2
}

print("\nTest Cases:")
print(JSON.stringify(test_cases))
