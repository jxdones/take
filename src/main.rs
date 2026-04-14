use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "roku", about = "terminal session recorder")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read a `.roku` script
    Play { file: std::path::PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Play { file } => {
            let _content = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read roku file: {}", file.display()))?;

            println!("Reading roku file from {}...", file.display());
            Ok(())
        }
    }
}
