// 并发 HTTP 请求示例
let request = Chen.http.request
console.log("测试并发 HTTP 请求...")

// 创建多个异步请求
async function fetch(url) {
    let resp = request("GET", url)
    return resp.status
}

async function main() {
    console.log("启动 3 个并发请求...")
    let p1 = fetch("https://httpbin.org/delay/1")
    let p2 = fetch("https://httpbin.org/delay/1")
    let p3 = fetch("https://httpbin.org/delay/1")

    console.log("等待所有请求完成...")
    let results = await Promise.all([p1, p2, p3])

    console.log("所有请求完成！")
    console.log("结果 1: " + results[0])
    console.log("结果 2: " + results[1])
    console.log("结果 3: " + results[2])
}

main()
