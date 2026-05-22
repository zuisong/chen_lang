// 1. Test condition-based loop (Go-style while)
let i = 0
while (i < 5) {
    console.log("for i < 5: " + i)
    i = i + 1
}

// 2. Test infinite loop with break
let k = 0
while (true) {
    if (k >= 3) {
        break
    }
    console.log("infinite for k: " + k)
    k = k + 1
}

console.log("--- Array.iter() (Values only) ---")
let arr = [10, 20, 30]
for (let x of arr) {
    console.log("arr x: " + x)
}

console.log("--- Array.entries() (Key-Value pairs) ---")
for (let entry of arr.entries()) {
    console.log("index: " + entry.key + ", value: " + entry.value)
}

console.log("--- Object.iter() (Values only) ---")
let obj = { a: 1, b: 2 }
for (let v of obj) {
    console.log("obj v: " + v)
}

console.log("--- Object.entries() (Key-Value pairs) ---")
for (let entry of obj.entries()) {
    console.log("entry key: " + entry["0"] + ", value: " + entry["1"])
}

console.log("--- Object.keys() ---")
for (let k of Object.keys(obj)) {
    console.log("key: " + k)
}

console.log("--- String.iter() ---")
let s = "ABC"
for (let char of s) {
    console.log("char: " + char)
}
