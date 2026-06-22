//! CLI binary for Nomos
//!
//! TODO: Add TUI subcommand

use crate::{cli::cli, startup::startup};

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
    let _cli = match cli(startup.global_config_file) {
        Ok(cli) => cli,
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
