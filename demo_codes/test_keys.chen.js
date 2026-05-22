let obj = { a: 1, b: 2 }
let keys = Object.keys(obj)
console.log("Object keys:", keys)
console.log("Keys length:", keys.length)

let i = 0
while (i < keys.length) {
    let key = keys[i]
    console.log("Key:", key, "Value:", obj[key])
    i = i + 1
}

let arr = [10, 20]
let arr_keys = Object.keys(arr)
console.log("Array keys:", arr_keys)
