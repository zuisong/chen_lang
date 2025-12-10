use std::{
    fs::OpenOptions,
    io::{self, Read},
};

use clap::{
    Command, CommandFactory, Parser,
    builder::{PossibleValuesParser, TypedValueParser, ValueParser},
};
use clap_complete::{Generator, Shell, generate};
use tracing::{Level, debug, metadata::LevelFilter};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

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
    PossibleValuesParser::new(["ERROR", "WARN", "INFO", "DEBUG", "TRACE"]).try_map(|s| s.parse::<Level>())
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

fn run_file(code_file: String) -> Result<(), chen_lang::ChenError> {
    let s = std::env::current_dir()?.join(code_file);

    debug!(?s);
    let mut f = OpenOptions::new().read(true).open(s)?;

    let mut code = String::new();
    f.read_to_string(&mut code)?;
    debug!(?code);

    chen_lang::run(code)?;
    Ok(())
}

fn print_completions<G: Generator>(g: G, cmd: &mut Command) {
    generate(g, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
