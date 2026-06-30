local JSON = require("stdlib/json")

local data = {
    pi = 3.141592653589793,
    e = 2.718281828459045,
    small = 0.000000001
}
local json_str = JSON.stringify(data)
print(json_str)
