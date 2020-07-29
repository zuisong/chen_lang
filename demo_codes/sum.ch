let i = 100
let sum = 0
for i!=0 {
    i = i - 1
    # 这里有相对复杂的逻辑运算
    if (i%2!=0) || (i%3==0)  {
        println(i)
        # 打印出来的 i 都是奇数 或者是能被三整除的偶数
        sum = sum + i
    }
}
# sum 为 100以为的奇数之和
println("100以内的 奇数或者是能被三整除的偶数 之和是")
println(sum)
