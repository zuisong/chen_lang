local io = require("stdlib/io")
local print = io.print
local println = io.println

-- Try-catch with finally
local cleanup_called = false

try
    println("In try block")
    error("Error occurred")
catch error
    println("In catch block: " + error)
finally
    println("In finally block")
    cleanup_called = true
end

println("Cleanup called: " + cleanup_called)
