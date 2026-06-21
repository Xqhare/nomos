//! CLI binary for Nomos
//!
//! TODO: Add TUI subcommand

use crate::startup::startup;

mod cli;
mod startup;

fn main() {
    let startup = match startup() {
        Ok(startup) => startup,
        Err(err) => {
            write_err_and_exit(&err.to_string());
            return;
        }
    };
}

fn write_err_and_exit(msg: &str) {
    eprintln!("{}", msg);
    std::process::exit(1);
}
