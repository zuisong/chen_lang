let io = import "stdlib/io"
let print = io.print
let println = io.println

let i=1
for i<=2 {
    let j = 1
    for j <= i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i=i+1
}
