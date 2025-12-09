
def NewPoint(x, y) {
    # 嵌套定义函数
    def point_str(self) {
        return "(" + self.x + "," + self.y + ")"
    }

    let methods = #{
        str: point_str
    }
    
    let instance = #{ x: x, y: y }
    set_meta(instance, #{ __index: methods })
    
    return instance
}

let p = NewPoint(10, 20)
println(p.str()) # 像调用对象方法一样