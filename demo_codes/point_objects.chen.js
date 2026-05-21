# Pattern: Custom Objects (Point with methods and operators)

# Define Point prototype (shared methods and metamethods)
let Point = {
    __index: {
        # Method: Return string representation
        to_string: function() {
            return "Point(" + this.x + ", " + this.y + ")"
        },
        # Method: Move point by dx, dy
        move_by: function(dx, dy) {
            this.x = this.x + dx
            this.y = this.y + dy
        }
    },
    # Metamethod: Operator Overloading for addition (+)
    __add: function(a, b) {
        # Returns a new Point instance
        return new_Point(a.x + b.x, a.y + b.y)
    },
    # Metamethod: Operator Overloading for subtraction (-)
    __sub: function(a, b) {
        return new_Point(a.x - b.x, a.y - b.y)
    },
    # Metamethod: Operator Overloading for multiplication (*)
    __mul: function(a, b) {
        return new_Point(a.x * b.x, a.y * b.y)
    }
}

# Constructor function for Point objects
function new_Point(x_coord, y_coord) {
    let instance = {
        x: x_coord,
        y: y_coord
    }
    # Set the instance's metatable to the Point prototype
    Chen.setMeta(instance, Point)
    return instance
}

# --- Usage Examples ---

# Create Point instances
let p1 = new_Point(10, 20)
let p2 = new_Point(3, 5)

console.log("Original Points:")
console.log(p1.to_string())
console.log(p2.to_string())

# Call a method to modify state
p1.move_by(5, -10)
console.log("p1 after move_by(5, -10):")
console.log(p1.to_string())

# Use overloaded operators
let p3_add = p1 + p2
console.log("p1 + p2 (overloaded +):")
console.log(p3_add.to_string())

# Use overloaded operators
let p4_sub = p1 - p2
console.log("p1 - p2 (overloaded -):")
console.log(p4_sub.to_string())

let p5_mul = new_Point(2, 3) * new_Point(4, 5)
console.log("p5_mul (overloaded *):")
console.log(p5_mul.to_string())
