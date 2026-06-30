local obj = { a = 1, b = 2 }
local keys = obj:keys()
print("Object keys:", keys)
print("Keys length:", #keys)

local i = 0
while i < #keys do
    local key = keys[i]
    print("Key:", key, "Value:", obj[key])
    i = i + 1
end

local arr = [10, 20]
local arr_keys = arr:keys()
print("Array keys:", arr_keys)
