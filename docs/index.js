import init, { run_wasm } from './pkg/chen_lang.js';

const examples = {
    hello: `// Basic: Hello World
console.log("Hello, Chen Lang!")

let a = 10
if (a > 5) {
    console.log("a is greater than 5")
}
`,
    if_else_if: `// Feature: Else If Chain
let score = 85

if (score >= 90) {
    console.log("Excellent! (A)")
} else if (score >= 80) {
    console.log("Good! (B)")
} else if (score >= 60) {
    console.log("Passed! (C)")
} else {
    console.log("Failed! (F)")
}

// It also works as an expression
let grade = if (score >= 90) { "A" } else if (score >= 60) { "P" } else { "F" }
console.log("Grade result: " + grade)
`,
    multiplication_table: `// Feature: for loop (9x9 Table)
let i = 1
while (i <= 9) {
    let j = 1
    while (j <= i) {
        console.print(j + "x" + i + "=" + i*j + " ")
        j = j + 1
    }
    console.log("")
    i = i + 1
}
`,
    fib: `// Algorithm: Fibonacci
function fib(n) {
    if (n <= 1) {
        return n
    }
    return fib(n-1) + fib(n-2)
}

console.log("Fibonacci of 10 is:")
console.log(fib(10))
`,
    objects: `// Pattern: Objects & Methods
function Person(name) {
    let p = { name: name }

    p.greet = function() {
        console.log("Hello, my name is " + this.name)
    }

    return p
}

let chen = Person("Chen")
chen.greet()
`,
    metamethod_funcs: `// Advanced: Metamethod Functions
// Implement a "Strict Object" that throws on unknown access
function create_strict_model(data) {
    let meta = {
        // Intercept missing property lookup
        __index: function(obj, key) {
            console.log("Warning: Accessing undefined property '" + key + "'")
            return null
        },
        // Intercept new property assignment
        __newindex: function(obj, key, value) {
            console.log("Blocked: Setting new property '" + key + "' to " + value)
        }
    }
    Chen.setMeta(data, meta)
    return data
}

let user = create_strict_model({ name: "Chen" })

console.log("Name: " + user.name)
console.log("Age: " + user.age)  // Triggers __index

user.score = 100            // Triggers __newindex
`,
    inheritance: `// Pattern: Prototype Inheritance
// Base "Class"
function Animal(name) {
    let a = { name: name }
    a.speak = function() {
        console.log(this.name + " makes a noise.")
    }
    return a
}

// Derived "Class"
function Dog(name) {
    // Create new prototype that inherits from Animal's prototype
    let dog = Object.create(Animal(name))

    // Define derived methods
    dog.bark = function() {
        console.log(this.name + " barks: Woof!")
    }

    return dog
}

let dog = Dog("Rex")
dog.speak() // Inherited from Animal
dog.bark()  // Defined in Dog
`,
    point_objects: `// Pattern: Custom Objects (Point with methods and operators)
// Define Point prototype (shared methods and metamethods)
let PointProto = { 
    // Method: Return string representation
    to_string: function() {
        return "Point(" + this.x + ", " + this.y + ")"
    },
    // Method: Move point by dx, dy
    move_by: function(dx, dy) {
        this.x = this.x + dx
        this.y = this.y + dy
    },
    // Metamethod: Operator Overloading for addition (+)
    __add: function(a, b) {
        return new_Point(a.x + b.x, a.y + b.y)
    },
    // Metamethod: Operator Overloading for subtraction (-)
    __sub: function(a, b) {
        return new_Point(a.x - b.x, a.y - b.y)
    },
    // Metamethod: Operator Overloading for multiplication (*)
    __mul: function(a, b) {
        return new_Point(a.x * b.x, a.y * b.y)
    }
}

// Constructor function for Point objects
function new_Point(x_coord, y_coord) {
    let instance = { 
        x: x_coord,
        y: y_coord
    }
    // Set the instance's metatable to the Point prototype
    Chen.setMeta(instance, PointProto)
    return instance
}

// --- Usage Examples ---

let p1 = new_Point(10, 20)
let p2 = new_Point(3, 5)

console.log("Original Points:")
console.log(p1.to_string())
console.log(p2.to_string())

p1.move_by(5, -10)
console.log("p1 after move_by(5, -10):")
console.log(p1.to_string())

let p3_add = p1 + p2
console.log("p1 + p2 (overloaded +):")
console.log(p3_add.to_string())
`,
    date: `// StdLib: Date & Time
let now = Chen.date.new()
console.log("Current time (ISO): " + now.format("%Y-%m-%d %H:%M:%S"))

// JSON serialization of Date
console.log("As JSON: " + JSON.stringify(now))
`,
    json: `// StdLib: JSON Processing
let data = { 
    name: "Chen Lang",
    features: ["Simple", "Dynamic", "Rust-based"],
    version: 0.1
}

let jsonStr = JSON.stringify(data)
console.log("Serialized JSON:")
console.log(jsonStr)

let parsed = JSON.parse(jsonStr)
console.log("Parsed JSON Name: " + parsed.name)
`,
    arrays: `// StdLib: Arrays
// Arrays are dynamic list-like objects
let arr = [1, 2, 3]

arr.push(4)
console.log("Array length: " + arr.length)

let popped = arr.pop()
console.log("Popped value: " + popped)

// Arrays can store any type
arr.push("Mixed")
console.log(JSON.stringify(arr))
`,
    closures: `// Feature: Closures
function make_counter(start) {
    let count = start
    
    // This inner function "captures" the 'count' variable
    return function() {
        count = count + 1
        return count
    }
}

let c1 = make_counter(0)
let c2 = make_counter(10)

console.log("Counter 1: " + c1()) // 1
console.log("Counter 1: " + c1()) // 2
console.log("Counter 2: " + c2()) // 11
`,
    async_task: `// Feature: Async/Await
async function fetch_data(id) {
    console.log("Fetching data for ID: " + id + "...")
    await Chen.timer.sleep(1000) // Simulate network delay
    return { id: id, data: "Data for " + id }
}

async function main() {
    console.log("Starting async tasks...")
    
    // Run tasks sequentially
    let r1 = await fetch_data(1)
    console.log("Got: " + r1.data)
    
    let r2 = await fetch_data(2)
    console.log("Got: " + r2.data)
    
    console.log("All tasks completed!")
}

await main()
`,
    async_http: `// Feature: Async HTTP Request
console.log("Sending request to httpbin.org...")
let url = "https://httpbin.org/anything"
let resp = Chen.http.request("GET", url)

console.log("Status: " + resp.status)
let data = JSON.parse(resp.body)
console.log("Response JSON origin: " + data.origin)
`,
    concurrent_http: `// Feature: Concurrent HTTP Requests
console.log("Starting concurrent HTTP requests...")

// Helper function to fetch URL and return status
async function fetch_status(url) {
    let resp = Chen.http.request("GET", url)
    return resp.status
}

async function fetch_uuid() {
    let resp = Chen.http.request("GET", "https://httpbin.org/uuid")
    let data = JSON.parse(resp.body)
    return data.uuid
}

async function main() {
    // Start promises concurrently
    let p1 = fetch_status("https://httpbin.org/delay/1")
    let p2 = fetch_status("https://httpbin.org/delay/1")
    let p3 = fetch_uuid()

    console.log("All requests started, waiting for completion...")

    // Wait for all to complete
    let results = await Promise.all([p1, p2, p3])

    console.log("All requests completed!")
    console.log("Request 1 status: " + results[0])
    console.log("Request 2 status: " + results[1])
    console.log("Request 3 UUID: " + results[2])
}

await main()
`,
    christmas_tree: `// Merry Christmas!
// Simple string repeat function
function repeat(str, count) {
    let res = ""
    let i = 0
    while (i < count) {
        res = res + str
        i = i + 1
    }
    return res
}

function print_tree(height) {
    console.log("🎄 Merry Christmas! 🎄")
    console.log("")

    // Print leaves
    let i = 1
    while (i <= height) {
        let spaces = repeat(" ", height - i)
        let stars = repeat("*", 2 * i - 1)
        console.log(spaces + stars)
        i = i + 1
    }

    // Print trunk
    let trunk_padding = repeat(" ", height - 2)
    
    let j = 0
    while (j < 2) {
        console.log(trunk_padding + "###")
        j = j + 1
    }
    
    console.log("")
    console.log(repeat(" ", height - 4) + "Happy New Year!")
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
            { rex: /(?<=^|\\s|;)(\/\/.*|\/\/ $)/g, cls: 'comment' },
            { rex: /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/g, cls: 'string' },
            { rex: /\b(let|function|if|else|return|while|for|of|break|continue|async|await|try|catch|finally|throw)\b/g, cls: 'keyword' },
            { rex: /\b(true|false)\b/g, cls: 'boolean' },
            { rex: /\b(null)\b/g, cls: 'null' },
            { rex: /\b(\d+(?:\.\d*)?)\b/g, cls: 'number' },
            { rex: /(\b\w+)(?=\s*\()/g, cls: 'function' },
            { rex: /([\+\-\*\/%=\!<>]=?|&&|\|\|)/g, cls: 'operator' }
        ];

        // Apply rules
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
