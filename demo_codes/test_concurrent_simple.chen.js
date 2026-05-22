// 简单的并发测试 - 不使用 HTTP
console.log("测试并发协程...")

async function task(name, count) {
    let i = 0
    while (i < count) {
        console.log(name + ": " + i)
        i = i + 1
    }
    return name + " done"
}

async function main() {
    console.log("启动异步任务...")
    let p1 = task("Task1", 3)
    let p2 = task("Task2", 3)

    console.log("等待完成...")
    let results = await Promise.all([p1, p2])

    console.log("完成！")
    console.log("结果 1: " + results[0])
    console.log("结果 2: " + results[1])
}

main()
