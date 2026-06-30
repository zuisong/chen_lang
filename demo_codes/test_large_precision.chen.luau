local io = require("stdlib/io")
local print = io.print
local println = io.println
local JSON = require("stdlib/json")

local data = {
    pi = 3.141592653589793,
    e = 2.718281828459045,
    small = 0.000000001
}
local json_str = JSON.stringify(data)
println(json_str)
