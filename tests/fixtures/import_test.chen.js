let data = { name: "Chen", version: 0.1 }
let json_str = JSON.stringify(data)
console.log("JSON: " + json_str)

let parsed = JSON.parse(json_str)
console.log("Name from JSON: " + parsed.name)
