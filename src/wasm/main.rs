extern crate chen_lang;

use std::ffi::{CStr, CString};
use std::mem;
use std::{
    os::raw::{c_char, c_void},
    rc::Rc,
};

fn main() {}

fn string_to_ptr(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub fn eval(input_ptr: *mut c_char) -> *mut c_char {
    let input = unsafe { CStr::from_ptr(input_ptr).to_string_lossy().into_owned() };

    match chen_lang::run(input) {
        Ok(_) => {}
        Err(_) => {}
    }

    string_to_ptr("OK".to_string())
}
