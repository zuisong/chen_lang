import init, { run_wasm } from './pkg/chen_lang.js';

const examples = {
    hello: `# Basic: Hello World
let io = import "stdlib/io"
let print = io.print
let println = io.println

print("Hello, Chen Lang!")

let a = 10
if a > 5 {
    println("a is greater than 5")
}
`,
    if_else_if: `# Feature: Else If Chain
let io = import "stdlib/io"
let println = io.println

let score = 85

if score >= 90 {
    println("Excellent! (A)")
} else if score >= 80 {
    println("Good! (B)")
} else if score >= 60 {
    println("Passed! (C)")
} else {
    println("Failed! (F)")
}

# It also works as an expression
let grade = if score >= 90 { "A" } else if score >= 60 { "P" } else { "F" }
println("Grade result: " + grade)
`,
    multiplication_table: `# Feature: for loop (9x9 Table)
let io = import "stdlib/io"
let print = io.print
let println = io.println

let i = 1
for i <= 9 {
    let j = 1
    for j <= i {
        print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    println("")
    i = i + 1
}
`,
    fib: `# Algorithm: Fibonacci
let io = import "stdlib/io"
let println = io.println

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
let io = import "stdlib/io"
let println = io.println

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
chen:greet()
`,
    inheritance: `# Pattern: Prototype Inheritance
let io = import "stdlib/io"
let println = io.println

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
dog:speak() # Inherited from Animal
dog:bark()  # Defined in Dog
`,
    point_objects: `# Pattern: Custom Objects (Point with methods and operators)
let io = import "stdlib/io"
let println = io.println


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
println(p1:to_string())
println(p2:to_string())

# Call a method to modify state
p1:move_by(5, -10)
println("p1 after move_by(5, -10):")
println(p1:to_string())

# Use overloaded operators
let p3_add = p1 + p2
println("p1 + p2 (overloaded +):")
println(p3_add:to_string())

let p4_sub = p1 - p2
println("p1 - p2 (overloaded -):")
println(p4_sub:to_string())

let p5_mul = new_Point(2,3) * new_Point(4,5)
println("p5_mul (overloaded *):")
println(p5_mul:to_string())
`,
    date: `# StdLib: Date & Time
let io = import "stdlib/io"
let Date = import "stdlib/date"
let JSON = import "stdlib/json"
let println = io.println

let now = Date:new()
println("Current time (ISO): " + now:format("%Y-%m-%d %H:%M:%S"))

# JSON serialization of Date
println("As JSON: " + JSON.stringify(now))
`,
    json: `# StdLib: JSON Processing
let io = import "stdlib/io"
let JSON = import "stdlib/json"
let println = io.println

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
let io = import "stdlib/io"
let JSON = import "stdlib/json"
let println = io.println

# Arrays are dynamic list-like objects
let arr = [1, 2, 3]

arr:push(4)
println("Array length: " + arr:len())

let popped = arr:pop()
println("Popped value: " + popped)

# Arrays can store any type
arr:push("Mixed")
println(JSON.stringify(arr))
`,
    closures: `# Feature: Closures
let io = import "stdlib/io"
let println = io.println

def make_counter(start) {
    let count = start
    
    # This inner function "captures" the 'count' variable
    # from the outer scope. It remembers it even after
    # make_counter has returned!
    def counter() {
        count = count + 1
        return count
    }
    
    return counter
}

let c1 = make_counter(0)
let c2 = make_counter(10)

println("Counter 1: " + c1()) # 1
println("Counter 1: " + c1()) # 2
println("Counter 1: " + c1()) # 3

println("Counter 2: " + c2()) # 11
println("Counter 2: " + c2()) # 12

# c1 is unaffected by c2
println("Counter 1 again: " + c1()) # 4
`,
    async_task: `# Feature: Async/Await (Coroutines)
let io = import "stdlib/io"
let println = io.println

# Coroutines using 'coroutine.create' and 'coroutine.yield'.
# This mimics Lua's asymmetric coroutines.
def generator(limit) {
    println("Generator starting...")
    let i = 0
    for i < limit {
        println("Generator yielding " + i)
        coroutine.yield(i)
        i = i + 1
    }
    return "Done"
}

# Create a coroutine from the function
let gen = coroutine.create(generator)
println("Generator status: " + coroutine.status(gen))

# Resume the coroutine to start it, passing 'limit' argument (3)
let val1 = coroutine.resume(gen, 3)
println("Main got: " + val1)

# Resume again
let val2 = coroutine.resume(gen)
println("Main got: " + val2)

# Resume again
let val3 = coroutine.resume(gen)
println("Main got: " + val3)

# Final resume gets the return value
let result = coroutine.resume(gen)
println("Main result: " + result)
println("Generator status: " + coroutine.status(gen))
`,
    async_http: `# Feature: Async HTTP Request
let http = import "stdlib/http"
let json = import "stdlib/json"
let println = import "stdlib/io".println

println("Sending request to httpbin.org...")
let url = "https://httpbin.org/anything"
let resp = http.request("GET", url)

println("Status: " + resp.status)
let data = json.parse(resp.body)
println("Response JSON origin: " + data.origin)
`,
    concurrent_http: `# Feature: Concurrent HTTP Requests
let http = import "stdlib/http"
let json = import "stdlib/json"
let println = import "stdlib/io".println

println("Starting concurrent HTTP requests...")

# Helper function to fetch URL and return status
def fetch_status(url) {
    let resp = http.request("GET", url)
    return resp.status
}

# Create coroutines for parallel requests
let co1 = coroutine.create(def() { fetch_status("https://httpbin.org/delay/1") })
let co2 = coroutine.create(def() { fetch_status("https://httpbin.org/delay/1") })
let co3 = coroutine.create(def() {
    let resp = http.request("GET", "https://httpbin.org/uuid")
    let data = json.parse(resp.body)
    return data.uuid
})

# Spawn all coroutines (non-blocking)
coroutine.spawn(co1)
coroutine.spawn(co2)
coroutine.spawn(co3)

println("All requests started, waiting for completion...")

# Wait for all to complete
let results = coroutine.await_all([co1, co2, co3])

println("All requests completed!")
println("Request 1 status: " + results[0])
println("Request 2 status: " + results[1])
println("Request 3 UUID: " + results[2])
`,
    christmas_tree: `# Merry Christmas!
let println = import "stdlib/io".println

# Simple string repeat function
def repeat(str, count) {
    let res = ""
    let i = 0
    for i < count {
        res = res + str
        i = i + 1
    }
    return res
}

def print_tree(height) {
    println("🎄 Merry Christmas! 🎄")
    println("")

    # Print leaves
    let i = 1
    for i <= height {
        let spaces = repeat(" ", height - i)
        let stars = repeat("*", 2 * i - 1)
        println(spaces + stars)
        i = i + 1
    }

    # Print trunk
    let trunk_padding = repeat(" ", height - 2)
    
    let j = 0
    for j < 2 {
        println(trunk_padding + "###")
        j = j + 1
    }
    
    println("")
    println(repeat(" ", height - 4) + "Happy New Year!")
}

print_tree(10)
`
};

async function run() {
    await init();
    const runBtn = document.getElementById('run');
    const codeArea = document.getElementById('code');
    const outputArea = document.getElementById('output');
    const exampleSelect = document.getElementById('example-select');
    const clearBtn = document.getElementById('clear-output');
    const lineNumbers = document.getElementById('line-numbers');
    const highlighting = document.getElementById('highlighting');

    const updateLineNumbers = () => {
        const lines = codeArea.value.split('\n').length;
        lineNumbers.innerHTML = Array(lines).fill(0).map((_, i) => `<div>${i + 1}</div>`).join('');

        // Also run highlighting
        highlight();
    };

    const highlight = () => {
        let code = codeArea.value;

        // Escape HTML
        code = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

        // Syntax Rules
        const rules = [
            { rex: /(?<=^|\s|;)(#[^\{].*|#$)/g, cls: 'comment' },
            { rex: /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/g, cls: 'string' },
            { rex: /\b(let|def|if|else|return|for|break|continue|async|await|try|catch|finally|throw)\b/g, cls: 'keyword' },
            { rex: /\b(true|false)\b/g, cls: 'boolean' },
            { rex: /\b(null)\b/g, cls: 'null' },
            { rex: /\b(\d+(?:\.\d*)?)\b/g, cls: 'number' },
            { rex: /(\b\w+)(?=\s*\()/g, cls: 'function' },
            { rex: /([\+\-\*\/%=\!<>]=?|&&|\|\|)/g, cls: 'operator' }
        ];

        // Apply rules
        // We use a temporary map to avoid double-highlighting
        let tokens = [];
        let output = code;

        // Simplified approach: sort-of-lexer
        // For simple playground, regex replacement in order with special placeholders or spans is okay
        // if we are careful about not matching inside spans.
        // A better way is matching all then sorting by index.

        const allMatches = [];
        rules.forEach(rule => {
            let match;
            while ((match = rule.rex.exec(code)) !== null) {
                allMatches.push({ index: match.index, length: match[0].length, cls: rule.cls, text: match[0] });
            }
        });

        // Sort by index
        allMatches.sort((a, b) => a.index - b.index);

        // Filter overlaps
        let lastEnd = 0;
        let finalHtml = "";
        allMatches.forEach(m => {
            if (m.index >= lastEnd) {
                finalHtml += code.substring(lastEnd, m.index);
                finalHtml += `<span class="token-${m.cls}">${m.text}</span>`;
                lastEnd = m.index + m.length;
            }
        });
        finalHtml += code.substring(lastEnd);

        highlighting.innerHTML = finalHtml + "\n"; // Extra newline to match textarea behavior
    };

    const syncScroll = () => {
        lineNumbers.scrollTop = codeArea.scrollTop;
        highlighting.scrollTop = codeArea.scrollTop;
        highlighting.scrollLeft = codeArea.scrollLeft;
    };

    // Load initial example
    codeArea.value = examples.hello;
    exampleSelect.value = 'hello';
    updateLineNumbers();

    codeArea.addEventListener('input', updateLineNumbers);
    codeArea.addEventListener('scroll', syncScroll);

    runBtn.addEventListener('click', async () => {
        const code = codeArea.value;
        outputArea.textContent = '';

        window.print_output = (text) => {
            outputArea.textContent += text;
            outputArea.scrollTop = outputArea.scrollHeight;
        };

        try {
            const result = await run_wasm(code);
            if (result) outputArea.textContent += result;
        } catch (e) {
            outputArea.textContent += `Error: ${e}`;
        } finally {
            window.print_output = null;
        }
    });

    exampleSelect.addEventListener('change', (e) => {
        const key = e.target.value;
        if (examples[key]) {
            codeArea.value = examples[key];
            updateLineNumbers();
            // Clear output when changing example
            outputArea.textContent = '';
        }
    });

    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            outputArea.textContent = '';
        });
    }
}

run();
