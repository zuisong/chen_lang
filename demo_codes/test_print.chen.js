let coroutine = Chen.coroutine
function task() { return 1 }
let co1 = coroutine.create(task)
console.log(co1)
