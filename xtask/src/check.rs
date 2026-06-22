//! Local workspace verification commands.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::rust_file_length_lint::run_rust_file_length_lint;

pub(crate) fn run_check() -> Result<(), String> {
    actionlint()?;
    source_layout_lint()?;
    run_command("cargo", ["fmt", "--all", "--", "--check"])?;
    run_command("cargo", ["build", "--workspace"])?;
    run_command("cargo", ["build", "--workspace", "--all-features"])?;
    run_command(
        "cargo",
        [
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_command("cargo", ["test", "--workspace"])?;
    run_command("cargo", ["test", "--workspace", "--all-features"])?;
    run_command("cargo", ["test", "--doc", "--workspace"])?;
    run_rust_file_length_lint(&["--all".to_owned()])
}

fn actionlint() -> Result<(), String> {
    if !Path::new(".github/workflows").exists() {
        return Ok(());
    }
    if command_output("actionlint", ["-version"]).is_err() {
        println!("actionlint=skipped (binary not found)");
        return Ok(());
    }
    run_command("actionlint", [".github/workflows/ci.yml"])
}

fn source_layout_lint() -> Result<(), String> {
    let mut violations = Vec::new();
    collect_source_layout_violations(Path::new("crates"), &mut violations)?;
    if violations.is_empty() {
        println!("source_layout_lint=ok");
        return Ok(());
    }
    for violation in &violations {
        eprintln!("{violation}");
    }
    Err(format!(
        "{} Rust test layout violation(s)",
        violations.len()
    ))
}

fn collect_source_layout_violations(
    path: &Path,
    violations: &mut Vec<String>,
) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(path).map_err(|source| format!("read {}: {source}", path.display()))?
    {
        let entry = entry.map_err(|source| format!("read {} entry: {source}", path.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_source_layout_violations(&path, violations)?;
            continue;
        }
        if path.extension().and_then(OsStr::to_str) != Some("rs") {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|source| format!("read {}: {source}", path.display()))?;
        let is_test_file = path.components().any(|component| {
            component.as_os_str() == "_tests_" || component.as_os_str() == "tests"
        });
        if !is_test_file && (contents.contains("#[test]") || contents.contains("#[tokio::test]")) {
            violations.push(format!(
                "{} contains test bodies outside a _tests_ module",
                path.display()
            ));
        }
    }
    Ok(())
}

fn run_command<I, S>(program: &str, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|source| format!("failed to run `{program}`: {source}"))?;
    if status.success() {
        return Ok(());
    }
    Err(format!("`{program}` exited with {status}"))
}

fn command_output<I, S>(program: &str, args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|source| format!("failed to run `{program}`: {source}"))?;
    if !output.status.success() {
        return Err(format!("`{program}` exited with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|source| format!("stdout was not utf-8: {source}"))
}
