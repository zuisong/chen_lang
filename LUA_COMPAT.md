# Lua 兼容特性

Chen Lang 使用 Luau/Lua 风格语法，并持续向 Lua 5.2 标准库和语言特性对齐。

## 语法特性

| 特性 | 示例 | 状态 |
| :--- | :--- | :--- |
| 变量声明 | `local x = 10` | ✅ |
| 多值声明 | `local a, b = f()` | ✅ |
| 多返回值 | `return a, b, c` | ✅ |
| 返回值透传 | `return f()` | ✅ |
| 变长参数 | `function f(...)` / `f(...)` / `{...}` / `select("#", ...)` | ✅ |
| 多目标赋值 | `a, b = b, a`（交换） | ✅ |
| 闭包 | `function() return count end` | ✅ |
| 表字面量 | `{ name = "Alice" }` | ✅ |
| 数组字面量 | `{ 1, 2, 3 }`（0-based） | ✅ |
| 泛型 for | `for k, v in pairs(t)` | ✅ |
| 数值 for | `for i = 1, 10, 2 do`（支持负步长） | ✅ |
| repeat-until | `repeat ... until cond` | ✅ |
| 字符串方法 | `s:upper()` / `s:sub(1, 3)` | ✅ |
| 三元 if 表达式 | `local x = if cond then a else b end` | ✅ |
| 幂运算 | `2 ^ 10`（右结合，`-2^2 == -4`） | ✅ |
| 复合赋值 | `x += 1`, `s ..= "!"` 等 | ✅ |

## 标准库

### 全局函数

| 函数 | 说明 |
| :--- | :--- |
| `type(v)` | 返回类型字符串（int/float/boolean/string/table/function/thread/nil） |
| `tostring(v)` | 转字符串（支持 `__tostring`） |
| `tonumber(s)` | 转数字，失败返回 nil |
| `pairs(t)` | 返回产生 `{key, value}` 对的迭代器 |
| `ipairs(t)` | 遍历数组部分 |
| `next(t, k)` | 返回 `(key, value)` 或 nil |
| `select(n, ...)` | 选择参数；`select("#", ...)` 返回个数 |
| `unpack(t)` | 展开数组为多值 |
| `pcall(f, ...)` | 安全调用，返回 `(ok, result)` |
| `xpcall(f, handler, ...)` | 带错误处理的安全调用 |
| `assert(v, msg)` | 断言 |
| `rawequal/rawget/rawset/rawlen` | 绕过元方法的原始操作 |
| `collectgarbage()` | 占位实现 |
| `_G`, `_VERSION` | 全局环境与版本 |

### table 库

| 函数 | 说明 |
| :--- | :--- |
| `table.insert(t, [pos,] v)` | 插入（0-based） |
| `table.remove(t, [pos])` | 删除并返回 |
| `table.concat(t, [sep])` | 连接 |
| `table.sort(t)` | 排序（数字/字符串升序） |
| `table.unpack(t)` | 展开 |
| `table.pack(...)` | 打包为数组（带 `n` 字段） |
| `table.getn(t)` | 长度 |

### string 库（亦可作为方法调用 `s:xxx()`）

| 函数 | 说明 |
| :--- | :--- |
| `string.len(s)` | 长度 |
| `string.sub(s, i, [j])` | 子串（1-based，支持负数） |
| `string.rep(s, n)` | 重复 |
| `string.upper/lower(s)` | 大小写 |
| `string.reverse(s)` | 反转 |
| `string.byte/char` | 字节码与字符 |
| `string.find(s, p)` | 查找，返回 `(start, end)` |
| `string.match(s, p)` | 匹配，返回捕获或完整匹配 |
| `string.gmatch(s, p)` | 迭代器 |
| `string.gsub(s, p, r)` | 替换，返回 `(新串, 次数)` |
| `string.format(fmt, ...)` | `%s %d %f %x %o %c %q %%`，支持 `%.2f` |

#### 模式匹配

支持 `%a %d %w %s %l %u %p %x %c`（大写为取反）、`.`、`*` `+` `-` `?` 量词、
`^` `$` 锚点、`[...]` 字符集（含范围 `a-z` 和 `%` 转义）、捕获 `(...)`、
`gsub` 中的 `%1` 反向引用。

### math 库

`pi`, `huge`, `abs`, `floor`, `ceil`, `max`, `min`, `sqrt`, `pow`, `exp`, `log`,
`sin/cos/tan/asin/acos/atan/atan2`, `deg/rad`, `random/randomseed`, `round`,
`sign`, `fmod`, `clamp`, `modf`。

### os 库

`time()`, `clock()`, `date(fmt)`, `getenv(name)`, `tmpname()`, `exit(code)`。

## 元方法

| 元方法 | 触发 |
| :--- | :--- |
| `__index` | 访问不存在字段（表或函数） |
| `__newindex` | 赋值不存在字段 |
| `__call` | `obj(...)` |
| `__tostring` | `print(obj)` / `tostring(obj)` |
| `__add/__sub/__mul/__div/__mod/__pow` | 算术运算 |
| `__concat` | `..` |
| `__eq` | `==` |
| `__lt` | `<` / `>`（经交换） |
| `__le` | `<=` / `>=`（经交换） |
| `__len` | `#` |

元方法查找会沿着元表链和 `__index` 链进行，因此支持继承式的方法/元方法复用。

## 已知限制

- 数组为 0-based（`{ "a" }` 的索引从 0 开始），与 Lua 的 1-based 不同
- `goto` 标签尚未实现
- 函数调用实参暂不支持"最后一个调用展开多个值"（除 `...` 外）
- `table.sort` 暂不支持自定义比较函数
