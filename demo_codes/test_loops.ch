let io = import "stdlib/io"

# 1. Test condition-based loop (Go-style while)
let i = 0
for i < 5 {
    io.println("for i < 5: " + i)
    i = i + 1
}

# 2. Test infinite loop with break
let k = 0
for {
    if k >= 3 {
        break
    }
    io.println("infinite for k: " + k)
    k = k + 1
}

io.println("--- Array.iter() (Values only) ---")
let arr = [10, 20, 30]
for x in arr {
    io.println("arr x: " + x)
}

io.println("--- Array.entries() (Key-Value pairs) ---")
for entry in arr:entries() {
    io.println("index: " + entry.key + ", value: " + entry.value)
}

io.println("--- Object.iter() (Values only) ---")
let obj = ${ a: 1, b: 2 }
for v in obj {
    io.println("obj v: " + v)
}

io.println("--- Object.entries() (Key-Value pairs) ---")
for entry in obj:entries() {
    io.println("entry key: " + entry.key + ", value: " + entry.value)
}

io.println("--- Object.keys() ---")
let keys = obj:keys()
for k in keys {
    io.println("key: " + k)
}

io.println("--- String.iter() ---")
let s = "ABC"
for char in s {
    io.println("char: " + char)
}
