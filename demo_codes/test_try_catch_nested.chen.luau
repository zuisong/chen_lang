-- Nested try-catch
try
    print("Outer try")

    try
        print("Inner try")
        error("Inner error")
    catch inner_error
        print("Inner catch: " + inner_error)
        error("Outer error")
    end

    print("This should not print")
catch outer_error
    print("Outer catch: " + outer_error)
end

print("Done")
