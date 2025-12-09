
def point_str(self) {
    return "(" + self.x + "," + self.y + ")"
}
def NewPoint(x, y) {


    # 1. 定义方法 (通常这些放在外面作为公共原型)
    let methods = #{
        str: point_str
    }
    
    # 2. 创建实例
    let instance = #{ x: x, y: y }
    
    # 3. 建立继承关系
    set_meta(instance, #{ __index: methods })
    
    return instance
}

let p = NewPoint(10, 20)
println(p.str()) # 像调用对象方法一样