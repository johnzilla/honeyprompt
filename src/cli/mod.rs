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
