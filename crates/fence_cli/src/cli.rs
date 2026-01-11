//! CLI argument definitions.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Parsed command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "fence",
    version,
    about = "A fast file-size fence for LLM-friendly codebases"
)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Show only errors (suppress warnings).
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Suppress all output.
    #[arg(long = "silent", global = true)]
    pub silent: bool,

    /// Show verbose output including skip warnings.
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    /// Path to config file (overrides discovery).
    #[arg(long = "config", global = true)]
    pub config: Option<PathBuf>,
}

/// Available commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Check files against line limits.
    Check(CheckArgs),
    /// Initialize a new .fence.toml config.
    Init(InitArgs),
}

/// Arguments for the check command.
#[derive(Args, Debug, Clone)]
pub struct CheckArgs {
    /// Paths to check (files or directories).
    #[arg(value_name = "PATH", allow_hyphen_values = true)]
    pub paths: Vec<PathBuf>,
}

/// Arguments for the init command.
#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Generate config that exempts all current violations.
    #[arg(long = "baseline")]
    pub baseline: bool,
}
