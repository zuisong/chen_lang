use std::{
    fs::OpenOptions,
    io::{self, Read},
};

use anyhow::{Ok, Result};
use clap::{
    builder::{PossibleValuesParser, TypedValueParser, ValueParser},
    Command, CommandFactory, Parser,
};
use clap_complete::{generate, Generator, Shell};
use tracing::{debug, metadata::LevelFilter, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[command(subcommand)]
    command: Option<SubCommand>,
    /// log level
    #[arg(short, long)]
    #[arg(ignore_case = true)]
    #[arg(default_value_t = Level::INFO, value_parser = level_parser())]
    log_level: Level,
}

fn level_parser() -> impl Into<ValueParser> {
    PossibleValuesParser::new(["ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
        .try_map(|s| s.parse::<Level>())
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

#[cfg(test)]
mod tests {
    #[test]
    fn _test() {
        assert_cmd::Command::new("cargo")
            .arg("build")

            .ok();
    }

    #[test]
    fn cmd_test() {
        assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .args(&["-h"])
            .ok();
    }
}

fn main() -> Result<()> {
    let matches = Args::parse();
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_filter(LevelFilter::from_level(matches.log_level)),
        )
        .try_init();

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

    debug!(?s);
    let mut f = OpenOptions::new().read(true).open(s)?;

    let mut v = vec![];
    f.read_to_end(&mut v)?;
    let code = String::from_utf8(v)?;

    debug!(?code);
    chen_lang::run(code)?;
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
