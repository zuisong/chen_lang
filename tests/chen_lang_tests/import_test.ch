import stdlib/io
import stdlib/json

let data = #{ name: "Chen", version: 0.1 }
let json_str = JSON.stringify(data)
println("JSON: " + json_str)

let parsed = JSON.parse(json_str)
println("Name from JSON: " + parsed.name)
