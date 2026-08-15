//! `portage` — inventory local disks and clouds, plan space-safe moves.
//!
//! PR 1 scope: `portage --help` and `portage init`. Everything else arrives
//! in later PRs per `docs/design.md`.

mod cmd;

use clap::{Parser, Subcommand};

/// Portage inventories files across local volumes (including USB) and clouds,
/// then moves them under a plan you confirm. Nothing mutates without a typed
/// plan id. No telemetry.
#[derive(Parser)]
#[command(name = "portage", version, about, long_about = None)]
struct Cli {
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
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init(args) => cmd::init::run(&args),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            std::process::ExitCode::from(2)
        }
    }
}
