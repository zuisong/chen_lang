# 简单的并发测试 - 不使用 HTTP
let coroutine = Chen.coroutine
console.log("测试并发协程...")

function task(name, count) {
    let i = 0
    while (i < count) {
        console.log(name + ": " + i)
        i = i + 1
    }
    return name + " done"
}

# 创建协程
let co1 = coroutine.create(function() { return task("Task1", 3) })
let co2 = coroutine.create(function() { return task("Task2", 3) })

console.log("启动协程...")

# 非阻塞启动
coroutine.spawn(co1)
coroutine.spawn(co2)

console.log("等待完成...")

# 等待所有协程完成
let results = coroutine.await_all([co1, co2])

console.log("完成！")
console.log("结果 1: " + results[0])
console.log("结果 2: " + results[1])
