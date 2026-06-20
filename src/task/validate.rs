use nemesis::NemesisError;

use crate::{
    NomosError,
    error::{NomosResult, Parser},
    task::Task,
    utils::TaskStatus,
};

/// Validate the task
/// - May only be status::done if all sub tasks are done.
/// - Title are not empty.
/// - Completion date may only be set if status is done and inception date is set.
pub fn validate_task(task: &Task) -> NomosResult<()> {
    validate_sub_tasks(task)?;
    if !is_marked_done(task) {
        if task.completion_date.is_some() {
            return Err(NemesisError::new(
                "nomos::parser::task::validate_task",
                NomosError::Parser(Parser::Task(
                    "Task is not marked as done but completion date is set".to_string(),
                )),
            )
            .add_ctx(format!("Task: {task:?}")));
        }
    }
    if task.completion_date.is_some() && task.inception_date.is_none() {
        return Err(NemesisError::new(
            "nomos::parser::task::validate_task",
            NomosError::Parser(Parser::Task(
                "Task is marked as done but inception date is not set".to_string(),
            )),
        )
        .add_ctx(format!("Task: {task:?}")));
    }
    if task.title.is_empty() {
        return Err(NemesisError::new(
            "nomos::parser::task::validate_task",
            NomosError::Parser(Parser::Task("Task title is empty".to_string())),
        )
        .add_ctx(format!("Task: {task:?}")));
    }
    Ok(())
}

fn validate_sub_tasks(task: &Task) -> NomosResult<()> {
    if let Some(tasks) = &task.sub_tasks {
        for sub_task in tasks.iter() {
            validate_task(sub_task)?;
        }
    }
    Ok(())
}

fn is_marked_done(task: &Task) -> bool {
    if task.status == TaskStatus::Done {
        if let Some(tasks) = &task.sub_tasks {
            for sub_task in tasks.iter() {
                if !is_marked_done(sub_task) {
                    return false;
                }
            }
        }
        return true;
    }
    false
}
