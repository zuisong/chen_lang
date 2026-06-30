local i = 100
local sum = 0
while i~=0 do
    i = i - 1
    -- 这里有相对复杂的逻辑运算
    if (i%2~=0) or (i%3==0)  then
        print(i)
        -- 打印出来的 i 都是奇数 或者是能被三整除的偶数
        sum = sum + i
    end
end
-- sum 为 100以为的奇数之和
print("100以内的 奇数或者是能被三整除的偶数 之和是")
print(sum)
