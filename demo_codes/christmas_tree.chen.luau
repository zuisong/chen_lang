local println = require("stdlib/io").println

-- 简单的字符串重复函数
local function repeat(str, count)
    local res = ""
    local i = 0
    while i < count do
        res = res + str
        i = i + 1
    end
    return res
end

local function print_tree(height)
    println("🎄 Merry Christmas! 🎄")
    println("")

    -- 打印树冠
    local i = 1
    while i <= height do
        local spaces = repeat(" ", height - i)
        local stars = repeat("*", 2 * i - 1)
        println(spaces + stars)
        i = i + 1
    end

    -- 打印树干
    local trunk_padding = repeat(" ", height - 2)
    
    local j = 0
    while j < 2 do
        println(trunk_padding + "###")
        j = j + 1
    end
    
    println("")
    println(repeat(" ", height - 4) + "Happy New Year!")
end

print_tree(10)
