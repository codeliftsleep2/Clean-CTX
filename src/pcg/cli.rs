//! Explicit operator-only maintenance; no automatic server startup integration.
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Describe or explicitly authorize an exact internal document governance change.
    Documents {
        #[command(subcommand)]
        command: DocumentCommand,
    },
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

#[derive(Debug, Subcommand)]
pub enum DocumentCommand {
    Describe {
        #[arg(long)]
        profile: PathBuf,
    },
    /// Owner-operated procedural submission; agents must not invoke this action.
    Authorize {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        digest: String,
    },
}

pub fn run(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    let (profile, rebuild, verify_only) = match command {
        Command::Documents { command } => {
            let report = match command {
                DocumentCommand::Describe { profile } => {
                    super::documents::operator::describe(&profile)?
                }
                DocumentCommand::Authorize { profile, digest } => {
                    super::documents::operator::authorize(&profile, &digest)?
                }
            };
            println!("{}", serde_json::to_string(&report)?);
            return Ok(());
        }
        Command::Import {
            profile,
            rebuild,
            verify_only,
        } => (profile, rebuild, verify_only),
    };
    /* Existing enrolled import remains separate from document governance. */
    let started = std::time::Instant::now();
    let prepared = super::import::prepare(&profile, verify_only)?;
    let mut report = super::import::finish(prepared, rebuild, verify_only)?;
    report
        .milliseconds
        .insert("maintenance_before_output", started.elapsed().as_millis());
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
