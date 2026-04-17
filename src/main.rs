use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "take", about = "Script and replay terminal sessions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read a `.take` script
    Play { file: std::path::PathBuf },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Play { file } => {
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read .take file: {}", file.display()))?;

            let take_file = take::parser::parse(&content).map_err(|e| anyhow::anyhow!(e))?;
            let frames = take::player::play(take_file).await?;
            println!("{} frames captured", frames.len());
            for (i, frame) in frames.iter().enumerate() {
                println!("\n--- frame {} ---\n{}", i + 1, frame.screen.contents());
            }
            Ok(())
        }
    }
}
