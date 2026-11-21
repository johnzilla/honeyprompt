use anyhow::{bail, Result};
use clap::Parser;
use honeyprompt::cli::{Cli, Commands, ExperimentCommand, SeedCommand};
use honeyprompt::{
    config, experiment, generator, monitor, report, seed, server, setup, store, test_agent, webhook,
};

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

            // --domain with an explicit path and no config there: create a
            // persistent project at that path (used by the Docker templates).
            if !use_tempdir && args.path.as_os_str() != "." {
                let config_path = args.path.join("honeyprompt.toml");
                if let (Some(domain), false) = (args.domain.as_deref(), config_path.exists()) {
                    std::fs::create_dir_all(args.path.join(".honeyprompt"))?;
                    let mut cfg = config::config_with_overrides(
                        &config::Config::default(),
                        Some(domain),
                        args.bind.as_deref(),
                        args.tiers.clone(),
                    );
                    if let Some(tp) = &args.trusted_proxies {
                        cfg.trusted_proxies = tp.clone();
                    }
                    std::fs::write(&config_path, toml::to_string_pretty(&cfg)?)?;
                    println!("Created {}", config_path.display());
                }
            }

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
                let mut cfg = config::config_with_overrides(
                    &base,
                    Some(domain),
                    args.bind.as_deref(),
                    args.tiers.clone(),
                );
                if let Some(tp) = &args.trusted_proxies {
                    cfg.trusted_proxies = tp.clone();
                }

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
                let mut cfg = config::config_with_overrides(
                    &base_cfg,
                    args.domain.as_deref(),
                    args.bind.as_deref(),
                    args.tiers.clone(),
                );
                if let Some(tp) = &args.trusted_proxies {
                    cfg.trusted_proxies = tp.clone();
                }

                // Generate on first run (no nonce map or page yet) or when
                // --domain changed the callback_base_url.
                let missing_site = !generator::callback_map_path(path).exists()
                    || !path.join("output").join("index.html").exists();
                let domain_changed =
                    args.domain.is_some() && cfg.callback_base_url != base_cfg.callback_base_url;
                if missing_site || domain_changed {
                    let db_path = path.join(".honeyprompt").join("events.db");
                    let conn = store::open_or_create_db(&db_path)?;
                    generator::generate(&cfg, &conn, path)?;
                    drop(conn);
                }

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
        Commands::Seed(args) => match args.command {
            SeedCommand::Add(a) => {
                let cfg = config::load_config(&a.path.join("honeyprompt.toml"))?;
                let conn =
                    store::open_or_create_db(&a.path.join(".honeyprompt").join("events.db"))?;
                let added = seed::add(&conn, &cfg, &a.label, a.note.as_deref(), a.format)?;
                eprintln!("Created seed {:?} -> {}", a.label, added.url);
                if seed::is_local_base(&cfg.callback_base_url) {
                    eprintln!(
                        "warning: callback_base_url is {}; nothing outside this machine can reach this seed",
                        cfg.callback_base_url
                    );
                }
                println!("{}", added.snippet);
                Ok(())
            }
            SeedCommand::List(a) => {
                let cfg = config::load_config(&a.path.join("honeyprompt.toml"))?;
                let conn =
                    store::open_or_create_db(&a.path.join(".honeyprompt").join("events.db"))?;
                print!(
                    "{}",
                    seed::render_list(&store::list_seeds(&conn)?, &cfg.callback_base_url)
                );
                Ok(())
            }
        },
        Commands::WebhookTest(args) => {
            let cfg = config::load_config(&args.path.join("honeyprompt.toml"))?;
            let Some(webhook) = webhook::Webhook::from_config_and_env(&cfg)? else {
                bail!(
                    "no webhook configured: set [webhook] url in honeyprompt.toml or {}",
                    webhook::ENV_URL
                );
            };
            println!("Sending a test alert to {}", webhook.describe());
            let rt = tokio::runtime::Runtime::new()?;
            match rt.block_on(webhook::send_test(&webhook)) {
                Ok(status) => {
                    println!("{} answered {}", webhook.host(), status);
                    Ok(())
                }
                Err(e) => bail!("webhook test failed: {}", e),
            }
        }
        Commands::TestAgent(args) => match test_agent::run_trials(&args) {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2); // D-05: exit 2 on error
            }
        },
        Commands::Experiment(args) => match args.command {
            ExperimentCommand::Summarize(a) => {
                let mut records = Vec::new();
                for f in &a.files {
                    records.extend(experiment::read_results(f)?);
                }
                print!("{}", experiment::render_summary(&records));
                Ok(())
            }
        },
    }
}
