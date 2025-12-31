use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "crescent_bench", version)]
#[command(about = "Benchmark harness for Crescent Credentials CLI")]
pub struct Cli {
    /// Path to Crescent repo root (contains ./creds and ./circuit_setup).
    #[arg(long, default_value = "vendor/crescent-credentials")]
    pub crescent_root: PathBuf,

    /// Output directory for runs.
    #[arg(long, default_value = "out")]
    pub out_dir: PathBuf,

    /// Keep stdout/stderr logs even on success.
    #[arg(long, default_value_t = false)]
    pub keep_logs: bool,

    /// Write per-iteration samples.csv (in addition to aggregated results.csv).
    #[arg(long, default_value_t = true)]
    pub write_samples: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Run zksetup+prove+show+verify once.
    Run {
        #[arg(long)]
        param: String,

        /// Optional presentation message to try to pass to show/verify.
        /// (May be suppressed automatically to avoid Crescent "Multiple presentation messages".)
        #[arg(long)]
        message: Option<String>,
    },

    /// Run zksetup+prove once, then show+verify N times.
    Bench {
        #[arg(long)]
        param: String,

        #[arg(long, default_value_t = 30)]
        iters: usize,

        /// Optional presentation message to try to pass to show/verify.
        /// (May be suppressed automatically to avoid Crescent "Multiple presentation messages".)
        #[arg(long)]
        message: Option<String>,
    },
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
