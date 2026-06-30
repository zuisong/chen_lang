-- 简单的并发测试 - 不使用 HTTP
local println = require("stdlib/io").println

println("测试并发协程...")

local function task(name, count)
    local i = 0
    while i < count do
        println(name + ": " + i)
        i = i + 1
    end
    return name + " done"
end

-- 创建协程
local co1 = coroutine.create(function() task("Task1", 3) end)
local co2 = coroutine.create(function() task("Task2", 3) end)

println("启动协程...")

-- 非阻塞启动
coroutine.spawn(co1)
coroutine.spawn(co2)

println("等待完成...")

-- 等待所有协程完成
local results = coroutine.await_all([co1, co2])

println("完成！")
println("结果 1: " + results[0])
println("结果 2: " + results[1])
