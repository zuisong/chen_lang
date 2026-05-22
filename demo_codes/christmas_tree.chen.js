// 简单的字符串重复函数
function repeat(str, count) {
    let res = ""
    let i = 0
    while (i < count) {
        res = res + str
        i = i + 1
    }
    return res
}

function print_tree(height) {
    console.log("🎄 Merry Christmas! 🎄")
    console.log("")

    // 打印树冠
    let i = 1
    while (i <= height) {
        let spaces = repeat(" ", height - i)
        // 装饰一点彩灯（简单随机模拟：用不同字符？）
        // 这里仅用星星
        let stars = repeat("*", 2 * i - 1)
        console.log(spaces + stars)
        i = i + 1
    }

    // 打印树干
    let trunk_padding = repeat(" ", height - 2)
    
    let j = 0
    while (j < 2) {
        console.log(trunk_padding + "###")
        j = j + 1
    }
    
    console.log("")
    console.log(repeat(" ", height - 4) + "Happy New Year!")
}

print_tree(10)
