local name = "Module"
function greet(n)
    return "Hello, " .. n .. " from " .. name
end
return {
    name = name,
    greet = greet
}
