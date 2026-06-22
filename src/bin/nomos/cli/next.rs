use std::path::PathBuf;

use eshu::CliCommand;
use nomos::{
    Nomos,
    prelude::{Task, TaskStatus},
};

use crate::{cli::flags::make_standard_flags, write_err_and_exit};

pub struct Next {
    global_config_file: PathBuf,
    flags: Vec<eshu::CliFlag>,
}

impl Next {
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> Self {
        Next {
            global_config_file: global_config_file.into(),
            flags: make_standard_flags(),
        }
    }
}

impl<'c> CliCommand<'c> for Next {
    fn name(&self) -> String {
        "next".to_string()
    }
    fn short_about(&self) -> String {
        "Show next task".to_string()
    }
    fn long_about(&self) -> String {
        "Calculates the next task and prints it to stdout".to_string()
    }
    fn flags(&self) -> &Vec<eshu::CliFlag> {
        &self.flags
    }
    fn subcommands(&self) -> Vec<std::rc::Rc<dyn CliCommand<'c>>> {
        vec![]
    }
    fn execute(&self, cli: &eshu::Cli<'c>) {
        let mut out: Vec<Task>;
        match Nomos::new(&self.global_config_file) {
            Ok(nomos) => {
                if let Some(status) = cli.get_flag_store("status") {
                    let status = status
                        .as_value()
                        .expect("status must be a value: status=value, not status=key=value");
                    if status.len() != 1 {
                        write_err_and_exit(
                            "status must be one single value: status=value. Passing in several does not work",
                        );
                        return;
                    }

                    match status[0].as_str().to_lowercase().as_str() {
                        "open" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::Open) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        "in_progress" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::InProgress) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        "done" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::Done) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        "blocked" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::Blocked) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        "deferred" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::Deferred) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        "cut" => {
                            out = {
                                match nomos.get_tasks_by_status(TaskStatus::Cut) {
                                    Ok(tasks) => tasks.iter().map(|t| t.clone()).collect(),
                                    Err(e) => {
                                        write_err_and_exit(&e.to_string());
                                        return;
                                    }
                                }
                            };
                        }
                        _ => {
                            write_err_and_exit(
                                "status must be one of: open, in_progress, done, blocked, deferred, cut",
                            );
                            return;
                        }
                    }
                } else {
                    out = nomos.get_tasks().iter().map(|t| t.clone()).collect();
                }
                debug_assert!(out.len() > 0); // From here on, only work from out
                if let Some(project) = cli.get_flag_store("project") {
                    let project = project
                        .as_value()
                        .expect("project must be a value: project=value, not project=key=value");
                    if project.len() != 1 {
                        write_err_and_exit(
                            "project must be one single value: project=value. Passing in several does not work",
                        );
                        return;
                    }
                    out = out
                        .into_iter()
                        .filter(|t| t.project == project[0].as_str())
                        .collect();
                }

                if let Some(number) = cli.get_flag_store("number") {
                    let number = number
                        .as_value()
                        .expect("number must be a value: number=value, not number=key=value");
                    if number.len() != 1 {
                        write_err_and_exit(
                            "number must be one single value: number=value. Passing in several does not work",
                        );
                        return;
                    }
                    let number = number[0].as_str().parse::<u32>().unwrap();
                    out = out.into_iter().take(number as usize).collect();
                } else {
                    // Default to showing only the first (or `next`) task
                    out = out.into_iter().take(1).collect();
                }
            }
            Err(e) => {
                write_err_and_exit(&e.to_string());
                return;
            }
        }
        if out.len() == 0 {
            println!("No tasks found");
            return;
        }
        for (i, task) in out.iter().enumerate() {
            println!("#{}", i + 1);
            println!("{}", task);
            println!("");
        }
        println!("Found {} tasks matching your query", out.len());
    }
}
