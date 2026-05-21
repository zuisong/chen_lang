# 这里是注释
# 注释以# 开始, 到行尾结束
let i = 1
while (i <= 9) {
    let j = 1
    while (j <= i) {
        console.print(j + "x" + i + "=" + i * j + " ")
        j = j + 1
    }
    console.log("")
    i = i + 1
}
