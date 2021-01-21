extern crate clap;

use std::fs::OpenOptions;
use std::io::Read;

use clap::{App, Arg};
use log::*;
use simple_logger::SimpleLogger;

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("chen_lang")
        .version("0.0.1")
        .author("zuisong <com.me@foxmail.com>")
        .about("a super tiny and toy language write by rust")
        .arg(
            Arg::with_name("INPUT_FILE")
                .help("要执行的源代码文件")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .required(false)
                .multiple(true)
                .help("v越多日志级别越低 (-vv is Info, -vvv is Debug)"),
        )
        .get_matches();
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => LevelFilter::Trace,
    };

    SimpleLogger::new().with_level(log_level).init().unwrap();

    let code_file = matches.value_of("INPUT_FILE").unwrap();
    let s = std::env::current_dir()?.join(code_file);

    debug!("{:?}", s);
    let mut f = OpenOptions::new().read(true).open(s)?;

    let mut v = vec![];
    f.read_to_end(&mut v)?;
    let code = String::from_utf8(v)?;

    debug!("{:?}", code);
    chen_lang::run(code)?;
    Ok(())
}
