use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use crate::cli::Cli;
use crate::out::RunContext;
use crate::stats;
use crate::types::{RunOutputs, SampleRow, Step, StepRun, SummaryRow};
use anyhow::{Context, Result, anyhow};
use walkdir::WalkDir;

pub fn run_once(
    cli: &Cli,
    ctx: &RunContext,
    param: &str,
    message: Option<String>,
) -> Result<RunOutputs> {
    // zksetup/prove/show/verify once each
    let mut runs = Vec::new();

    let pass_msg = decide_message_policy(cli, param, message)?;

    for step in Step::all_in_order() {
        let r = run_step(cli, ctx, param, step, 0, pass_msg.clone())?;
        print_run_line(&r);
        runs.push(r);
    }

    build_outputs(ctx, param, runs)
}

pub fn bench(
    cli: &Cli,
    ctx: &RunContext,
    param: &str,
    iters: usize,
    message: Option<String>,
) -> Result<RunOutputs> {
    let mut runs = Vec::new();

    // Only run zksetup+prove once.
    let pass_msg = decide_message_policy(cli, param, message)?;

    for step in [Step::Zksetup, Step::Prove] {
        let r = run_step(cli, ctx, param, step, 0, None)?;
        print_run_line(&r);
        runs.push(r);
    }

    for i in 0..iters {
        let show = run_step(cli, ctx, param, Step::Show, i, pass_msg.clone())?;
        print_run_line(&show);
        runs.push(show);

        let verify = run_step(cli, ctx, param, Step::Verify, i, pass_msg.clone())?;
        print_run_line(&verify);
        runs.push(verify);
    }

    build_outputs(ctx, param, runs)
}

fn print_run_line(r: &StepRun) {
    if r.ok {
        println!(
            "[ok]   {} iter={} duration_ms={}",
            r.step.as_str(),
            r.iter,
            r.duration_ms
        );
    } else {
        println!(
            "[fail] {} iter={} duration_ms={} exit_code={}",
            r.step.as_str(),
            r.iter,
            r.duration_ms,
            r.exit_code
        );
    }
}

fn build_outputs(ctx: &RunContext, param: &str, runs: Vec<StepRun>) -> Result<RunOutputs> {
    // Raw samples
    let mut samples = Vec::with_capacity(runs.len());
    for r in &runs {
        samples.push(SampleRow {
            run_id: ctx.run_id.clone(),
            param: param.to_string(),
            step: r.step.as_str().to_string(),
            iter: r.iter,
            ok: r.ok,
            duration_ms: r.duration_ms,
            exit_code: r.exit_code,
            kept_stdout: r
                .kept_stdout
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            kept_stderr: r
                .kept_stderr
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            artifact_path: r
                .artifact_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            artifact_bytes: r.artifact_bytes,
        });
    }

    // Aggregate per-step
    let mut by_step: HashMap<Step, Vec<&StepRun>> = HashMap::new();
    for r in &runs {
        by_step.entry(r.step).or_default().push(r);
    }

    let mut summary = Vec::new();
    for step in Step::all_in_order() {
        let Some(items) = by_step.get(&step) else {
            continue;
        };
        let durations: Vec<u128> = items.iter().map(|x| x.duration_ms).collect();
        let ok = items.iter().filter(|x| x.ok).count();
        let fail = items.len() - ok;
        let (min, p50, p95, mean, max) = stats::summarize_u128(&durations);
        let artifact_sizes: Vec<u64> = items.iter().filter_map(|x| x.artifact_bytes).collect();
        let artifact_iters = artifact_sizes.len();
        let (artifact_p50, artifact_mean, artifact_max) = if artifact_iters > 0 {
            let (p50b, meanb, maxb) = stats::summarize_u64(&artifact_sizes);
            (Some(p50b), Some(meanb), Some(maxb))
        } else {
            (None, None, None)
        };

        summary.push(SummaryRow {
            run_id: ctx.run_id.clone(),
            param: param.to_string(),
            step: step.as_str().to_string(),
            iters: items.len(),
            ok,
            fail,
            min_ms: min,
            p50_ms: p50,
            p95_ms: p95,
            mean_ms: mean,
            max_ms: max,
            artifact_iters,
            artifact_p50_bytes: artifact_p50,
            artifact_mean_bytes: artifact_mean,
            artifact_max_bytes: artifact_max,
        });
    }

    Ok(RunOutputs { samples, summary })
}

fn run_step(
    cli: &Cli,
    ctx: &RunContext,
    param: &str,
    step: Step,
    iter: usize,
    msg: Option<String>,
) -> Result<StepRun> {
    let creds_dir = cli.crescent_root.join("creds");
    if !creds_dir.is_dir() {
        return Err(anyhow!(
            "Crescent creds dir not found at {}",
            creds_dir.display()
        ));
    }

    let (stdout_path, stderr_path) = (
        ctx.logs_dir.join(format!(
            "{}_{}_{:04}_stdout.log",
            param,
            step.as_str(),
            iter
        )),
        ctx.logs_dir.join(format!(
            "{}_{}_{:04}_stderr.log",
            param,
            step.as_str(),
            iter
        )),
    );

    let started_fs = SystemTime::now();
    let started = Instant::now();
    let out = invoke_crescent(&creds_dir, param, step, msg)?;
    let duration_ms = started.elapsed().as_millis();

    let exit_code = out.status.code().unwrap_or(-1);
    let ok = out.status.success();

    let mut kept_stdout = None;
    let mut kept_stderr = None;

    if cli.keep_logs || !ok {
        fs::create_dir_all(&ctx.logs_dir).context("create logs dir")?;
        fs::write(&stdout_path, &out.stdout)
            .with_context(|| format!("write {}", stdout_path.display()))?;
        fs::write(&stderr_path, &out.stderr)
            .with_context(|| format!("write {}", stderr_path.display()))?;
        kept_stdout = Some(stdout_path);
        kept_stderr = Some(stderr_path);
    }

    // New: after show, try to locate the proof/presentation artifact and record size.
    let (artifact_path, artifact_bytes) = if ok && step == Step::Show {
        let tv_dir = creds_dir.join("test-vectors").join(param);
        find_new_artifact(&tv_dir, started_fs).unwrap_or((None, None))
    } else {
        (None, None)
    };

    Ok(StepRun {
        step,
        iter,
        ok,
        duration_ms,
        exit_code,
        kept_stdout,
        kept_stderr,
        artifact_path,
        artifact_bytes,
    })
}

fn find_new_artifact(tv_dir: &Path, since: SystemTime) -> Result<(Option<PathBuf>, Option<u64>)> {
    if !tv_dir.is_dir() {
        return Ok((None, None));
    }

    // Allow 1s slack for filesystem timestamp granularity.
    let threshold = since
        .checked_sub(Duration::from_secs(1))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut best: Option<(PathBuf, SystemTime, bool)> = None;
    for entry in WalkDir::new(tv_dir).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path().to_path_buf();

        let meta = fs::metadata(&p)?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if mtime < threshold {
            continue;
        }

        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let preferred = name.contains("presentation") || name.contains("proof");

        let should_replace = match &best {
            None => true,
            Some((_bp, bt, bpref)) => {
                (preferred && !*bpref) || (preferred == *bpref && mtime > *bt)
            }
        };

        if should_replace {
            best = Some((p, mtime, preferred));
        }
    }

    let Some((path, _mtime, _pref)) = best else {
        return Ok((None, None));
    };

    let bytes = fs::metadata(&path)?.len();
    Ok((Some(path), Some(bytes)))
}

fn invoke_crescent(
    creds_dir: &Path,
    param: &str,
    step: Step,
    msg: Option<String>,
) -> Result<std::process::Output> {
    // Prefer calling the already-built binary (faster), fallback to cargo.
    let exe = if cfg!(windows) {
        "crescent.exe"
    } else {
        "crescent"
    };
    let direct_rel = PathBuf::from("target").join("release").join(exe);
    let direct_abs = creds_dir.join(&direct_rel);

    if direct_abs.is_file() {
        let mut cmd = Command::new(&direct_rel); // IMPORTANT: relative to creds_dir
        cmd.current_dir(creds_dir)
            .arg(step.as_str())
            .arg("--name")
            .arg(param);

        if step.supports_message() {
            if let Some(m) = msg {
                cmd.arg("--presentation-message").arg(m);
            }
        }

        return cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("run crescent (direct)");
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(creds_dir)
        .arg("run")
        .arg("--release")
        .arg("--bin")
        .arg("crescent")
        .arg(step.as_str())
        .arg("--name")
        .arg(param);

    if step.supports_message() {
        if let Some(m) = msg {
            cmd.arg("--presentation-message").arg(m);
        }
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run crescent (cargo)")
}

fn decide_message_policy(
    _cli: &Cli,
    param: &str,
    message: Option<String>,
) -> Result<Option<String>> {
    // If user didn't pass a message, do nothing.
    let Some(m) = message else { return Ok(None) };

    // Avoid Crescent panic "Multiple presentation messages" by suppressing the arg if
    // test-vectors already embed a presentation message.
    if test_vectors_have_presentation_message(param)? {
        println!(
            "[info] test-vectors for '{param}' appear to already include a presentation message; \
not passing --presentation-message to avoid Crescent panic"
        );
        return Ok(None);
    }

    Ok(Some(m))
}

fn test_vectors_have_presentation_message(param: &str) -> Result<bool> {
    // Heuristic: scan any JSON under creds/test-vectors/<param> for the literal key.
    // If the directory doesn't exist, return false.
    let tv_dir = PathBuf::from("vendor/crescent-credentials/creds/test-vectors").join(param);
    if !tv_dir.is_dir() {
        return Ok(false);
    }

    for entry in WalkDir::new(&tv_dir).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(p)?;
        if bytes
            .windows(b"presentation_message".len())
            .any(|w| w == b"presentation_message")
        {
            return Ok(true);
        }
    }
    Ok(false)
}
