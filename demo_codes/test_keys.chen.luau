local io = require("stdlib/io")
local print = io.print
local println = io.println

local obj = { a = 1, b = 2 }
local keys = obj:keys()
println("Object keys:", keys)
println("Keys length:", #keys)

local i = 0
while i < #keys do
    local key = keys[i]
    println("Key:", key, "Value:", obj[key])
    i = i + 1
end

local arr = [10, 20]
local arr_keys = arr:keys()
println("Array keys:", arr_keys)
