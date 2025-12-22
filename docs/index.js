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
