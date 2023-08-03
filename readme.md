
# chen_lang

## A tiny programming language written in [Rust](https://www.rust-lang.org)

[🇨🇳 中文版](./readme_cn.md)

---

### Sample Code

```
# This is a comment 
# Comments start with #, and go to the end of the line
# The expressions in if and for evaluate to bool
let i=1
for i<=9 {
    let j = 1
    for j<=i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}
```

Output:

```
1x1=1  
1x2=2 2x2=4
1x3=3 2x3=6 3x3=9
1x4=4 2x4=8 3x4=12 4x4=16
1x5=5 2x5=10 3x5=15 4x5=20 5x5=25
1x6=6 2x6=12 3x6=18 4x6=24 5x6=30 6x6=36
1x7=7 2x7=14 3x7=21 4x7=28 5x7=35 6x7=42 7x7=49
1x8=8 2x8=16 3x8=24 4x8=32 5x8=40 6x8=48 7x8=56 8x8=64  
1x9=9 2x9=18 3x9=27 4x9=36 5x9=45 6x9=54 7x9=63 8x9=72 9x9=81
```

---

```
let i = 100
let sum = 0 
for i!=0 {
    i = i - 1
    # Here is relatively complex logical operation
    if (i%2!=0) || (i%3==0) {
        # println(i)
        # Uncomment the line above
        # Print out all odd numbers or even numbers divisible by 3
        sum = sum + i
    }
}
println("The sum of odd numbers or even numbers divisible by 3 below 100 is")  
println(sum)
```

Output:

```
The sum of odd numbers or even numbers divisible by 3 below 100 is
3316
```

---

```
# Use chen_lang to print the first 30 Fibonacci numbers 
let n = 1
let i = 1
let j = 2
println("Print the first 10 Fibonacci numbers")
for n <= 30 {
   println(i)
   let tmp = i
   i = j
   j = tmp + j
   n = n + 1 
}
```

Output:

```
Print the first 10 Fibonacci numbers
1
2
3  
5
8
13
21
34
55
89
```

---

### TODO

* [x] if condition statements
* [x] else statements
* [x] for loops
* [ ] Support break and continue keywords
* [x] bool type
* [x] int type
* [x] Arithmetic operators + - * / %
* [x] Comparison operators > >= < <= == !=
* [x] Logical operators && || !
* [x] Operator precedence
* [x] Operator precedence can be changed dynamically with parentheses
* [ ] Custom methods
* [ ] More code comments
* [ ] Write blog to document this project
* [ ] More comprehensive unit tests
* [ ] Object oriented features

---
Afterword: I finally implemented a program I've always wanted to write, thanks to the compiler principles course at Harbin Institute of Technology.
