function NewPoint(x, y) {
    // 嵌套定义函数
    function point_str() {
        return "(" + this.x + "," + this.y + "," + this.now.format('%Y-%m-%d %H:%M:%S') + ")"
    }

    let methods = {
        str: point_str
    }
    let d = Chen.date.new()
    console.log(d.__type)
    console.log(d.format('%Y'))

    let instance = { x: x, y: y, now: d }
    Chen.setMeta(instance, { __index: methods })

    return instance
}

let p = NewPoint(10, 20)
console.log(p.str()) // 像调用对象方法一样
console.log(JSON.stringify(p))
