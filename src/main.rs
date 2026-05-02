use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use take::player::{DEFAULT_COLS, DEFAULT_ROWS};
use take::renderer;

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
            let output = take_file
                .output
                .clone()
                .unwrap_or_else(|| "output.gif".to_string());
            let cols = take_file.cols.unwrap_or(DEFAULT_COLS);
            let rows = take_file.rows.unwrap_or(DEFAULT_ROWS);

            let frames = take::player::play(take_file).await?;
            println!("Rendering GIF...");

            let _ = renderer::export_gif(&frames, cols, rows, &output);
            println!("Done! Saved to {}", output);

            Ok(())
        }
    }
}
