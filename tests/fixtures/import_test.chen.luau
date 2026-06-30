local io = require("stdlib/io")
local JSON = require("stdlib/json")

local data = { name = "Chen", version = 0.1 }
local json_str = JSON.stringify(data)
io.write("JSON: " .. json_str)

local parsed = JSON.parse(json_str)
io.write("Name from JSON: " .. parsed.name)
