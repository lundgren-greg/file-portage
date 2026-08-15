//! `portage doctor` — catalog health checks (PR 3 stub).
//!
//! Opens (creating and migrating if needed) the catalog and runs
//! `PRAGMA integrity_check` plus `PRAGMA foreign_key_check`. Overlay-root
//! and sync-client checks arrive with PR 4; `--backup` arrives later.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;
use portage_catalog::{Catalog, LockMode};

use super::init;

/// Arguments for `portage doctor`.
#[derive(Args)]
pub struct DoctorArgs {
    /// Data directory (defaults to the platform data dir created by init).
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}

/// Run `portage doctor`.
pub fn run(args: &DoctorArgs, verbose: bool) -> Result<()> {
    let data_dir = match &args.data_dir {
        Some(dir) => dir.clone(),
        None => init::default_data_dir()?,
    };
    if !portage_core::config::lock_path(&data_dir).exists() {
        bail!(
            "{} is not a Portage data dir (no portage.lock); run: portage init",
            data_dir.display()
        );
    }

    let _guard = portage_core::obs::init_tracing(&data_dir, verbose)
        .with_context(|| format!("initializing logging in {}", data_dir.display()))?;

    let catalog = Catalog::open(&data_dir, LockMode::Exclusive)
        .with_context(|| format!("opening catalog in {}", data_dir.display()))?;

    println!(
        "catalog      : {}",
        portage_core::config::catalog_path(&data_dir).display()
    );
    println!("schema       : v{}", catalog.user_version()?);

    let problems = catalog.integrity_check()?;
    let fk_violations = catalog.foreign_key_check()?;
    tracing::info!(
        problems = problems.len(),
        fk_violations,
        data_dir = %data_dir.display(),
        "doctor"
    );

    if problems.is_empty() {
        println!("integrity    : ok");
    } else {
        for line in &problems {
            println!("integrity    : {line}");
        }
    }
    if fk_violations == 0 {
        println!("foreign keys : ok");
    } else {
        println!("foreign keys : {fk_violations} violation(s)");
    }
    println!("overlay roots: not checked yet (detectors arrive in PR 4)");

    if !problems.is_empty() || fk_violations > 0 {
        bail!("catalog failed health checks");
    }
    Ok(())
}
