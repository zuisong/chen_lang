-- Try-catch with finally
local cleanup_called = false

try
    print("In try block")
    error("Error occurred")
catch error
    print("In catch block: " + error)
finally
    print("In finally block")
    cleanup_called = true
end

print("Cleanup called: " + cleanup_called)
