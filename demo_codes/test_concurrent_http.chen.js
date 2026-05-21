# 并发 HTTP 请求示例
let coroutine = Chen.coroutine
let request = Chen.http.request
console.log("测试并发 HTTP 请求...")

# 创建多个协程，每个发送一个 HTTP 请求
function fetch(url) {
    let resp = request("GET", url)
    return resp.status
}

# 创建协程
let co1 = coroutine.create(function() { return fetch("https://httpbin.org/delay/1") })
let co2 = coroutine.create(function() { return fetch("https://httpbin.org/delay/1") })
let co3 = coroutine.create(function() { return fetch("https://httpbin.org/delay/1") })

console.log("启动 3 个并发请求...")

# 非阻塞启动所有协程
coroutine.spawn(co1)
coroutine.spawn(co2)
coroutine.spawn(co3)

console.log("等待所有请求完成...")

# 等待所有协程完成
let results = coroutine.await_all([co1, co2, co3])

console.log("所有请求完成！")
console.log("结果 1: " + results[0])
console.log("结果 2: " + results[1])
console.log("结果 3: " + results[2])
