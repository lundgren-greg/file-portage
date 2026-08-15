//! `portage` — inventory local disks and clouds, plan space-safe moves.
//!
//! PR 1 scope: `portage --help` and `portage init`. PR 1.5 adds structured
//! logging, metrics, and `portage status`. PR 3 adds the `doctor` stub
//! (catalog integrity checks). Everything else arrives in later PRs per
//! `docs/design.md`.

mod cmd;

use clap::{Parser, Subcommand};

/// Portage inventories files across local volumes (including USB) and clouds,
/// then moves them under a plan you confirm. Nothing mutates without a typed
/// plan id. No telemetry.
#[derive(Parser)]
#[command(name = "portage", version, about, long_about = None)]
struct Cli {
    /// Verbose stderr logging (debug level). RUST_LOG overrides.
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the Portage data directory, config, and lock file.
    ///
    /// Measures free space on the system drive. If it is below 8 GiB, prints
    /// a recommendation for the largest volume instead — it never relocates
    /// silently. Pass --data-dir to choose the location explicitly.
    Init(cmd::init::InitArgs),

    /// Show run metrics and data-dir health. --format=prom emits Prometheus
    /// text for a local textfile collector (no listener, no push).
    Status(cmd::status::StatusArgs),

    /// Check catalog health: schema version, SQLite integrity, foreign keys.
    /// Overlay-root checks arrive in PR 4.
    Doctor(cmd::doctor::DoctorArgs),
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init(args) => cmd::init::run(&args, cli.verbose),
        Command::Status(args) => cmd::status::run(&args, cli.verbose),
        Command::Doctor(args) => cmd::doctor::run(&args, cli.verbose),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            std::process::ExitCode::from(2)
        }
    }
}
