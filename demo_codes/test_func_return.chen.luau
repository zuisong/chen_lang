local io = require("stdlib/io")
local print = io.print
local println = io.println

local function func()
    return 123
end
local x = 1
x = func()
println(x)
