use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::vm::native_coroutine::native_coroutine_yield;

/// 创建 Lua 风格的 `string` 库对象。
pub fn create_string_library() -> Value {
    let mut table = crate::value::Table {
        data: IndexMap::new(),
        metatable: None,
    };
    table
        .data
        .insert("__type".to_string(), Value::string("String".to_string()));
    table.data.insert(
        "len".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_len) as Box<NativeFnType>)),
    );
    table.data.insert(
        "sub".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_sub) as Box<NativeFnType>)),
    );
    table.data.insert(
        "rep".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_rep) as Box<NativeFnType>)),
    );
    table.data.insert(
        "byte".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_byte) as Box<NativeFnType>)),
    );
    table.data.insert(
        "char".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_char) as Box<NativeFnType>)),
    );
    table.data.insert(
        "reverse".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_reverse) as Box<NativeFnType>)),
    );
    table.data.insert(
        "upper".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_upper) as Box<NativeFnType>)),
    );
    table.data.insert(
        "lower".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_lower) as Box<NativeFnType>)),
    );
    table.data.insert(
        "trim".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_trim) as Box<NativeFnType>)),
    );
    table.data.insert(
        "find".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_find) as Box<NativeFnType>)),
    );
    table.data.insert(
        "match".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_match) as Box<NativeFnType>)),
    );
    table.data.insert(
        "gmatch".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_gmatch) as Box<NativeFnType>)),
    );
    table.data.insert(
        "gsub".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_gsub) as Box<NativeFnType>)),
    );
    table.data.insert(
        "format".to_string(),
        Value::NativeFunction(Rc::new(Box::new(native_string_format) as Box<NativeFnType>)),
    );

    Value::Object(Rc::new(RefCell::new(table)))
}

fn as_string_arg(args: &[Value], idx: usize) -> Result<&str, VMRuntimeError> {
    match args.get(idx) {
        Some(Value::String(s)) => Ok(s.as_str()),
        Some(v) => Err(ValueError::TypeMismatch {
            expected: ValueType::String,
            found: v.get_type(),
            operation: "string".into(),
        }
        .into()),
        None => Ok(""),
    }
}

fn as_int_arg(args: &[Value], idx: usize, default: i32) -> i32 {
    args.get(idx).and_then(|v| v.to_int()).unwrap_or(default)
}

pub fn native_string_len(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    Ok(Value::Int(s.chars().count() as i32))
}

/// Lua 1-based 索引规范化为 Rust 0-based 索引（支持负数）
fn normalize_index(len: i64, mut i: i64, default: Option<i64>) -> i64 {
    if i == 0 {
        i = default.unwrap_or(0);
    }
    if i < 0 {
        i += len + 1;
    }
    i
}

/// `string.sub(s, i [, j])`，索引 1-based，支持负数
pub fn native_string_sub(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    let start = as_int_arg(&args, 1, 1) as i64;
    let end = as_int_arg(&args, 2, len as i32) as i64;

    let mut s_idx = normalize_index(len, start, Some(1));
    let mut e_idx = normalize_index(len, end, Some(len));

    if s_idx < 1 {
        s_idx = 1;
    }
    if e_idx > len {
        e_idx = len;
    }
    if s_idx > e_idx {
        return Ok(Value::string(String::new()));
    }

    let result: String = chars[(s_idx - 1) as usize..e_idx as usize].iter().collect();
    Ok(Value::string(result))
}

pub fn native_string_rep(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    let n = as_int_arg(&args, 1, 0).max(0) as usize;
    Ok(Value::string(s.repeat(n)))
}

/// `string.byte(s [, i [, j]])`，返回字符的字节码
pub fn native_string_byte(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    let bytes: Vec<u8> = s.bytes().collect();
    let len = bytes.len() as i64;

    let start = as_int_arg(&args, 1, 1) as i64;
    let end = as_int_arg(&args, 2, start as i32) as i64;

    let mut s_idx = normalize_index(len, start, Some(1));
    let mut e_idx = normalize_index(len, end, Some(s_idx));

    if s_idx < 1 {
        s_idx = 1;
    }
    if e_idx > len {
        e_idx = len;
    }
    if s_idx > e_idx {
        return Ok(Value::Null);
    }

    let values: Vec<Value> = bytes[(s_idx - 1) as usize..e_idx as usize]
        .iter()
        .map(|b| Value::Int(*b as i32))
        .collect();
    if values.len() == 1 {
        Ok(values[0].clone())
    } else {
        let mut data = IndexMap::new();
        for (i, b) in values.iter().enumerate() {
            data.insert((i + 1).to_string(), b.clone());
        }
        Ok(Value::Object(Rc::new(RefCell::new(crate::value::Table {
            data,
            metatable: None,
        }))))
    }
}

pub fn native_string_char(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let mut result = String::new();
    for arg in &args {
        if let Some(n) = arg.to_int()
            && let Some(c) = char::from_u32(n as u32)
        {
            result.push(c);
        }
    }
    Ok(Value::string(result))
}

pub fn native_string_reverse(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    Ok(Value::string(s.chars().rev().collect::<String>()))
}

pub fn native_string_upper(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    Ok(Value::string(s.to_uppercase()))
}

pub fn native_string_lower(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    Ok(Value::string(s.to_lowercase()))
}

pub fn native_string_trim(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?;
    Ok(Value::string(s.trim().to_string()))
}

/// `string.find(s, pattern [, init [, plain]])`，返回 (start, end) 或 nil
pub fn native_string_find(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?.to_string();
    let pattern = as_string_arg(&args, 1)?.to_string();
    let init = as_int_arg(&args, 2, 1);
    let plain = args.get(3).map(|v| v.is_truthy()).unwrap_or(false);

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let mut start_idx = normalize_index(len, init as i64, Some(1)).max(1);

    if plain {
        let pat_chars: Vec<char> = pattern.chars().collect();
        while start_idx <= len {
            let from = (start_idx - 1) as usize;
            let mut matched = true;
            for (k, pc) in pat_chars.iter().enumerate() {
                if from + k >= chars.len() || &chars[from + k] != pc {
                    matched = false;
                    break;
                }
            }
            if matched {
                let start = start_idx as i32;
                let end = (start_idx - 1 + pat_chars.len() as i64) as i32;
                vm.stack.push(Value::Int(start));
                vm.stack.push(Value::Int(end));
                return Ok(Value::Null);
            }
            start_idx += 1;
        }
        return Ok(Value::Null);
    }

    match find_match(&s, &pattern, (start_idx - 1) as usize) {
        Some((start, end, captures)) => {
            let start_val = Value::Int((start + 1) as i32);
            let end_val = Value::Int(end as i32);
            vm.stack.push(start_val);
            vm.stack.push(end_val);
            for cap in captures {
                vm.stack.push(Value::string(cap));
            }
            Ok(Value::Null)
        }
        None => Ok(Value::Null),
    }
}

/// `string.match(s, pattern)`，返回匹配的子串或第一个捕获
pub fn native_string_match(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?.to_string();
    let pattern = as_string_arg(&args, 1)?.to_string();

    match find_match(&s, &pattern, 0) {
        Some((start, end, captures)) => {
            if !captures.is_empty() {
                Ok(Value::string(captures[0].clone()))
            } else {
                let m: String = s.chars().skip(start).take(end - start).collect();
                Ok(Value::string(m))
            }
        }
        None => Ok(Value::Null),
    }
}

/// `string.gmatch(s, pattern)`，返回迭代协程，依次产生每个匹配
pub fn native_string_gmatch(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?.to_string();
    let pattern = as_string_arg(&args, 1)?.to_string();

    let matches: Vec<Value> = collect_matches(&s, &pattern)
        .into_iter()
        .map(|m| Value::string(m))
        .collect();
    let index = Rc::new(RefCell::new(0));

    let iter_body = move |vm: &mut VM, _args: Vec<Value>| {
        let mut idx = index.borrow_mut();
        if *idx < matches.len() {
            let val = matches[*idx].clone();
            *idx += 1;
            return native_coroutine_yield(vm, vec![val]);
        }
        Ok(Value::Null)
    };

    let mut fiber = Fiber::new();
    let nf_rc = Rc::new(Box::new(iter_body) as Box<NativeFnType>);
    fiber.native_function = Some(nf_rc.clone());
    fiber.stack.push(Value::NativeFunction(nf_rc));
    Ok(Value::Coroutine(Rc::new(RefCell::new(fiber))))
}

/// `string.gsub(s, pattern, repl)`，返回 (新字符串, 替换次数)
pub fn native_string_gsub(vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let s = as_string_arg(&args, 0)?.to_string();
    let pattern = as_string_arg(&args, 1)?.to_string();
    let repl = args.get(2).cloned().unwrap_or(Value::Null);

    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut count = 0i32;
    let mut pos = 0usize;

    let repl_is_string = matches!(&repl, Value::String(_));
    let repl_str = repl.to_string();

    loop {
        match find_match_at(&chars, &pattern, pos) {
            Some((start, end, captures)) => {
                result.extend(chars[pos..start].iter());
                if repl_is_string {
                    result.push_str(&expand_replacement(&repl_str, &captures));
                } else {
                    result.push_str(&repl_str);
                }
                count += 1;
                let next = if end > start { end } else { end + 1 };
                pos = next;
                if pos > chars.len() {
                    break;
                }
            }
            None => {
                result.extend(chars[pos..].iter());
                break;
            }
        }
    }

    vm.stack.push(Value::string(result));
    Ok(Value::Int(count))
}

/// `string.format(fmt, ...)`，支持 %s %d %i %f %x %X %o %c %q %% 和 * 宽度
pub fn native_string_format(_vm: &mut VM, args: Vec<Value>) -> Result<Value, VMRuntimeError> {
    let fmt = as_string_arg(&args, 0)?;
    let mut arg_idx = 1;
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c != '%' {
            result.push(c);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            result.push('%');
            break;
        }

        // 解析宽度/精度标志
        let mut spec = String::new();
        let mut has_star = false;
        let mut is_float_precision = false;
        while i < chars.len() {
            let sc = chars[i];
            if sc.is_ascii_digit() || sc == '.' || sc == '-' || sc == '+' || sc == ' ' || sc == '#' || sc == '0' {
                if sc == '.' {
                    is_float_precision = true;
                }
                spec.push(sc);
                i += 1;
            } else if sc == '*' {
                has_star = true;
                i += 1;
            } else {
                break;
            }
        }

        if i >= chars.len() {
            break;
        }
        let conv = chars[i];
        i += 1;

        let mut get_arg = || -> Value {
            let v = args.get(arg_idx).cloned().unwrap_or(Value::Null);
            arg_idx += 1;
            v
        };

        if has_star {
            let _ = get_arg();
        }

        match conv {
            's' => {
                let v = get_arg();
                result.push_str(&v.to_string());
            }
            'd' | 'i' => {
                let v = get_arg();
                let n = v.to_int().unwrap_or(0);
                result.push_str(&format_digits(&spec, n as i64));
            }
            'f' => {
                let v = get_arg();
                let d = v.to_float().unwrap_or_else(|| rust_decimal::Decimal::ZERO);
                let precision = if is_float_precision {
                    spec.split('.').last().and_then(|s| s.parse::<usize>().ok())
                } else {
                    None
                };
                result.push_str(&format_decimal(&spec, d, precision));
            }
            'x' => {
                let v = get_arg();
                result.push_str(&format!("{:x}", v.to_int().unwrap_or(0).max(0)));
            }
            'X' => {
                let v = get_arg();
                result.push_str(&format!("{:X}", v.to_int().unwrap_or(0).max(0)));
            }
            'o' => {
                let v = get_arg();
                result.push_str(&format!("{:o}", v.to_int().unwrap_or(0).max(0)));
            }
            'c' => {
                let v = get_arg();
                if let Some(n) = v.to_int()
                    && let Some(c) = char::from_u32(n as u32)
                {
                    result.push(c);
                }
            }
            'q' => {
                let v = get_arg();
                result.push_str(&format!("\"{}\"", v.to_string()));
            }
            'e' | 'E' | 'g' | 'G' => {
                let v = get_arg();
                let d = v.to_float().unwrap_or_else(|| rust_decimal::Decimal::ZERO);
                if let Some(f) = d.to_f64() {
                    result.push_str(&format!("{:e}", f));
                }
            }
            '%' => result.push('%'),
            _ => {
                result.push('%');
                result.push(conv);
            }
        }
    }

    Ok(Value::string(result))
}

fn format_digits(spec: &str, n: i64) -> String {
    let width = spec.parse::<usize>().unwrap_or(0);
    let s = format!("{}", n);
    if s.len() >= width {
        s
    } else {
        format!("{}{}", " ".repeat(width - s.len()), s)
    }
}

fn format_decimal(_spec: &str, d: rust_decimal::Decimal, precision: Option<usize>) -> String {
    if let Some(p) = precision {
        format!("{:.*}", p, d)
    } else {
        d.normalize().to_string()
    }
}

fn expand_replacement(repl: &str, captures: &[String]) -> String {
    let chars: Vec<char> = repl.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if let Some(d) = next.to_digit(10) {
                if (d as usize) <= captures.len() {
                    result.push_str(&captures[(d - 1) as usize]);
                }
                i += 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// 在字符串中查找 pattern 的所有匹配（返回完整匹配文本，若有捕获则返回第一个捕获）
fn collect_matches(s: &str, pattern: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut matches = Vec::new();
    let mut pos = 0usize;
    loop {
        match find_match_at(&chars, pattern, pos) {
            Some((start, end, captures)) => {
                if !captures.is_empty() {
                    matches.push(captures[0].clone());
                } else {
                    let m: String = chars[start..end].iter().collect();
                    matches.push(m);
                }
                let next = if end > start { end } else { end + 1 };
                pos = next;
                if pos > chars.len() {
                    break;
                }
            }
            None => break,
        }
    }
    matches
}

/// 查找匹配，返回 (start, end, captures)。start/end 是字符索引。
fn find_match(s: &str, pattern: &str, from: usize) -> Option<(usize, usize, Vec<String>)> {
    let chars: Vec<char> = s.chars().collect();
    find_match_at(&chars, pattern, from)
}

fn find_match_at(chars: &[char], pattern: &str, from: usize) -> Option<(usize, usize, Vec<String>)> {
    let compiled = compile_pattern(pattern)?;
    let anchored_start = matches!(compiled.first(), Some((PatElem::Anchor(Anchor::Start), _)));
    let elements: Vec<(PatElem, Quant)> = if anchored_start {
        compiled[1..].to_vec()
    } else {
        compiled
    };

    let mut pos = from.min(chars.len());
    if anchored_start {
        let (end, caps) = match_elements(chars, &elements, pos)?;
        return Some((pos, end, caps));
    }

    while pos <= chars.len() {
        if let Some((end, caps)) = match_elements(chars, &elements, pos) {
            return Some((pos, end, caps));
        }
        pos += 1;
    }
    None
}

#[derive(Clone, PartialEq)]
enum Quant {
    One,
    Star,  // 0+ 贪婪
    Plus,  // 1+ 贪婪
    Minus, // 0+ 非贪婪
    Q,     // 0 或 1
}

#[derive(Clone)]
enum Anchor {
    Start,
    End,
}

#[derive(Clone)]
enum PatElem {
    Lit(char),
    Any,
    /// `%d` 等字符类；negate=true 表示 `%D`
    Class {
        classes: Vec<fn(char) -> bool>,
        negate: bool,
    },
    /// `[set]`
    Set {
        chars: Vec<char>,
        ranges: Vec<(char, char)>,
        classes: Vec<fn(char) -> bool>,
        negate: bool,
    },
    /// 捕获组 `(...)`，内部元素已编译
    Capture(Vec<(PatElem, Quant)>),
    Anchor(Anchor),
}

fn compile_pattern(pattern: &str) -> Option<Vec<(PatElem, Quant)>> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut elems: Vec<(PatElem, Quant)> = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let elem: PatElem = match c {
            '^' => PatElem::Anchor(Anchor::Start),
            '$' => PatElem::Anchor(Anchor::End),
            '.' => PatElem::Any,
            '%' => {
                if i + 1 >= chars.len() {
                    return None;
                }
                let nc = chars[i + 1];
                i += 1;
                if let Some(f) = elem_class_fn(nc) {
                    PatElem::Class {
                        classes: vec![f],
                        negate: false,
                    }
                } else if let Some(f) = elem_neg_class_fn(nc) {
                    PatElem::Class {
                        classes: vec![f],
                        negate: true,
                    }
                } else {
                    // %x 转义字面字符
                    PatElem::Lit(nc)
                }
            }
            '[' => {
                let mut j = i + 1;
                let mut negate = false;
                if j < chars.len() && chars[j] == '^' {
                    negate = true;
                    j += 1;
                }
                let mut set_chars = Vec::new();
                let mut ranges = Vec::new();
                let mut classes = Vec::new();
                let mut closed = false;
                while j < chars.len() {
                    if chars[j] == ']' {
                        closed = true;
                        break;
                    }
                    let a = chars[j];
                    if a == '%' && j + 1 < chars.len() {
                        let nc = chars[j + 1];
                        if let Some(f) = elem_class_fn(nc) {
                            classes.push(f);
                        }
                        j += 2;
                        continue;
                    }
                    if j + 2 < chars.len() && chars[j + 1] == '-' {
                        ranges.push((a, chars[j + 2]));
                        j += 3;
                        continue;
                    }
                    set_chars.push(a);
                    j += 1;
                }
                if !closed {
                    return None;
                }
                i = j;
                PatElem::Set {
                    chars: set_chars,
                    ranges,
                    classes,
                    negate,
                }
            }
            '(' => {
                let mut j = i + 1;
                let mut depth = 1;
                while j < chars.len() {
                    if chars[j] == '(' {
                        depth += 1;
                    } else if chars[j] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if j >= chars.len() {
                    return None;
                }
                let inner: String = chars[i + 1..j].iter().collect();
                let inner_elems = compile_pattern(&inner)?;
                i = j;
                PatElem::Capture(inner_elems)
            }
            c => PatElem::Lit(c),
        };

        // 量词
        let quant = if i + 1 < chars.len() {
            match chars[i + 1] {
                '*' => Some(Quant::Star),
                '+' => Some(Quant::Plus),
                '-' => Some(Quant::Minus),
                '?' => Some(Quant::Q),
                _ => None,
            }
        } else {
            None
        };

        if let Some(q) = quant {
            elems.push((elem, q));
            i += 2;
        } else {
            elems.push((elem, Quant::One));
            i += 1;
        }
    }

    Some(elems)
}

/// 从 pos 开始匹配元素列表，返回 (结束位置, 捕获文本列表)。带回溯。
fn match_elements(chars: &[char], elems: &[(PatElem, Quant)], pos: usize) -> Option<(usize, Vec<String>)> {
    if elems.is_empty() {
        return Some((pos, Vec::new()));
    }
    let (elem, quant) = &elems[0];
    let rest = &elems[1..];

    match quant {
        Quant::One => {
            if let Some((end, mut caps)) = match_elem(chars, elem, pos) {
                let (rend, rcaps) = match_elements(chars, rest, end)?;
                caps.extend(rcaps);
                Some((rend, caps))
            } else {
                None
            }
        }
        Quant::Q => {
            if let Some((end, caps)) = match_elem(chars, elem, pos) {
                if let Some((rend, rcaps)) = match_elements(chars, rest, end) {
                    let mut all = caps;
                    all.extend(rcaps);
                    return Some((rend, all));
                }
            }
            match_elements(chars, rest, pos)
        }
        Quant::Star | Quant::Plus | Quant::Minus => {
            // 收集所有可能的重复结束点（从多到少）
            let mut attempts: Vec<(usize, Vec<String>)> = Vec::new();
            let mut cur = pos;
            let mut cur_caps: Vec<String> = Vec::new();
            let min = if matches!(quant, Quant::Plus) { 1 } else { 0 };
            let mut count = 0;
            if min == 0 {
                attempts.push((cur, cur_caps.clone()));
            }
            while let Some((end, caps)) = match_elem(chars, elem, cur) {
                cur = end;
                count += 1;
                cur_caps.extend(caps);
                if count >= min {
                    attempts.push((cur, cur_caps.clone()));
                }
                if cur >= chars.len() {
                    break;
                }
            }

            let greedy = !matches!(quant, Quant::Minus);
            if greedy {
                for (end, caps) in attempts.into_iter().rev() {
                    if let Some((rend, rcaps)) = match_elements(chars, rest, end) {
                        let mut all = caps;
                        all.extend(rcaps);
                        return Some((rend, all));
                    }
                }
                None
            } else {
                for (end, caps) in attempts {
                    if let Some((rend, rcaps)) = match_elements(chars, rest, end) {
                        let mut all = caps;
                        all.extend(rcaps);
                        return Some((rend, all));
                    }
                }
                None
            }
        }
    }
}

/// 匹配单个元素，返回 (结束位置, 捕获)
fn match_elem(chars: &[char], elem: &PatElem, pos: usize) -> Option<(usize, Vec<String>)> {
    if pos >= chars.len() {
        return None;
    }
    match elem {
        PatElem::Lit(c) => {
            if chars[pos] == *c {
                Some((pos + 1, Vec::new()))
            } else {
                None
            }
        }
        PatElem::Any => Some((pos + 1, Vec::new())),
        PatElem::Class { classes, negate } => {
            let m = classes.iter().any(|f| f(chars[pos]));
            if m != *negate {
                Some((pos + 1, Vec::new()))
            } else {
                None
            }
        }
        PatElem::Set {
            chars: set_chars,
            ranges,
            classes,
            negate,
        } => {
            let c = chars[pos];
            let m = set_chars.contains(&c)
                || ranges.iter().any(|(a, b)| c >= *a && c <= *b)
                || classes.iter().any(|f| f(c));
            if m != *negate {
                Some((pos + 1, Vec::new()))
            } else {
                None
            }
        }
        PatElem::Capture(inner) => {
            if let Some((end, _)) = match_elements(chars, inner, pos) {
                let text: String = chars[pos..end].iter().collect();
                Some((end, vec![text]))
            } else {
                None
            }
        }
        PatElem::Anchor(_) => None,
    }
}

/// 字符类匹配函数（%d, %a 等）
fn elem_class_fn(elem: char) -> Option<fn(char) -> bool> {
    match elem {
        'a' => Some(|c: char| c.is_alphabetic()),
        'd' => Some(|c: char| c.is_ascii_digit()),
        'w' => Some(|c: char| c.is_alphanumeric()),
        's' => Some(|c: char| c.is_whitespace()),
        'l' => Some(|c: char| c.is_lowercase()),
        'u' => Some(|c: char| c.is_uppercase()),
        'p' => Some(|c: char| c.is_ascii_punctuation()),
        'x' => Some(|c: char| c.is_ascii_hexdigit()),
        'c' => Some(|c: char| c.is_ascii_control()),
        _ => None,
    }
}

/// 取反字符类（%A, %D 等）
fn elem_neg_class_fn(elem: char) -> Option<fn(char) -> bool> {
    match elem {
        'A' => Some(|c: char| !c.is_alphabetic()),
        'D' => Some(|c: char| !c.is_ascii_digit()),
        'W' => Some(|c: char| !c.is_alphanumeric()),
        'S' => Some(|c: char| !c.is_whitespace()),
        'L' => Some(|c: char| !c.is_lowercase()),
        'U' => Some(|c: char| !c.is_uppercase()),
        'P' => Some(|c: char| !c.is_ascii_punctuation()),
        'X' => Some(|c: char| !c.is_ascii_hexdigit()),
        'C' => Some(|c: char| !c.is_ascii_control()),
        _ => None,
    }
}
