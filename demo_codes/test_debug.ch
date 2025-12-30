let io = import "stdlib/io"
let print = io.print
let println = io.println

def func(){
    return 123
}
let x = 1
x = func()
println(x)
