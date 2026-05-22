use std::cell::RefCell;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::value::{Value, Table};

pub fn create_symbol_object() -> Value {
    let mut data = IndexMap::new();
    
    // In Chen Lang, we implement Symbols as unique string constants for simplicity, 
    // similar to early JS polyfills, while keeping the API surface identical to JS.
    data.insert("iterator".to_string(), Value::string("@@iterator".to_string()));
    data.insert("asyncIterator".to_string(), Value::string("@@asyncIterator".to_string()));
    
    let table = Table {
        data,
        metatable: None,
    };
    
    Value::Object(Rc::new(RefCell::new(table)))
}
