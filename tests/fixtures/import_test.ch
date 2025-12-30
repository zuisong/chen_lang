let io = import "stdlib/io"
let JSON = import "stdlib/json"

let data = #{ name: "Chen", version: 0.1 }
let json_str = JSON.stringify(data)
io.println("JSON: " + json_str)

let parsed = JSON.parse(json_str)
io.println("Name from JSON: " + parsed.name)
