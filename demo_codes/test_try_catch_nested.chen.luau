local io = require("stdlib/io")
local print = io.print
local println = io.println

-- Nested try-catch
try
    println("Outer try")

    try
        println("Inner try")
        error("Inner error")
    catch inner_error
        println("Inner catch: " + inner_error)
        error("Outer error")
    end

    println("This should not print")
catch outer_error
    println("Outer catch: " + outer_error)
end

println("Done")
