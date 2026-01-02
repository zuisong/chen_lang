let println = import "stdlib/io".println

# 简单的字符串重复函数
def repeat(str, count) {
    let res = ""
    let i = 0
    for i < count {
        res = res + str
        i = i + 1
    }
    return res
}

def print_tree(height) {
    println("🎄 Merry Christmas! 🎄")
    println("")

    # 打印树冠
    let i = 1
    for i <= height {
        let spaces = repeat(" ", height - i)
        # 装饰一点彩灯（简单随机模拟：用不同字符？）
        # 这里仅用星星
        let stars = repeat("*", 2 * i - 1)
        println(spaces + stars)
        i = i + 1
    }

    # 打印树干
    let trunk_padding = repeat(" ", height - 2)
    
    let j = 0
    for j < 2 {
        println(trunk_padding + "###")
        j = j + 1
    }
    
    println("")
    println(repeat(" ", height - 4) + "Happy New Year!")
}

print_tree(10)
