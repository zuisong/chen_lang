use std::collections::HashMap;

fn main() {
    let a: HashMap<String, String> = HashMap::new();

    let b: HashMap<String, String> = a;
    f1(b);

    //    println!("{:?}", a);
    //    println!("{:?}", b);

    println!("hello world")
}

fn f1(aa: HashMap<String, String>) {}

fn f2(aa: &HashMap<String, String>) {}

fn f3(aa: &mut HashMap<String, String>) {}

fn f4(aa: *const HashMap<String, String>) {
    unsafe {
        let bb = &*aa;
    };
}

fn f5(aa: *mut HashMap<String, String>) {
    unsafe {
        let mut bb = &*aa;
    };
}
