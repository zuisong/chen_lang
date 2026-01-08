# Chen Lang 项目开发计划

## 核心目标：实现类似 Lua 的自定义类型系统 (基于 Table 和 Metatable)

### 设计理念
Chen Lang 的对象系统将模仿 Lua 的极简主义设计：
*   **数据结构**: 仅引入一种通用数据结构 **Table** (哈希表)，用于同时表示对象 (Object) 和 字典 (Map)。
*   **面向对象**: 不引入传统的 `class` 关键字。通过 **Metatable (元表)** 机制实现原型继承、运算符重载和自定义行为。
*   **语法支持**: 支持对象字面量 `${ k: v }`，属性访问 `obj.field` 和索引访问 `obj[index]`。

---

## 详细实施方案与进度

### 第一阶段：Value 系统改造 (基础层) - [已完成 ✅]
**目标**: 在底层 `Value` 枚举中支持 `Table` 结构。

*   **设计**:
    *   `Table` 结构体:
        ```rust
        pub struct Table {
            pub data: HashMap<String, Value>,
            pub metatable: Option<Rc<RefCell<Table>>>, // 预留给 Metatable
        }
        ```
    *   `Value` 枚举新增变体 `Object(Rc<RefCell<Table>>)`。使用 `Rc<RefCell<...>>` 是为了支持共享引用和内部可变性（多个变量指向同一个对象，且可以修改其属性）。
*   **实现细节**:
    *   已修改 `src/value.rs`。
    *   `Display` trait 已更新，对象打印为 `{k: v, ...}`。
    *   `PartialEq` 已更新，对象比较采用指针相等性 (`Rc::ptr_eq`)。

### 第二阶段：语法与解析器扩展 (前端层) - [已完成 ✅]
**目标**: 让 Parser 能识别对象相关的语法。

*   **设计**:
    *   **对象字面量**: `${ key: val, key2: val2 }`。使用 `${` 而不是 `{` 是为了避免与代码块 `Block` 的歧义。
    *   **属性访问**: `obj.field`。
    *   **索引访问**: `obj["field"]` 或 `obj[expr]`。
    *   **赋值目标**: 支持 `obj.field = val` 和 `obj[expr] = val` 作为赋值语句的左值。
*   **实现细节**:
    *   **Token**: `src/token.rs` 新增 `Token::Dot` (.) 和 `Token::HashLBig` (${)。
    *   **AST**: `src/expression.rs` 新增 `ObjectLiteral`, `GetField`, `Index` (Expression) 和 `SetField`, `SetIndex` (Statement)。
    *   **Pest Parser**: 更新 `src/chen.pest` 和 `src/parse_pest.rs`，重构 `primary` 规则以支持后缀表达式 (`atom ~ postfix*`)。
    *   **Handwritten Parser**: 更新 `src/parse.rs`，重构 `parse_primary` 并新增 `parse_postfix_expr` 以支持链式调用和成员访问。

### 第三阶段：编译器与指令生成 (编译层) - [已完成 ✅]
**目标**: 将新的 AST 节点编译为字节码指令。

*   **设计**:
    需要引入新的 VM 指令来操作对象。
    *   `NewObject`: 创建空 Table 压栈。
    *   `SetField(String)`: 弹出 value, object -> `object.data[key] = value`。
    *   `GetField(String)`: 弹出 object -> 压入 `object.data[key]`。
    *   `SetIndex`: 弹出 value, index, object -> `object.data[index.to_string()] = value`。
    *   `GetIndex`: 弹出 index, object -> 压入 `object.data[index.to_string()]`。

*   **实现计划**:
    1.  在 `src/vm.rs` 的 `Instruction` 枚举中添加上述指令。
    2.  在 `src/compiler.rs` 中实现编译逻辑。

### 第四阶段：虚拟机运行时 (执行层) - [已完成 ✅]
**目标**: 在 VM 中实现对象的操作逻辑，包括 Metatable 的支持。

*   **设计**:
    *   **基础操作**: `GetField`/`SetField` 直接读写 `Table.data` (HashMap)。
    *   **元表 (Metatable) 支持 (核心难点)**:
        *   **读取 (`GetField`)**:
            如果 `object.data` 中找不到 key：
            1.  检查 `object.metatable` 是否存在。
            2.  如果存在，查找 metatable 中的 `__index` 字段。
            3.  如果 `__index` 是 Table，递归查找。✅
            4.  如果 `__index` 是 Function，调用它 `call(__index, object, key)`。(已支持)
        *   **写入 (`SetField`)**:
            如果 `object.data` 中找不到 key 且存在 `__newindex` 元方法，则调用之。(未来功能)
        *   **运算符重载 (`Add`, `Sub` 等)**:
            修改 `Value::add` 等方法。如果操作数不是基本类型，检查是否有 `__add` 元方法并调用。✅

*   **实现状态**:
    1.  ✅ 在 `src/vm.rs` 的 `execute_instruction` 中实现基础指令。
    2.  ✅ 在 `src/value.rs` 中实现 `get_field_with_meta` 和 `set_field_with_meta` 逻辑。
    3.  ✅ 添加内置函数 `set_meta()` 和 `get_meta()`。
    4.  ✅ **运算符重载**: 已实现 `+` (`__add`), `-` (`__sub`), `*` (`__mul`) 的重载支持。当操作数包含元方法时，VM 会自动将其转换为函数调用。

### 第五阶段：标准库与用户侧 (应用层)
**目标**: 暴露 `set_meta` 等函数，让用户能定义“类”。

*   **设计**:
    *   内置函数 `set_meta(obj, meta)`: 设置对象的元表。
    *   内置函数 `get_meta(obj)`: 获取对象的元表。

*   **用户代码示例 (最终效果)**:
    ```chen
    let Person = ${
        __index: ${
            say_hi: def(self) { println("Hi " + self.name) }
        }
    }
    
    def new_person(name) {
        let p = ${ name: name }
        set_meta(p, Person)
        return p
    }
    
    let p = new_person("Chen")
    p.say_hi()
    ```

---
**当前状态**: 
*   **对象系统**: 基础对象、元表、原型继承 (`__index`)、运算符重载 (`__add`, `__sub`, `__mul`) 均已实现并测试通过。
*   **编译器/VM**: 修复了全局变量解析、内置函数调用优化以及元方法调用时的指令流控制问题。
*   **下一步**: 继续完善标准库，或考虑添加更多元方法（如 `__tostring`, `__call` 等）。
