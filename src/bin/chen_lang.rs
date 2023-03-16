extern crate clap;

use clap_complete::{generate, Generator, Shell};
use std::fs::OpenOptions;
use std::io;
use std::io::Read;

use clap::{value_parser, Arg, ArgAction, Command};
use log::*;
use simple_logger::SimpleLogger;

fn main() -> Result<(), anyhow::Error> {
    let mut cmd = Command::new("chen_lang")
        .version("0.0.1")
        .author("zuisong <com.me@foxmail.com>")
        .about("a super tiny and toy language write by rust")
        .arg(
            Arg::new("run")
                .action(ArgAction::Set)
                .long("run")
                .help("要执行的源代码文件")
                .required(false),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .action(ArgAction::Count)
                .required(false)
                .help("v越多日志级别越低 (-vv is Info, -vvv is Debug)"),
        )
        .arg(
            Arg::new("generator")
                .required(false)
                .long("generate")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        );
    let matches = cmd.clone().get_matches();

    let log_level = match matches.get_count("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => LevelFilter::Trace,
    };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    if let Some(generator) = matches.get_one::<Shell>("generator").copied() {
        eprintln!("Generating completion file for {}...", generator);
        print_completions(generator, &mut cmd);
    }

    if let Some(code_file) = matches.get_one::<String>("run") {
        let s = std::env::current_dir()?.join(code_file);

        debug!("{:?}", s);
        let mut f = OpenOptions::new().read(true).open(s)?;

        let mut v = vec![];
        f.read_to_end(&mut v)?;
        let code = String::from_utf8(v)?;

        debug!("{:?}", code);
        chen_lang::run(code)?;
    }
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
