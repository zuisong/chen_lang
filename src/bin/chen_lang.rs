use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
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
    /// Start REPL
    Repl,
}

fn main() -> anyhow::Result<()> {
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
            SubCommand::Repl => repl()?,
        },
    }

    Ok(())
}

fn repl() -> anyhow::Result<()> {
    println!("Chen Lang REPL v0.1.0");
    println!("Type 'exit' or 'quit' to exit.");

    let mut vm = chen_lang::vm::VM::new();
    let mut global_program = chen_lang::vm::Program::default();

    // We need to keep track of total instructions for offset
    let mut offset = 0;

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut input = String::new();

    loop {
        print!("> ");
        stdout.flush()?;
        input.clear();
        if stdin.read_line(&mut input)? == 0 {
            break; // EOF
        }

        debug!("REPL input: {:?}", input);

        let trimmed = input.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        // Parse
        match chen_lang::parser::parse_from_source(&input) {
            Ok(ast) => {
                // Compile
                let chars: Vec<char> = input.chars().collect();
                let mut new_program = chen_lang::compiler::compile_with_offset(&chars, ast, offset);

                // Hack: If the last instruction is Pop, remove it to print the result
                if let Some(chen_lang::vm::Instruction::Pop) = new_program.instructions.last() {
                    new_program.instructions.pop();
                }

                let start_pc = global_program.instructions.len();

                // Adjust symbol locations
                for sym in new_program.syms.values_mut() {
                    sym.location += start_pc as i32;
                }

                // Adjust lines
                for (idx, line) in new_program.lines {
                    global_program.lines.insert(idx + start_pc, line);
                }

                // Merge instructions
                global_program.instructions.append(&mut new_program.instructions);

                // Merge symbols
                global_program.syms.extend(new_program.syms);

                // Execute
                vm.program = Some(std::rc::Rc::new(global_program.clone()));
                match vm.execute_from(start_pc) {
                    Ok(value) => {
                        if value != chen_lang::value::Value::Null {
                            println!("{}", value);
                        }
                    }
                    Err(e) => {
                        eprintln!("Runtime Error: {}", e);
                    }
                }

                offset = global_program.instructions.len();
            }
            Err(e) => {
                eprintln!("Parse Error: {}", e);
            }
        }
    }
    Ok(())
}

fn run_file(code_file: String) -> Result<(), chen_lang::ChenError> {
    let mut code = String::new();
    if code_file == "-" {
        io::stdin().read_to_string(&mut code)?;
    } else {
        let s = std::env::current_dir()?.join(&code_file);
        debug!(?s);
        let mut f = OpenOptions::new().read(true).open(s)?;
        f.read_to_string(&mut code)?;
    }
    debug!(?code);

    if let Err(e) = chen_lang::run(code.clone()) {
        let s = chen_lang::report_error(&code, &code_file, &e);
        eprintln!("{s}");
        std::process::exit(1);
    }
    Ok(())
}

fn print_completions<G: Generator>(g: G, cmd: &mut Command) {
    generate(g, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
