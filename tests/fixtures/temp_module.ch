let name = "Module"
def greet(n) {
    return "Hello, " + n + " from " + name
}
return ${
    name: name,
    greet: greet
}
