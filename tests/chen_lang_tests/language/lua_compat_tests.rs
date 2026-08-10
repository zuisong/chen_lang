use crate::common::run_chen_lang_code;

#[test]
fn test_multi_return() {
    let code = r#"
    local function f()
        return 1, 2, 3
    end
    local a, b, c = f()
    print(a, b, c)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "123");
}

#[test]
fn test_multi_return_truncation() {
    let code = r#"
    local function f()
        return 1, 2, 3
    end
    local x, y = f()
    print(x, y)
    print(f())
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "12");
    assert_eq!(lines[1], "1");
}

#[test]
fn test_return_passthrough() {
    let code = r#"
    local function inner()
        return "a", "b"
    end
    local function outer()
        return inner()
    end
    local x, y = outer()
    print(x, y)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "ab");
}

#[test]
fn test_multi_return_padding() {
    let code = r#"
    local function f()
        return 42
    end
    local a, b, c = f()
    print(a, b, c)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "42nilnil");
}

#[test]
fn test_multi_assign_swap() {
    let code = r#"
    local a = 1
    local b = 2
    a, b = b, a
    print(a, b)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "21");
}

#[test]
fn test_multi_local_list() {
    let code = r#"
    local a, b, c = 1, 2
    print(a, b, c)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "12nil");
}

#[test]
fn test_pairs_iteration() {
    let code = r#"
    local t = { name = "Alice", age = 30 }
    local keys = {}
    for k, v in pairs(t) do
        print(k, v)
    end
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "nameAlice");
    assert_eq!(lines[1], "age30");
}

#[test]
fn test_ipairs_iteration() {
    let code = r#"
    local a = { "x", "y", "z" }
    for i, v in ipairs(a) do
        print(i, v)
    end
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "0x");
    assert_eq!(lines[1], "1y");
    assert_eq!(lines[2], "2z");
}

#[test]
fn test_for_in_entries_multivars() {
    let code = r#"
    local t = { a = 1, b = 2 }
    for k, v in t:entries() do
        print(k, v)
    end
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "a1");
    assert_eq!(lines[1], "b2");
}

#[test]
fn test_next_function() {
    let code = r#"
    local t = { name = "Alice", age = 30 }
    local k, v = next(t)
    print(k, v)
    local k2 = next(t, k)
    print(k2 ~= nil)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "nameAlice");
    assert_eq!(lines[1], "true");
}

#[test]
fn test_type_function() {
    let code = r#"
    print(type(1), type(1.5), type("s"), type(true), type(nil))
    print(type({}), type(function() end))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "intfloatstringbooleannil");
    assert_eq!(lines[1], "tablefunction");
}

#[test]
fn test_tonumber_tostring() {
    let code = r#"
    print(tonumber("42"), tonumber("3.14"))
    print(tonumber("abc"))
    print(tostring(123))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "423.14");
    assert_eq!(lines[1], "nil");
    assert_eq!(lines[2], "123");
}

#[test]
fn test_pcall() {
    let code = r#"
    local function ok_fn(x)
        return x * 2
    end
    local function fail_fn()
        error("boom")
    end
    local ok, res = pcall(ok_fn, 21)
    print(ok, res)
    local ok2, err = pcall(fail_fn)
    print(ok2, err)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "true42");
    assert_eq!(lines[1], "falseboom");
}

#[test]
fn test_xpcall() {
    let code = r#"
    local function fail_fn()
        error("oops")
    end
    local function handler(e)
        return "handled:" + e
    end
    local ok, res = xpcall(fail_fn, handler)
    print(ok, res)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "falsehandled:oops");
}

#[test]
fn test_assert_function() {
    let code = r#"
    print(assert(42))
    local ok, err = pcall(assert, false, "custom msg")
    print(ok, err)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "42");
    assert_eq!(lines[1], "falsecustom msg");
}

#[test]
fn test_select_function() {
    let code = r#"
    local x, y = select(2, "a", "b", "c")
    print(x, y)
    local z = select(-1, "a", "b", "c")
    print(z)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "bc");
    assert_eq!(lines[1], "c");
}

#[test]
fn test_select_count() {
    let code = r##"
    print(select("#", "a", "b", "c"))
    "##;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "3");
}

#[test]
fn test_unpack_function() {
    let code = r#"
    local a, b, c = unpack({10, 20, 30})
    print(a, b, c)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "102030");
}

#[test]
fn test_table_library() {
    let code = r#"
    local t = { 3, 1, 2 }
    table.insert(t, 4)
    print(table.concat(t, ","))
    print(table.remove(t))
    table.sort(t)
    print(table.concat(t, ","))
    print(table.getn(t))
    local p = table.pack("a", "b")
    print(p.n)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "3,1,2,4");
    assert_eq!(lines[1], "4");
    assert_eq!(lines[2], "1,2,3");
    assert_eq!(lines[3], "3");
    assert_eq!(lines[4], "2");
}

#[test]
fn test_string_library() {
    let code = r#"
    print(string.sub("hello world", 1, 5))
    print(string.sub("hello", -2))
    print(string.rep("ab", 3))
    print(string.upper("abc"))
    print(string.reverse("abc"))
    print(string.char(65), string.byte("A"))
    print("hello":upper())
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "hello");
    assert_eq!(lines[1], "lo");
    assert_eq!(lines[2], "ababab");
    assert_eq!(lines[3], "ABC");
    assert_eq!(lines[4], "cba");
    assert_eq!(lines[5], "A65");
    assert_eq!(lines[6], "HELLO");
}

#[test]
fn test_string_format() {
    let code = r#"
    print(string.format("%s=%d %.2f", "x", 42, 3.14159))
    print(string.format("%x", 255))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "x=42 3.14");
    assert_eq!(lines[1], "ff");
}

#[test]
fn test_string_patterns() {
    let code = r#"
    print(string.find("hello world", "world"))
    print(string.match("key=value", "(%w+)=(%w+)"))
    local s, n = string.gsub("aaa bbb aaa", "aaa", "X")
    print(s, n)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "7");
    assert_eq!(lines[1], "key");
    assert_eq!(lines[2], "X bbb X2");
}

#[test]
fn test_string_gmatch() {
    let code = r#"
    for m in string.gmatch("a1 b2 c3", "%a%d") do
        print(m)
    end
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["a1", "b2", "c3"]);
}

#[test]
fn test_math_library() {
    let code = r#"
    print(math.floor(3.7), math.ceil(3.2), math.abs(-5))
    print(math.max(1, 5, 3), math.min(1, 5, 3))
    print(math.sqrt(16), math.pow(2, 10))
    print(math.pi > 3.14)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "345");
    assert_eq!(lines[1], "51");
    assert_eq!(lines[2], "41024");
    assert_eq!(lines[3], "true");
}

#[test]
fn test_os_library() {
    let code = r#"
    print(os.getenv("HOME") ~= nil)
    print(type(os.time()) == "int" or type(os.time()) == "float")
    print(#os.date() > 0)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "true");
    assert_eq!(lines[1], "true");
    assert_eq!(lines[2], "true");
}

#[test]
fn test_call_metamethod() {
    let code = r#"
    local obj = { __call = function(self, x) return x * 2 end }
    set_meta(obj, { __call = obj.__call })
    print(obj(21))
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "42");
}

#[test]
fn test_tostring_metamethod() {
    let code = r#"
    local Point = { __tostring = function(self) return "(" + self.x + "," + self.y + ")" end }
    local p = { x = 1, y = 2 }
    set_meta(p, Point)
    print(p)
    print(tostring(p))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "(1,2)");
    assert_eq!(lines[1], "(1,2)");
}

#[test]
fn test_len_eq_lt_metamethods() {
    let code = r#"
    local M = {
        __len = function(self) return self.n end,
        __eq = function(a, b) return a.v == b.v end,
        __lt = function(a, b) return a.v < b.v end,
    }
    local a = { v = 5, n = 42 }
    local b = { v = 5, n = 42 }
    local c = { v = 9, n = 42 }
    set_meta(a, M)
    set_meta(b, M)
    set_meta(c, M)
    print(#a, a == b, a < c, c > a)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "42truetruetrue");
}

#[test]
fn test_concat_div_mod_metamethods() {
    let code = r#"
    local Box = {
        __concat = function(a, b) return "[" + a.v + "]" end,
        __div = function(a, b) return a.v / b.v end,
        __mod = function(a, b) return a.v % b.v end,
    }
    local x = { v = 10 }
    local y = { v = 3 }
    set_meta(x, Box)
    set_meta(y, Box)
    print(x .. y)
    print(x / y)
    print(x % y)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "[10]");
    assert_eq!(lines[1], "3");
    assert_eq!(lines[2], "1");
}

#[test]
fn test_raw_functions() {
    let code = r#"
    local t = { a = 1 }
    print(rawget(t, "a"))
    rawset(t, "b", 2)
    print(rawget(t, "b"))
    print(rawlen(t))
    print(rawequal(t, t))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "1");
    assert_eq!(lines[1], "2");
    assert_eq!(lines[2], "2");
    assert_eq!(lines[3], "true");
}

#[test]
fn test_global_env() {
    let code = r#"
    print(type(_G))
    print(_G ~= nil)
    print(type(_VERSION))
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "table");
    assert_eq!(lines[1], "true");
    assert_eq!(lines[2], "string");
}

#[test]
fn test_vararg_collect() {
    let code = r#"
    local function sum(...)
        local total = 0
        local args = { ... }
        for i, v in ipairs(args) do
            total = total + v
        end
        return total
    end
    print(sum(1, 2, 3, 4))
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "10");
}

#[test]
fn test_vararg_mixed_params() {
    let code = r#"
    local function first(a, b, ...)
        return a, b, ...
    end
    local x, y, z, w = first(1, 2, 3, 4, 5)
    print(x, y, z, w)
    "#;
    assert_eq!(run_chen_lang_code(code).unwrap().trim(), "1234");
}

#[test]
fn test_vararg_passthrough() {
    let code = r##"
    local function inner(...)
        return select("#", ...)
    end
    print(inner("a", "b", "c"))
    local function wrap(...)
        return inner(...)
    end
    print(wrap(1, 2))
    "##;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "3");
    assert_eq!(lines[1], "2");
}

#[test]
fn test_power_operator() {
    let code = r#"
    print(2 ^ 10)
    print(2 ^ 3 ^ 2)
    print(-2 ^ 2)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "1024");
    assert_eq!(lines[1], "512");
    assert_eq!(lines[2], "-4");
}

#[test]
fn test_numeric_for_negative_step() {
    let code = r#"
    local sum = 0
    for i = 10, 1, -1 do
        sum = sum + i
    end
    print(sum)
    local count = 0
    for i = 1, 10, 2 do
        count = count + 1
    end
    print(count)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "55");
    assert_eq!(lines[1], "5");
}

#[test]
fn test_inheritance_via_index() {
    let code = r#"
    local Base = {
        __index = {
            greet = function(self) return "hi " + self.name end,
            __tostring = function(self) return "Base(" + self.name + ")" end
        }
    }
    local Derived = { __index = { greet = function(self) return "yo " + self.name end } }
    set_meta(Derived, { __index = Base.__index })

    local b = { name = "bob" }
    set_meta(b, Base)
    local d = { name = "dan" }
    set_meta(d, Derived)
    print(b:greet())
    print(d:greet())
    print(d)
    "#;
    let out = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "hi bob");
    assert_eq!(lines[1], "yo dan");
    assert_eq!(lines[2], "Base(dan)");
}
