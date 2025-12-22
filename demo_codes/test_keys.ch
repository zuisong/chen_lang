import stdlib/io
let print = io.print
let println = io.println

let obj = #{ a: 1, b: 2 }
let keys = obj:keys()
println("Object keys:", keys)
println("Keys length:", keys:len())

let i = 0
for i < keys:len() {
    let key = keys[i]
    println("Key:", key, "Value:", obj[key])
    i = i + 1
}

let arr = [10, 20]
let arr_keys = arr:keys()
println("Array keys:", arr_keys)
