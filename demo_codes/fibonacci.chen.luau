-- 用 chen_lang 打印斐波那契数列前三十个数
local n = 1
local i = 1
local j = 2
local tmp = 0
while n <= 30 do
   print(i)
   tmp = i
   i = j
   j = tmp + j
   n = n + 1
end
