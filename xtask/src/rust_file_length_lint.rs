//! Rust source file length lint.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

const MAX_RUST_LINES: usize = 300;

pub(crate) fn run_rust_file_length_lint(args: &[String]) -> Result<(), String> {
    if args != ["--all"] {
        return Err("usage: cargo xtask rust-file-length-lint --all".to_owned());
    }
    let mut violations = Vec::new();
    collect_violations(Path::new("crates"), &mut violations)?;
    collect_violations(Path::new("xtask"), &mut violations)?;
    if violations.is_empty() {
        println!("rust_file_length_lint=ok");
        return Ok(());
    }
    for violation in &violations {
        eprintln!("{violation}");
    }
    Err(format!(
        "{} Rust file length violation(s)",
        violations.len()
    ))
}

fn collect_violations(path: &Path, violations: &mut Vec<String>) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(path).map_err(|source| format!("read {}: {source}", path.display()))?
    {
        let entry = entry.map_err(|source| format!("read {} entry: {source}", path.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_violations(&path, violations)?;
            continue;
        }
        if path.extension().and_then(OsStr::to_str) != Some("rs") {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|source| format!("read {}: {source}", path.display()))?;
        let line_count = contents.lines().count();
        if line_count > MAX_RUST_LINES {
            violations.push(format!(
                "{} has {line_count} lines; max is {MAX_RUST_LINES}",
                path.display()
            ));
        }
    }
    Ok(())
}
