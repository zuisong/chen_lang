local io = require("stdlib/io")

-- 1. Test condition-based loop (Go-style while)
local i = 0
while i < 5 do
    io.println("for i < 5: " + i)
    i = i + 1
end

-- 2. Test infinite loop with break
local k = 0
while true do
    if k >= 3 then
        break
    end
    io.println("infinite for k: " + k)
    k = k + 1
end

io.println("--- Array.iter() (Values only) ---")
local arr = [10, 20, 30]
for x in arr do
    io.println("arr x: " + x)
end

io.println("--- Array.entries() (Key-Value pairs) ---")
for entry in arr:entries() do
    io.println("index: " + entry.key + ", value: " + entry.value)
end

io.println("--- Object.iter() (Values only) ---")
local obj = { a = 1, b = 2 }
for v in obj do
    io.println("obj v: " + v)
end

io.println("--- Object.entries() (Key-Value pairs) ---")
for entry in obj:entries() do
    io.println("entry key: " + entry.key + ", value: " + entry.value)
end

io.println("--- Object.keys() ---")
local keys = obj:keys()
for k in keys do
    io.println("key: " + k)
end

io.println("--- String.iter() ---")
local s = "ABC"
for char in s do
    io.println("char: " + char)
end
