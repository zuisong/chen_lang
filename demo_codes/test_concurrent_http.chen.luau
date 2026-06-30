-- 并发 HTTP 请求示例
local http = require("stdlib/http")
local json = require("stdlib/json")
local println = require("stdlib/io").println

println("测试并发 HTTP 请求...")

-- 创建多个协程，每个发送一个 HTTP 请求
local function fetch(url)
    local resp = http.request("GET", url)
    return resp.status
end

-- 创建协程
local co1 = coroutine.create(function() fetch("https://httpbin.org/delay/1") end)
local co2 = coroutine.create(function() fetch("https://httpbin.org/delay/1") end)
local co3 = coroutine.create(function() fetch("https://httpbin.org/delay/1") end)

println("启动 3 个并发请求...")

-- 非阻塞启动所有协程
coroutine.spawn(co1)
coroutine.spawn(co2)
coroutine.spawn(co3)

println("等待所有请求完成...")

-- 等待所有协程完成
local results = coroutine.await_all([co1, co2, co3])

println("所有请求完成！")
println("结果 1: " + results[0])
println("结果 2: " + results[1])
println("结果 3: " + results[2])
