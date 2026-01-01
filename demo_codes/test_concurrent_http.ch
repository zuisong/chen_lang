# 并发 HTTP 请求示例
let http = import "stdlib/http"
let json = import "stdlib/json"
let println = import "stdlib/io".println

println("测试并发 HTTP 请求...")

# 创建多个协程，每个发送一个 HTTP 请求
def fetch(url) {
    let resp = http.request("GET", url)
    return resp.status
}

# 创建协程
let co1 = coroutine.create(def() { fetch("https://httpbin.org/delay/1") })
let co2 = coroutine.create(def() { fetch("https://httpbin.org/delay/1") })
let co3 = coroutine.create(def() { fetch("https://httpbin.org/delay/1") })

println("启动 3 个并发请求...")

# 非阻塞启动所有协程
coroutine.spawn(co1)
coroutine.spawn(co2)
coroutine.spawn(co3)

println("等待所有请求完成...")

# 等待所有协程完成
let results = coroutine.await_all([co1, co2, co3])

println("所有请求完成！")
println("结果 1: " + results[0])
println("结果 2: " + results[1])
println("结果 3: " + results[2])
