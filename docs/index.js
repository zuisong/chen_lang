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
    point_objects: `# Pattern: Custom Objects (Point with methods and operators)

# Define Point prototype (shared methods and metamethods)
let Point = #{
    __index: #{
        # Method: Return string representation
        to_string: def(self) {
            return "Point(" + self.x + ", " + self.y + ")"
        },
        # Method: Move point by dx, dy
        move_by: def(self, dx, dy) {
            self.x = self.x + dx
            self.y = self.y + dy
        }
    },
    # Metamethod: Operator Overloading for addition (+)
    __add: def(a, b) {
        # Returns a new Point instance
        return new_Point(a.x + b.x, a.y + b.y)
    },
    # Metamethod: Operator Overloading for subtraction (-)
    __sub: def(a, b) {
        return new_Point(a.x - b.x, a.y - b.y)
    },
    # Metamethod: Operator Overloading for multiplication (*)
    __mul: def(a, b) {
        return new_Point(a.x * b.x, a.y * b.y)
    }
}

# Constructor function for Point objects
def new_Point(x_coord, y_coord) {
    let instance = #{
        x: x_coord,
        y: y_coord
    }
    # Set the instance's metatable to the Point prototype
    set_meta(instance, Point)
    return instance
}

# --- Usage Examples ---

# Create Point instances
let p1 = new_Point(10, 20)
let p2 = new_Point(3, 5)

println("Original Points:")
println(p1.to_string())
println(p2.to_string())

# Call a method to modify state
p1.move_by(5, -10)
println("p1 after move_by(5, -10):")
println(p1.to_string())

# Use overloaded operators
let p3_add = p1 + p2
println("p1 + p2 (overloaded +):")
println(p3_add.to_string())

let p4_sub = p1 - p2
println("p1 - p2 (overloaded -):")
println(p4_sub.to_string())

let p5_mul = new_Point(2,3) * new_Point(4,5)
println("p5_mul (overloaded *):")
println(p5_mul.to_string())
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
