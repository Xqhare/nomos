use std::{path::PathBuf, process::exit};

use eshu::CliCommand;

use crate::write_err_and_exit;

pub struct All {
    global_config_file: PathBuf,
    flags: Vec<CliFlag>,
}

impl All {
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> Self {
        All {
            global_config_file: global_config_file.into(),
        }
    }
}

impl CliCommand for All {
    fn name(&self) -> String {}
    fn short_about(&self) -> String {}
    fn long_about(&self) -> String {}
    fn flags(&self) -> &Vec<eshu::CliFlag> {
        &vec![]
    }
    fn subcommands(&self) -> Vec<std::rc::Rc<dyn CliCommand<'c>>> {
        vec![]
    }
    fn execute(&self, _cli: &eshu::Cli<'c>) {}
}
