local io = require("stdlib/io")
local JSON = require("stdlib/json")

local data = { name = "Chen", version = 0.1 }
local json_str = JSON.stringify(data)
io.println("JSON: " .. json_str)

local parsed = JSON.parse(json_str)
io.println("Name from JSON: " .. parsed.name)
