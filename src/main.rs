use anyhow::{bail, Result};
use clap::Parser;
use honeyprompt::cli::{Cli, Commands};
use honeyprompt::{config, generator, monitor, report, server, store};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => {
            let path = &args.path;

            // Check that we're not re-initializing an existing project
            let config_path = path.join("honeyprompt.toml");
            if config_path.exists() {
                bail!("honeyprompt.toml already exists — project already initialized");
            }

            // Create output/ directory
            std::fs::create_dir_all(path.join("output"))?;

            // Create .honeyprompt/overrides/ directory
            std::fs::create_dir_all(path.join(".honeyprompt").join("overrides"))?;

            // Write default config
            config::write_default_config(&config_path)?;

            // Open/create SQLite DB (runs migrations)
            let db_path = path.join(".honeyprompt").join("events.db");
            let _conn = store::open_or_create_db(&db_path)?;

            println!("Initialized honeyprompt project in {}", path.display());
            println!("Edit honeyprompt.toml, then run `honeyprompt generate`.");
            Ok(())
        }
        Commands::Generate(args) => {
            let path = &args.path;

            // Load config
            let config_path = path.join("honeyprompt.toml");
            let cfg = config::load_config(&config_path)?;

            // Open SQLite DB
            let db_path = path.join(".honeyprompt").join("events.db");
            let conn = store::open_or_create_db(&db_path)?;

            // Run generation pipeline
            generator::generate(&cfg, &conn, path)?;

            println!("Generated output/ — ready to deploy.");
            Ok(())
        }
        Commands::Serve(args) => {
            let path = &args.path;
            let config_path = path.join("honeyprompt.toml");
            let cfg = config::load_config(&config_path)?;
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::serve(&cfg, path, args.json))?;
            Ok(())
        }
        Commands::Monitor(args) => {
            let path = &args.path;
            let config_path = path.join("honeyprompt.toml");
            let cfg = config::load_config(&config_path)?;
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(monitor::monitor(&cfg, path, &args))?;
            Ok(())
        }
        Commands::Report(args) => {
            let path = &args.path;
            let db_path = path.join(".honeyprompt").join("events.db");
            let conn = store::open_or_create_db(&db_path)?;
            let markdown = report::generate_report(&conn)?;
            if args.stdout {
                print!("{}", markdown);
            } else {
                let out_path = if let Some(ref p) = args.output {
                    p.clone()
                } else {
                    path.join("report.md")
                };
                std::fs::write(&out_path, &markdown)?;
                println!("Report written to {}", out_path.display());
            }
            Ok(())
        }
    }
}
