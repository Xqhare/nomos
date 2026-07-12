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

    if task.status == TaskStatus::Done {
        if let Some(sub_tasks) = &task.sub_tasks {
            for sub_task in sub_tasks.iter() {
                if sub_task.status != TaskStatus::Done && sub_task.status != TaskStatus::Cut {
                    return Err(NemesisError::new(
                        "nomos::parser::task::validate_task",
                        NomosError::Parser(Parser::Task(
                            "Done task has unresolved subtasks".to_string(),
                        )),
                    )
                    .add_ctx(format!("Task: {task:?}")));
                }
            }
        }
    }

    if !is_resolved(task) {
        if task.completion_date.is_some() {
            return Err(NemesisError::new(
                "nomos::parser::task::validate_task",
                NomosError::Parser(Parser::Task(
                    "Task is not resolved but completion date is set".to_string(),
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

fn is_resolved(task: &Task) -> bool {
    if task.status == TaskStatus::Done || task.status == TaskStatus::Cut {
        if let Some(tasks) = &task.sub_tasks {
            for sub_task in tasks.iter() {
                if !is_resolved(sub_task) {
                    return false;
                }
            }
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::parser::parse_string;

    #[test]
    fn test_validate_done_task_with_unresolved_subtasks() {
        let content = "\
- [x] Parent Task ::
    - [ ] Open Subtask
";
        let res = parse_string(content, Path::new("test.md"), Some("proj".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_done_task_with_resolved_subtasks() {
        let content = "\
- [x] Parent Task ::
    - [x] Done Subtask
    - [C] Cut Subtask
";
        let res = parse_string(content, Path::new("test.md"), Some("proj".to_string()));
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_cut_task_with_completion_date() {
        let content = "\
- [C] Parent Task :: 2026-07-12 2026-07-12
    - [x] Done Subtask
";
        let res = parse_string(content, Path::new("test.md"), Some("proj".to_string()));
        assert!(res.is_ok());

        let content_invalid = "\
- [C] Parent Task :: 2026-07-12 2026-07-12
    - [ ] Open Subtask
";
        let res_invalid = parse_string(content_invalid, Path::new("test.md"), Some("proj".to_string()));
        assert!(res_invalid.is_err());
    }
}

