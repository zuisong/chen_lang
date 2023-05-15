extern crate clap;

use clap::{value_parser, Arg, ArgAction, Command};
use clap_complete::{generate, Generator, Shell};
use log::*;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
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
            Arg::new("completion")
                .required(false)
                .long("completion")
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level.as_str()))
        .default_format()
        .format(|buf, record| -> Result<(), io::Error> {
            let style = buf.style();
            let timestamp = buf.timestamp();
            writeln!(
                buf,
                "{} {} [{}:{}]: {}",
                record.level(),
                timestamp,
                record.file().unwrap_or(""),
                record.line().unwrap_or(0),
                style.value(record.args())
            )
        })
        .init();

    if let Some(shell) = matches.get_one::<Shell>("completion").copied() {
        eprintln!("Generating completion file for {}...", shell);
        print_completions(shell, &mut cmd);
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
