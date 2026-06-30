local io = require("stdlib/io")
local print = io.print
local println = io.println

-- Basic try-catch test
try
    error("Something went wrong!")
catch error
    println("Caught error: " + error)
end

println("Program continues...")
