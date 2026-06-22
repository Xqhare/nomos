use std::{path::PathBuf, process::exit};

use eshu::CliCommand;

use crate::write_err_and_exit;

pub struct Validate {
    global_config_file: PathBuf,
    flags: Vec<eshu::CliFlag>,
}

impl Validate {
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> Self {
        Validate {
            global_config_file: global_config_file.into(),
            flags: vec![],
        }
    }
}

impl<'c> CliCommand<'c> for Validate {
    fn name(&self) -> String {
        "validate".to_string()
    }
    fn short_about(&self) -> String {
        format!("Validate all files and general structure.")
    }
    fn long_about(&self) -> String {
        format!(
            "Validation Tool\nUseful to run after setting up Nomos to make sure the global configuration file, all project specific configuration files as well as all files pointed to by them, are valid, present and can be parsed."
        )
    }
    fn flags(&self) -> &Vec<eshu::CliFlag> {
        &self.flags
    }
    fn subcommands(&self) -> Vec<std::rc::Rc<dyn CliCommand<'c>>> {
        vec![]
    }
    fn execute(&self, _cli: &eshu::Cli<'c>) {
        match nomos::Nomos::new(&self.global_config_file) {
            Ok(_) => {
                println!("Nomos validation pass succesful!");
                exit(0)
            }
            Err(err) => write_err_and_exit(&format!("{err}")),
        }
    }
}
