let io = import "stdlib/io"
let print = io.print
let println = io.println
let JSON = import "stdlib/json"
let Date = import "stdlib/date"


def NewPoint(x, y) {
    # 嵌套定义函数
    def point_str(self) {
        return "(" + self.x + "," + self.y + "," + self.now:format('%Y-%m-%d %H:%M:%S') + ")"
    }

    let methods = ${
        str: point_str
    }
    let d = Date:new()
    println(d.__type)
    println(d:format('%Y'))

    let instance = ${ x: x, y: y, now: d }
    set_meta(instance, ${ __index: methods })

    return instance
}

let p = NewPoint(10, 20)
println(p:str()) # 像调用对象方法一样
println(JSON.stringify(p))
