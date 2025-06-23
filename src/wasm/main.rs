use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn main() {}

fn string_to_ptr(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub fn eval(input_ptr: *mut c_char) -> *mut c_char {
    let input = unsafe { CStr::from_ptr(input_ptr).to_string_lossy().into_owned() };

    if let Ok(_) = chen_lang::run(input) {}

    string_to_ptr("OK".to_string())
}
