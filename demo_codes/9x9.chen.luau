-- 这里是注释,
-- 注释以# 开始, 到行尾结束
-- if 和 for 里面的表达式运算结果都是bool类型
local i=1
while i<=9 do
    local j = 1
    while j <= i do
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    end
    print("")
    i=i+1
end
