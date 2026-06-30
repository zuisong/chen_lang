local JSON = require("stdlib/json")
local Date = require("stdlib/date")


local function NewPoint(x, y)
    -- 嵌套定义函数
    local function point_str(self)
        return "(" + self.x + "," + self.y + "," + self.now:format('%Y-%m-%d %H:%M:%S') + ")"
    end

    local methods = {
        str = point_str
    }
    local d = Date:new()
    print(d.__type)
    print(d:format('%Y'))

    local instance = { x = x, y = y, now = d }
    set_meta(instance, { __index = methods })

    return instance
end

local p = NewPoint(10, 20)
print(p:str()) -- 像调用对象方法一样
print(JSON.stringify(p))
