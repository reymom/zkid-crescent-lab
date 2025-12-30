mod cli;
mod out;
mod runner;
mod stats;
mod types;

use anyhow::Result;
use cli::{Cli, Cmd};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = out::RunContext::new(&cli)?;

    match &cli.cmd {
        Cmd::Run { param, message } => {
            let outputs = runner::run_once(&cli, &ctx, param, message.clone())?;
            out::write_outputs(&ctx, &outputs)?;
        }
        Cmd::Bench {
            param,
            iters,
            message,
        } => {
            let outputs = runner::bench(&cli, &ctx, param, *iters, message.clone())?;
            out::write_outputs(&ctx, &outputs)?;
        }
    }

    println!("Wrote: {}", ctx.run_dir.join("results.csv").display());
    if ctx.write_samples {
        println!("Wrote: {}", ctx.run_dir.join("samples.csv").display());
    }
    if ctx.logs_dir.exists() {
        println!("Logs:  {}", ctx.logs_dir.display());
    } else {
        println!("(No logs kept; only failures are logged unless --keep-logs is set.)");
    }

    Ok(())
}
