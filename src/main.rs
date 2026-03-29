use anyhow::Result;
use clap::Parser;
use honeyprompt::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(_args) => {
            println!("Init not yet implemented");
            Ok(())
        }
        Commands::Generate(_args) => {
            println!("Generate not yet implemented");
            Ok(())
        }
    }
}
