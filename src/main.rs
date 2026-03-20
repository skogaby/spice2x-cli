mod cli;
mod commands;
mod output;
mod protocol;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, OutputFormat};
use output::{Formatter, JsonFormatter, TextFormatter};
use protocol::Connection;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let mut conn = Connection::new(&cli.host, cli.port, &cli.password)?;
    let result = commands::execute(&mut conn, cli.command)?;

    let formatter: Box<dyn Formatter> = match cli.format {
        OutputFormat::Json => Box::new(JsonFormatter),
        OutputFormat::Text => Box::new(TextFormatter),
    };

    println!("{}", formatter.format(&result));
    Ok(())
}
