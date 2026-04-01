use anyhow::{bail, Result};
use clap::Parser;
use honeyprompt::cli::{Cli, Commands};
use honeyprompt::{config, generator, monitor, report, server, setup, store, test_agent};

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
            // Determine if we need tempdir mode:
            // --domain set AND path is default "." AND no honeyprompt.toml exists at "."
            let use_tempdir = args.domain.is_some()
                && args.path.as_os_str() == "."
                && !std::path::Path::new("./honeyprompt.toml").exists();

            if use_tempdir {
                // Tempdir mode: generate ephemeral project (reuses test-agent pattern)
                let domain = args.domain.as_deref().unwrap();
                let tmp = tempfile::TempDir::new()?;
                let tmp_path = tmp.path().to_path_buf();

                // Create project structure
                std::fs::create_dir_all(tmp_path.join(".honeyprompt"))?;
                std::fs::create_dir_all(tmp_path.join("output"))?;

                // Build config from defaults + overrides
                let base = config::Config::default();
                let cfg = config::config_with_overrides(
                    &base,
                    Some(domain),
                    args.bind.as_deref(),
                    args.tiers.clone(),
                );

                // Write config to tempdir
                let config_path = tmp_path.join("honeyprompt.toml");
                let toml_string = toml::to_string_pretty(&cfg)?;
                std::fs::write(&config_path, toml_string)?;

                // Open DB and generate
                let db_path = tmp_path.join(".honeyprompt").join("events.db");
                let conn = store::open_or_create_db(&db_path)?;
                generator::generate(&cfg, &conn, &tmp_path)?;
                drop(conn);

                println!("honeyprompt serve --domain {}", domain);
                println!("  callback_base_url: {}", cfg.callback_base_url);
                println!("  bind: {}", cfg.bind_address);
                println!("  tiers: {:?}", cfg.tiers);
                println!("  mode: tempdir (ephemeral)");

                // Serve (tmp kept alive by _keep binding until serve exits)
                let rt = tokio::runtime::Runtime::new()?;
                let _keep = tmp; // prevent TempDir drop until serve exits
                rt.block_on(server::serve(&cfg, &tmp_path, args.json))?;
            } else {
                // Standard mode: load config from project dir, apply overrides
                let path = &args.path;
                let config_path = path.join("honeyprompt.toml");
                let base_cfg = config::load_config(&config_path)?;
                let cfg = config::config_with_overrides(
                    &base_cfg,
                    args.domain.as_deref(),
                    args.bind.as_deref(),
                    args.tiers.clone(),
                );

                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(server::serve(&cfg, path, args.json))?;
            }
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
        Commands::Setup(args) => {
            let path = &args.path;
            let config_path = path.join("honeyprompt.toml");
            if config_path.exists() {
                eprintln!(
                    "honeyprompt.toml already exists at {}",
                    config_path.display()
                );
                eprintln!("Delete it first if you want to re-run setup.");
                std::process::exit(1);
            }
            setup::run_setup(path)?;
            Ok(())
        }
        Commands::TestAgent(args) => {
            let format = args.format.clone();
            match test_agent::run(&args) {
                Ok(scorecard) => {
                    let output = match format {
                        honeyprompt::cli::OutputFormat::Text => scorecard.render_text(),
                        honeyprompt::cli::OutputFormat::Json => scorecard.render_json(),
                    };
                    println!("{}", output);
                    std::process::exit(scorecard.exit_code());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(2); // D-05: exit 2 on error
                }
            }
        }
    }
}
