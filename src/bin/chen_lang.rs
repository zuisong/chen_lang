extern crate clap;
use std::{
    fs::OpenOptions,
    io::{self, Read},
};

use anyhow::{Ok, Result};
use clap::{builder::PossibleValuesParser, Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use tracing::{debug, Level};

use crate::clap::builder::TypedValueParser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[command(subcommand)]
    command: Option<SubCommand>,
    /// log level
    #[arg(short, long)]
    #[arg(default_value_t = Level::INFO)]
    #[arg(ignore_case = true)]
    #[arg(value_parser=
        PossibleValuesParser::new([ "ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
        .map(|s| s.parse::<Level>().unwrap()),
    )]
    log_level: Level,
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
    tracing_subscriber::fmt()
        .with_max_level(matches.log_level)
        .with_line_number(true)
        .with_file(true)
        .with_thread_names(true)
        .with_thread_ids(true)
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
