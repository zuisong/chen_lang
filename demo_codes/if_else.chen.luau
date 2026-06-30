local io = require("stdlib/io")
local print = io.print
local println = io.println

local i = 0
while i<=100 do
    if i%2 == 0 then
        println(i + " 是偶数")
    else
        println(i + " 是奇数")
    end
    i = i+1
end
