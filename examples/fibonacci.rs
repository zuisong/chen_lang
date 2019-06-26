fn main() {
    simple_logger::init().unwrap();
    let cpde: String = r#"
n = 1
i = 1
j = 2
for n <= 30 {
   println(i)
   tmp = i
   i = j
   j = tmp + j
   n = n + 1
}
"#
    .to_string();

    chen_lang::run(cpde).unwrap();
}
