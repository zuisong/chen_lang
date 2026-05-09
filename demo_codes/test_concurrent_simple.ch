# 简单的并发测试 - 不使用 HTTP
let println = import("stdlib/io").println

println("测试并发协程...")

def task(name, count) {
    let i = 0
    for i < count {
        println(name + ": " + i)
        i = i + 1
    }
    return name + " done"
}

# 创建协程
let co1 = coroutine.create(def() { task("Task1", 3) })
let co2 = coroutine.create(def() { task("Task2", 3) })

println("启动协程...")

# 非阻塞启动
coroutine.spawn(co1)
coroutine.spawn(co2)

println("等待完成...")

# 等待所有协程完成
let results = coroutine.await_all([co1, co2])

println("完成！")
println("结果 1: " + results[0])
println("结果 2: " + results[1])
