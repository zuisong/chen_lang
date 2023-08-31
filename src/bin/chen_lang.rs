extern crate clap;
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
};

use anyhow::{Ok, Result};
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use log::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[command(subcommand)]
    command: Option<SubCommand>,
    /// v越多日志级别越低 (-vv is Info, -vvv is Debug
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    /// Generate tab-completion scripts for your shell
    Completions {
        #[arg(long, short, value_enum)]
        shell: Shell,
    },
    /// Run
    Run {
        ///要执行的源代码文件
        code_file: String,
    },
}
fn main() -> Result<()> {
    let matches = Args::parse();
    let log_level = match matches.verbose {
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

    match matches.command {
        None => Args::command().print_help()?,
        Some(command) => match command {
            SubCommand::Completions { shell } => print_completions(shell, &mut Args::command()),
            SubCommand::Run { code_file } => run_file(code_file)?,
        },
    }

    Ok(())
}

fn run_file(code_file: String) -> Result<()> {
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
fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
