-- Basic try-catch test
try
    error("Something went wrong!")
catch error
    print("Caught error: " + error)
end

print("Program continues...")
