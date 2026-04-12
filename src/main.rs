mod config;
mod daemon;
mod install;
mod proxy;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(name = "eproxy")]
#[command(version = VERSION)]
#[command(about = "Local SNI-aware API forwarder", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run in foreground
    Run,
    /// Spawn in background
    Daemon,
    /// Install Linux systemd user service
    Install,
    /// Show current version
    Version,
}

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

async fn try_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("eproxy {VERSION}");
            Ok(())
        }
        command => {
            let layout = config::ensure_layout()?;
            let config_path = resolve_config_path(cli.config.as_deref(), &layout.config_path);
            match command {
                Commands::Run => run_command(&config_path, &layout.logs_dir).await,
                Commands::Daemon => daemon_command(&config_path, &layout.pid_file),
                Commands::Install => install::install_systemd_user_service(&config_path),
                Commands::Version => unreachable!("version handled above"),
            }
        }
    }
}

async fn run_command(config_path: &Path, logs_dir: &Path) -> Result<()> {
    let resolved = config::load_or_bootstrap(config_path)?;
    let _logging_guard = init_logging(logs_dir, &resolved.log_level)?;

    info!(version = VERSION, config = %config_path.display(), "starting eproxy");
    proxy::run(resolved).await
}

fn daemon_command(config_path: &Path, pid_file: &Path) -> Result<()> {
    let _ = config::load_or_bootstrap(config_path)?;
    let pid = daemon::spawn_daemon(config_path, pid_file)?;
    println!("eproxy daemon started with pid {pid}");
    println!("pid file: {}", pid_file.display());
    Ok(())
}

fn resolve_config_path(cli_path: Option<&Path>, default_path: &Path) -> PathBuf {
    cli_path
        .map(std::borrow::ToOwned::to_owned)
        .unwrap_or_else(|| default_path.to_path_buf())
}

fn init_logging(
    log_dir: &Path,
    level: &str,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = tracing_appender::rolling::daily(log_dir, "eproxy.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_new(level)
        .with_context(|| format!("invalid log level/filter '{level}' in config"))?;

    let stdout_layer = tracing_subscriber::fmt::layer();
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init()
        .context("failed to initialize logging")?;

    Ok(guard)
}
