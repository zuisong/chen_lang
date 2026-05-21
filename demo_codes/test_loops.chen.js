let coroutine = Chen.coroutine

# 1. Test condition-based loop (Go-style while)
let i = 0
while (i < 5) {
    console.log("for i < 5: " + i)
    i = i + 1
}

# 2. Test infinite loop with break
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
let arr_co = arr.iter()
while (coroutine.status(arr_co) != "dead") {
    let x = coroutine.resume(arr_co)
    if (x != null) {
        console.log("arr x: " + x)
    }
}

console.log("--- Array.entries() (Key-Value pairs) ---")
let arr_ent_co = arr.entries()
while (coroutine.status(arr_ent_co) != "dead") {
    let entry = coroutine.resume(arr_ent_co)
    if (entry != null) {
        console.log("index: " + entry.key + ", value: " + entry.value)
    }
}

console.log("--- Object.iter() (Values only) ---")
let obj = { a: 1, b: 2 }
let obj_co = obj.iter()
while (coroutine.status(obj_co) != "dead") {
    let v = coroutine.resume(obj_co)
    if (v != null) {
        console.log("obj v: " + v)
    }
}

console.log("--- Object.entries() (Key-Value pairs) ---")
let obj_ent_arr = obj.entries()
let obj_ent_co = obj_ent_arr.iter()
while (coroutine.status(obj_ent_co) != "dead") {
    let entry = coroutine.resume(obj_ent_co)
    if (entry != null) {
        console.log("entry key: " + entry["0"] + ", value: " + entry["1"])
    }
}

console.log("--- Object.keys() ---")
let keys = Object.keys(obj)
let keys_co = keys.iter()
while (coroutine.status(keys_co) != "dead") {
    let k = coroutine.resume(keys_co)
    if (k != null) {
        console.log("key: " + k)
    }
}

console.log("--- String.iter() ---")
let s = "ABC"
let s_co = s.iter()
while (coroutine.status(s_co) != "dead") {
    let char = coroutine.resume(s_co)
    if (char != null) {
        console.log("char: " + char)
    }
}
