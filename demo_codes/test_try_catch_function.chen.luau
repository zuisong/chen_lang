local io = require("stdlib/io")
local print = io.print
local println = io.println

-- Function with exception
local function divide(a, b)
    if b == 0 then
        error("Division by zero")
    end
    a / b
end

try
    local result = divide(10, 2)
    println("Result: " + result)

    local bad_result = divide(10, 0)
    println("This should not print")
catch error
    println("Caught: " + error)
end

println("Program completed")
