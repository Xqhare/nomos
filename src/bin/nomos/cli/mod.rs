use std::{path::PathBuf, rc::Rc};

use eshu::Cli;
use nemesis::NemesisResultExt;
use nomos::NomosResult;

use crate::cli::{all::All, help::make_about, next::Next, validate::Validate, update::Update};

mod all;
mod flags;
mod help;
mod next;
mod validate;
mod update;

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
        .add_command(Rc::new(Update::new(&global_config_file)))
        .try_parse()
        .add_ctx("Error during Nomos startup: cli creation / parsing")?;
    Ok(cli)
}
