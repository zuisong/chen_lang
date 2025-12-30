let io = import "stdlib/io"
let print = io.print
let println = io.println

# Pattern: Custom Objects (Point with methods and operators)

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
