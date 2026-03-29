use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "honeyprompt", version, about = "Detect and measure unsafe behavior by AI browsing agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new honeyprompt project
    Init(InitArgs),
    /// Generate honeypot output files
    Generate(GenerateArgs),
    /// Start the honeypot HTTP server
    Serve(ServeArgs),
}

#[derive(Parser)]
pub struct InitArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct GenerateArgs {
    /// Project directory containing honeyprompt.toml
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct ServeArgs {
    /// Project directory containing honeyprompt.toml and output/
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Output events as JSON lines instead of structured text
    #[arg(long)]
    pub json: bool,
}
