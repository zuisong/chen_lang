local io = require("stdlib/io")
io.print("Please enter your name: ")
local name = io.readline()
io.println("Hello, " + name + "!")
