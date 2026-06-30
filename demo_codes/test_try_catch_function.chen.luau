-- Function with exception
local function divide(a, b)
    if b == 0 then
        error("Division by zero")
    end
    a / b
end

try
    local result = divide(10, 2)
    print("Result: " + result)

    local bad_result = divide(10, 0)
    print("This should not print")
catch error
    print("Caught: " + error)
end

print("Program completed")
