//! Explicit operator-only maintenance; no automatic server startup integration.
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Admit reviewed local Git evidence, rebuild, or audit the derived projection.
    Import {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long, conflicts_with = "verify_only")]
        rebuild: bool,
        #[arg(long, conflicts_with = "rebuild")]
        verify_only: bool,
    },
}

pub fn run(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    let Command::Import {
        profile,
        rebuild,
        verify_only,
    } = command;
    let started = std::time::Instant::now();
    let prepared = super::import::prepare(&profile, verify_only)?;
    let mut report = super::import::finish(prepared, rebuild, verify_only)?;
    report
        .milliseconds
        .insert("maintenance_before_output", started.elapsed().as_millis());
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
