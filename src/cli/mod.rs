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
    /// Start the live TUI event monitor
    Monitor(MonitorArgs),
    /// Generate a Markdown disclosure report from captured events
    Report(ReportArgs),
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

#[derive(Parser)]
pub struct MonitorArgs {
    /// Project directory containing honeyprompt.toml and output/
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Attach to a running server's database instead of starting a new server
    #[arg(long)]
    pub attach: bool,
    /// Port to bind the integrated server on (default from config)
    #[arg(long)]
    pub port: Option<u16>,
}

#[derive(Parser)]
pub struct ReportArgs {
    /// Project directory containing honeyprompt.toml and .honeyprompt/events.db
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Output file path (default: report.md in project directory)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Print report to stdout instead of writing a file
    #[arg(long)]
    pub stdout: bool,
}
