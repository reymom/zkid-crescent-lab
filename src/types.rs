use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    Zksetup,
    Prove,
    Show,
    Verify,
}

impl Step {
    pub fn as_str(&self) -> &'static str {
        match self {
            Step::Zksetup => "zksetup",
            Step::Prove => "prove",
            Step::Show => "show",
            Step::Verify => "verify",
        }
    }

    pub fn all_in_order() -> [Step; 4] {
        [Step::Zksetup, Step::Prove, Step::Show, Step::Verify]
    }

    pub fn supports_message(&self) -> bool {
        matches!(self, Step::Show | Step::Verify)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SampleRow {
    pub run_id: String,
    pub param: String,
    pub step: String,
    pub iter: usize,
    pub ok: bool,
    pub duration_ms: u128,
    pub exit_code: i32,
    pub kept_stdout: Option<String>,
    pub kept_stderr: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRow {
    pub run_id: String,
    pub param: String,
    pub step: String,
    pub iters: usize,
    pub ok: usize,
    pub fail: usize,
    pub min_ms: u128,
    pub p50_ms: u128,
    pub p95_ms: u128,
    pub mean_ms: u128,
    pub max_ms: u128,
}

#[derive(Debug, Clone)]
pub struct StepRun {
    pub step: Step,
    pub iter: usize,
    pub ok: bool,
    pub duration_ms: u128,
    pub exit_code: i32,
    pub kept_stdout: Option<PathBuf>,
    pub kept_stderr: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RunOutputs {
    pub run_id: String,
    pub run_dir: PathBuf,
    pub samples: Vec<SampleRow>,
    pub summary: Vec<SummaryRow>,
}
