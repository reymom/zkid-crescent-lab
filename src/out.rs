use crate::cli::Cli;
use crate::param_meta;
use crate::types::{RunOutputs, SampleRow, SummaryRow};
use anyhow::{Context, Result};
use chrono::Utc;
use csv::Writer;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct RunContext {
    pub run_id: String,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub write_samples: bool,
}

#[derive(Debug, Serialize)]
struct Meta {
    run_id: String,
    utc_started_at: String,
    crescent_git_head: Option<String>,
    rustc_version: Option<String>,
}

impl RunContext {
    pub fn new(cli: &Cli) -> Result<Self> {
        let run_id = format!("run_{}", Utc::now().format("%Y%m%dT%H%M%SZ"));
        let run_dir = cli.out_dir.join(&run_id);
        let logs_dir = run_dir.join("logs");
        fs::create_dir_all(&run_dir).context("create run dir")?;

        // Note: logs_dir created lazily on first log write (so it can be absent on success).
        write_meta(cli, &run_id, &run_dir)?;

        Ok(Self {
            run_id,
            run_dir,
            logs_dir,
            write_samples: cli.write_samples,
        })
    }
}

pub fn write_outputs(ctx: &RunContext, out: &RunOutputs) -> Result<()> {
    write_summary_csv(&ctx.run_dir.join("results.csv"), &out.summary)?;
    if ctx.write_samples {
        write_samples_csv(&ctx.run_dir.join("samples.csv"), &out.samples)?;
    }

    write_param_tables(&ctx.run_dir, &out.summary)?;
    Ok(())
}

fn write_summary_csv(path: &Path, rows: &[SummaryRow]) -> Result<()> {
    let mut wtr = Writer::from_path(path).with_context(|| format!("create {}", path.display()))?;
    for r in rows {
        wtr.serialize(r).context("serialize summary row")?;
    }
    wtr.flush().context("flush results.csv")?;
    Ok(())
}

fn write_samples_csv(path: &Path, rows: &[SampleRow]) -> Result<()> {
    let mut wtr = Writer::from_path(path).with_context(|| format!("create {}", path.display()))?;
    for r in rows {
        wtr.serialize(r).context("serialize sample row")?;
    }
    wtr.flush().context("flush samples.csv")?;
    Ok(())
}

fn write_meta(cli: &Cli, run_id: &str, run_dir: &Path) -> Result<()> {
    let meta = Meta {
        run_id: run_id.to_string(),
        utc_started_at: Utc::now().to_rfc3339(),
        crescent_git_head: git_head(&cli.crescent_root).ok(),
        rustc_version: rustc_version().ok(),
    };

    let meta_path = run_dir.join("meta.json");
    fs::write(&meta_path, serde_json::to_vec_pretty(&meta)?)
        .with_context(|| format!("write {}", meta_path.display()))?;
    Ok(())
}

fn rustc_version() -> Result<String> {
    let out = Command::new("rustc")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run rustc --version")?;

    anyhow::ensure!(
        out.status.success(),
        "rustc --version failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_head(repo_root: &Path) -> Result<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("git rev-parse HEAD")?;

    anyhow::ensure!(
        out.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn write_param_tables(run_dir: &Path, summary: &[SummaryRow]) -> Result<()> {
    // Determine param from summary (all rows share the same param)
    let param = summary
        .first()
        .map(|r| r.param.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let meta = param_meta::lookup(&param);

    // Pull proof size from the SHOW row
    let show = summary.iter().find(|r| r.step == "show");
    let proof_p50 = show.and_then(|r| r.artifact_p50_bytes);
    let proof_max = show.and_then(|r| r.artifact_max_bytes);

    // CSV
    let csv_path = run_dir.join("param_table.csv");
    let mut csv = String::new();
    csv.push_str("param,credential_type,signature_algo,claim_count,public_inputs,public_outputs,proof_p50_bytes,proof_max_bytes\n");

    let (cred, sig, cc, pi, po) = if let Some(m) = meta {
        (
            m.credential_type,
            m.signature_algo,
            opt_u32(m.claim_count),
            opt_u32(m.public_inputs),
            opt_u32(m.public_outputs),
        )
    } else {
        ("", "", "".to_string(), "".to_string(), "".to_string())
    };

    csv.push_str(&format!(
        "{},{},{},{},{},{},{},{}\n",
        param,
        cred,
        sig,
        cc,
        pi,
        po,
        opt_u64(proof_p50),
        opt_u64(proof_max),
    ));

    fs::write(&csv_path, csv).with_context(|| format!("write {}", csv_path.display()))?;

    // Markdown table for blog
    let md_path = run_dir.join("param_table.md");
    let md = format!(
        "| param | credential | sig algo | claims | public inputs | public outputs | proof p50 (bytes) | proof max (bytes) |\n\
         |---|---|---:|---:|---:|---:|---:|---:|\n\
         | {} | {} | {} | {} | {} | {} | {} | {} |\n",
        param,
        cred,
        sig,
        cc,
        pi,
        po,
        opt_u64(proof_p50),
        opt_u64(proof_max),
    );
    fs::write(&md_path, md).with_context(|| format!("write {}", md_path.display()))?;
    Ok(())
}

fn opt_u32(v: Option<u32>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}
fn opt_u64(v: Option<u64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}
