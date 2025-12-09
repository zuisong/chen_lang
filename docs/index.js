import init, { run_wasm } from './pkg/chen_lang.js';

const examples = {
    hello: `# Basic: Hello World
print("Hello, Chen Lang!")

let a = 10
if a > 5 {
    println("a is greater than 5")
}
`,
    fib: `# Algorithm: Fibonacci
def fib(n) {
    if n <= 1 {
        return n
    }
    return fib(n-1) + fib(n-2)
}

println("Fibonacci of 10 is:")
println(fib(10))
`,
    objects: `# Pattern: Objects & Methods (Lua-style)
def Person(name) {
    let p = #{ name: name }
    
    def greet(self) {
        println("Hello, my name is " + self.name)
    }
    
    # Set prototype (methods)
    set_meta(p, #{ 
        __index: #{ greet: greet } 
    })
    return p
}

let chen = Person("Chen")
chen.greet()
`,
    inheritance: `# Pattern: Prototype Inheritance
# Base "Class"
def Animal(name) {
    let a = #{ name: name }
    def speak(self) {
        println(self.name + " makes a noise.")
    }
    set_meta(a, #{ __index: #{ speak: speak } })
    return a
}

# Derived "Class"
def Dog(name) {
    # 1. Create base instance
    let d = Animal(name)
    
    # 2. Define derived methods
    def bark(self) {
        println(self.name + " barks: Woof!")
    }
    
    # 3. Create methods table that inherits from base's methods
    # Get base prototype (metatable.__index)
    let base_proto = get_meta(d).__index
    
    # Create new prototype that inherits from base_proto
    let dog_proto = #{ bark: bark }
    set_meta(dog_proto, #{ __index: base_proto })
    
    # 4. Update instance's metatable to use new prototype
    set_meta(d, #{ __index: dog_proto })
    
    return d
}

let dog = Dog("Rex")
dog.speak() # Inherited from Animal
dog.bark()  # Defined in Dog
`,
    date: `# StdLib: Date & Time
let now = Date.new()
println("Current time (ISO): " + now.format("%Y-%m-%d %H:%M:%S"))

# JSON serialization of Date
println("As JSON: " + JSON.stringify(now))
`,
    json: `# StdLib: JSON Processing
let data = #{
    name: "Chen Lang",
    features: ["Simple", "Dynamic", "Rust-based"],
    version: 0.1
}

let jsonStr = JSON.stringify(data)
println("Serialized JSON:")
println(jsonStr)

let parsed = JSON.parse(jsonStr)
println("Parsed JSON Name: " + parsed.name)
`,
    arrays: `# StdLib: Arrays
# Arrays are dynamic list-like objects
let arr = [1, 2, 3]

arr.push(4)
println("Array length: " + arr.len())

let popped = arr.pop()
println("Popped value: " + popped)

# Arrays can store any type
arr.push("Mixed")
println(JSON.stringify(arr))
`
};

async function run() {
    await init();
    const runBtn = document.getElementById('run');
    const codeArea = document.getElementById('code');
    const outputArea = document.getElementById('output');
    const exampleSelect = document.getElementById('example-select');

    // Load initial example
    codeArea.value = examples.hello;
    exampleSelect.value = 'hello';

    runBtn.addEventListener('click', () => {
        const code = codeArea.value;
        try {
            const result = run_wasm(code);
            outputArea.textContent = result;
        } catch (e) {
            outputArea.textContent = `Error: ${e}`;
        }
    });

    exampleSelect.addEventListener('change', (e) => {
        const key = e.target.value;
        if (examples[key]) {
            codeArea.value = examples[key];
            // Clear output when changing example
            outputArea.textContent = '';
        }
    });
}

run();
