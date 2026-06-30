-- 1. Test condition-based loop (Go-style while)
local i = 0
while i < 5 do
    print("for i < 5: " + i)
    i = i + 1
end

-- 2. Test infinite loop with break
local k = 0
while true do
    if k >= 3 then
        break
    end
    print("infinite for k: " + k)
    k = k + 1
end

print("--- Array.iter() (Values only) ---")
local arr = [10, 20, 30]
for x in arr do
    print("arr x: " + x)
end

print("--- Array.entries() (Key-Value pairs) ---")
for entry in arr:entries() do
    print("index: " + entry.key + ", value: " + entry.value)
end

print("--- Object.iter() (Values only) ---")
local obj = { a = 1, b = 2 }
for v in obj do
    print("obj v: " + v)
end

print("--- Object.entries() (Key-Value pairs) ---")
for entry in obj:entries() do
    print("entry key: " + entry.key + ", value: " + entry.value)
end

print("--- Object.keys() ---")
local keys = obj:keys()
for k in keys do
    print("key: " + k)
end

print("--- String.iter() ---")
local s = "ABC"
for char in s do
    print("char: " + char)
end
