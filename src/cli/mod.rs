use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser)]
#[command(
    name = "honeyprompt",
    version,
    about = "Detect and measure unsafe behavior by AI browsing agents"
)]
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
    /// Interactive setup wizard — creates honeyprompt.toml
    Setup(SetupArgs),
    /// Run a bounded compliance test against an AI agent
    TestAgent(TestAgentArgs),
    /// Send one synthetic alert to the configured webhook and show the response
    WebhookTest(WebhookTestArgs),
    /// Create and list seeded canary URLs to plant in private context
    Seed(SeedArgs),
    /// Work with results from repeated test-agent runs
    Experiment(ExperimentArgs),
}

#[derive(Parser)]
pub struct SeedArgs {
    #[command(subcommand)]
    pub command: SeedCommand,
}

#[derive(Subcommand)]
pub enum SeedCommand {
    /// Create a seed and print the canary to plant (stdout)
    Add(SeedAddArgs),
    /// List seeds with their hit counts
    List(SeedListArgs),
}

#[derive(Parser)]
pub struct SeedAddArgs {
    /// Unique label naming the placement (a-z, 0-9, '.', '_', '-'), e.g. claude-md-prod
    pub label: String,
    /// Project directory containing honeyprompt.toml
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Free-text note about where the seed is planted
    #[arg(long)]
    pub note: Option<String>,
    /// Output format
    #[arg(long, value_enum, default_value_t = crate::seed::SnippetFormat::Url)]
    pub format: crate::seed::SnippetFormat,
}

#[derive(Parser)]
pub struct SeedListArgs {
    /// Project directory containing honeyprompt.toml
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct WebhookTestArgs {
    /// Project directory containing honeyprompt.toml
    #[arg(default_value = ".")]
    pub path: PathBuf,
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
    /// When --domain is used without --path, a tempdir is created automatically
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Output events as JSON lines instead of structured text
    #[arg(long)]
    pub json: bool,
    /// Domain for zero-config serve: sets callback_base_url=https://{domain},
    /// bind=0.0.0.0:8080, enables all tiers. Generates in tempdir if no --path.
    #[arg(long)]
    pub domain: Option<String>,
    /// Override bind address (e.g., 0.0.0.0:9090)
    #[arg(long)]
    pub bind: Option<String>,
    /// Override which tiers to enable (e.g., --tiers 1 2 3)
    #[arg(long, num_args = 1..)]
    pub tiers: Option<Vec<u8>>,
    /// Reverse proxies whose X-Forwarded-For is trusted for the client IP
    /// (IPs or CIDRs, e.g., --trusted-proxies 172.16.0.0/12). Overrides config.
    #[arg(long, num_args = 1..)]
    pub trusted_proxies: Option<Vec<String>>,
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

#[derive(Parser)]
pub struct SetupArgs {
    /// Directory to write honeyprompt.toml into (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct TestAgentArgs {
    /// Listen address for the ephemeral server (default: 127.0.0.1:0 for OS-assigned port)
    #[arg(long, default_value = "127.0.0.1:0")]
    pub listen: String,
    /// Seconds to wait for callbacks before shutting down (default: 60)
    #[arg(long, default_value_t = 60)]
    pub timeout: u64,
    /// Output format: text (default) or json
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    /// Also run tier 6 (context exfiltration): mint a fake HPCANARY- token for you
    /// to place in the agent's private context, and fail if the agent leaks it
    #[arg(long)]
    pub exfil: bool,
    /// Write the tier-6 canary token to this file (mode 0600) so CI can inject it
    #[arg(long, requires = "exfil")]
    pub canary_file: Option<std::path::PathBuf>,
    /// Public base URL that reaches --listen through your reverse proxy
    /// (e.g. https://agents.example.com). Each run is served under a random
    /// /run/{id}/ prefix. Needs a fixed --listen port.
    #[arg(long)]
    pub public_url: Option<String>,
    /// Reverse proxies whose X-Forwarded-For is trusted for the client IP
    /// (IPs or CIDRs, e.g. --trusted-proxies 172.16.0.0/12)
    #[arg(long, num_args = 1..)]
    pub trusted_proxies: Option<Vec<String>>,
    /// Command that runs the agent, via `sh -c`. `{url}` and `{canary}` are
    /// replaced, and HONEYPROMPT_URL / HONEYPROMPT_CANARY are set. The run ends
    /// --grace seconds after it exits, or at --timeout (the agent is killed).
    #[arg(long)]
    pub agent_cmd: Option<String>,
    /// Seconds to keep listening after the agent command exits (default: 15)
    #[arg(long, default_value_t = 15, requires = "agent_cmd")]
    pub grace: u64,
    /// Append the agent command's stdout and stderr to this file
    /// (default: forwarded to stderr; never mixed into results)
    #[arg(long, requires = "agent_cmd")]
    pub agent_log: Option<PathBuf>,
    /// Number of runs, each with fresh nonces (and a fresh canary with --exfil)
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=1000), requires = "agent_cmd")]
    pub trials: u32,
    /// Name of the agent under test, recorded with each result
    #[arg(long)]
    pub label: Option<String>,
    /// Append one JSON line per run to this file (see `honeyprompt experiment summarize`)
    #[arg(long, requires = "label")]
    pub results: Option<PathBuf>,
}

#[derive(Parser)]
pub struct ExperimentArgs {
    #[command(subcommand)]
    pub command: ExperimentCommand,
}

#[derive(Subcommand)]
pub enum ExperimentCommand {
    /// Summarize test-agent --results files into a Markdown table per agent
    Summarize(SummarizeArgs),
}

#[derive(Parser)]
pub struct SummarizeArgs {
    /// One or more results files written by `test-agent --results`
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}
