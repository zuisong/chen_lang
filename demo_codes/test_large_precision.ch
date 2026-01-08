let io = import "stdlib/io"
let print = io.print
let println = io.println
let JSON = import "stdlib/json"

let data = ${
    pi: 3.141592653589793,
    e: 2.718281828459045,
    small: 0.000000001
}
let json_str = JSON.stringify(data)
println(json_str)
