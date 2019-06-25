# 用 chen_lang 打印斐波那契数列前三十个数
n = 1
i = 1
j = 2
for n <= 30 {
   println(i)
   tmp = i
   i = j
   j = tmp + j
   n = n + 1
}