-- 简单的字符串重复函数
local function repeat_str(str, count)
    local res = ""
    local i = 0
    while i < count do
        res = res + str
        i = i + 1
    end
    return res
end

local function print_tree(height)
    print("🎄 Merry Christmas! 🎄")
    print("")

    -- 打印树冠
    local i = 1
    while i <= height do
        local spaces = repeat_str(" ", height - i)
        local stars = repeat_str("*", 2 * i - 1)
        print(spaces + stars)
        i = i + 1
    end

    -- 打印树干
    local trunk_padding = repeat_str(" ", height - 2)
    
    local j = 0
    while j < 2 do
        print(trunk_padding + "###")
        j = j + 1
    end
    
    print("")
    print(repeat_str(" ", height - 4) + "Happy New Year!")
end

print_tree(10)
