# 这里是注释,
# 注释以# 开始, 到行尾结束
# if 和 for 里面的表达式运算结果都是bool类型
let i=1
for i<=9 {
    let j = 1
    for j<=i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}

