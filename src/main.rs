use log::Level;
use std::fs::OpenOptions;
use std::io::Read;
use log::*;
fn main() -> Result<(), failure::Error> {
    simple_logger::init_with_level(Level::Warn)?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("请指定要执行的源代码文件");
    }
    let s = std::env::current_dir()?.join(&args[1]);

    debug!("{:?}", s);
    let mut f = OpenOptions::new()
        .read(true)
        .open(s)?;

    let mut v = vec![];

    f.read_to_end(&mut v)?;
    let mut code = String::from_utf8(v)?;

    debug!("{:?}", code);
    chen_lang::run(code)?;
    Ok(())
}
