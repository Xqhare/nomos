use std::{path::PathBuf, rc::Rc};

use eshu::Cli;
use nemesis::NemesisResultExt;
use nomos::NomosResult;

use crate::cli::{all::All, next::Next, validate::Validate};

mod all;
mod flags;
mod next;
mod validate;

// TODO: The different `execute` functions share *a lot* of common code.
// This should be abstracted out.

pub fn cli<'c, P: Into<PathBuf>>(global_config_file: P) -> NomosResult<Cli<'c>> {
    let global_config_file = global_config_file.into();
    let cli = Cli::new("Nomos")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_about(&make_about())
        .add_command(Rc::new(Validate::new(&global_config_file)))
        .add_command(Rc::new(All::new(&global_config_file)))
        .add_command(Rc::new(Next::new(&global_config_file)))
        .try_parse()
        .add_ctx("Error during Nomos startup: cli creation / parsing")?;
    Ok(cli)
}

#[expect(
    clippy::too_many_lines,
    reason = "writing about page is a lot of static text"
)]
fn make_about() -> String {
    const LINEBREAK: &str = "\n";
    const DOUBLE_LINEBREAK: &str = "\n\n";
    let mut string = format!(
        "Nomos is a decentralized, text-based project management system built in Rust. It utilizes a human-readable markdown-compatible task protocol of the same name."
    );
    string.push_str(DOUBLE_LINEBREAK);

    string.push_str(
        "Important: This software expects files following the syntax defined for Nomos files.",
    );
    string.push_str(LINEBREAK);
    string.push_str("Find out more here: ");
    string.push_str(env!("CARGO_PKG_REPOSITORY"));
    string.push_str(DOUBLE_LINEBREAK);

    string.push_str("Usage:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("On fist execution Nomos creates a `config.json` file inside `~/.config/nomos`, or as a fallback inside `~/.nomos`.");
    string.push_str(LINEBREAK);
    string.push_str("Update the `search_bases` key in the `config.json` file with paths pointing to the root directories containing the projects you want to track.");
    string.push_str(LINEBREAK);
    string.push_str("Example: If the path `~/projects/rust` is present, Nomos will crawl each subdirectory of it.");
    string.push_str(LINEBREAK);
    string.push_str("Nomos will look for a `nomos.json` inside the directory. If found it will read each file it finds held by the key `task_files`.");
    string.push_str(LINEBREAK);
    string.push_str("Should Nomos be unable to find `nomos.json`, and the directory contains either a `.git` directory or a `README.md` file, Nomos falls back to looking for [nomos, todo, tasks, roadmap] files with either a `.txt` or `.md` extension inside that directory.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str(
        "Inside the global `config.json` file, you can also define specific files to be tracked.",
    );
    string.push_str(LINEBREAK);
    string.push_str("Please note that each entry requires a project name and a single, valid, path pointing to the file.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Using Nomos inside a project:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("As described above, you can either provide a project specific configuration file with the name `nomos.json` in the project root, or rely on the fallback behaviour of the crawler.");
    string.push_str(LINEBREAK);
    string.push_str("Using the project configuarion file is heavily recommended. Example files are provided in the Nomos repository (found at: ");
    string.push_str(env!("CARGO_PKG_REPOSITORY"));
    string.push_str(" ).");
    string.push_str(LINEBREAK);
    string.push_str("Project configuration files must contain the key `task_files` that holds an array of strings.");
    string.push_str(LINEBREAK);
    string.push_str("Example:");
    string.push_str(LINEBREAK);
    // Don't change the line below, the whitespace is important. Sure there are better ways,
    // but KISS and such
    // Will probably be fucked up by the eshu formatter anyways
    string.push_str("{\n  \"task_files\": [\n    \"TODO.md\",\n    \"docs/roadmap.md\"\n  ]\n}");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos, the CLI program you are using, also supports a range of subcommands.");
    string
}
