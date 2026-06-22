//! Local automation entrypoint.

mod check;
mod review;
mod rust_file_length_lint;

use std::env;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("check") => check::run_check(),
        Some("review") => review::run_review(),
        Some("rust-file-length-lint") => {
            rust_file_length_lint::run_rust_file_length_lint(&args[1..])
        }
        Some(command) => Err(format!("unknown xtask command `{command}`")),
        None => Err("usage: cargo xtask <check|review|rust-file-length-lint>".to_owned()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
