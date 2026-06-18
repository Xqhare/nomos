use std::{iter::Peekable, path::Path, str::Lines};

use athena::local_date::LocalDate;
use nemesis::NemesisError;

use crate::{
    error::{NomosError, NomosResult, Parser},
    notes::{Note, Notes},
    parser::parse_line,
    tags::Tags,
    utils::{Dependencies, FileData, TaskStatus, make_tags_and_dependencies_from_line},
};

/// A collection of tasks
#[derive(Debug, Clone)]
pub struct Tasks(Vec<Task>);

impl From<Vec<Task>> for Tasks {
    fn from(value: Vec<Task>) -> Self {
        Self(value)
    }
}

impl Tasks {
    /// Creates an empty collection
    pub fn new() -> Self {
        Self(Vec::new())
    }
    /// Returns an iterator over the tasks
    pub fn iter(&self) -> impl Iterator<Item = &Task> {
        self.0.iter()
    }
    /// Returns a mutable iterator over the tasks
    ///
    /// # Notes
    /// Uses swap_remove
    pub fn remove(&mut self, index: usize) {
        self.0.swap_remove(index);
    }
    /// Adds a task
    pub fn push(&mut self, task: Task) {
        self.0.push(task);
    }
    /// Extends the collection
    pub fn extend(&mut self, tasks: Tasks) {
        self.0.extend(tasks.0);
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub status: TaskStatus,
    pub priority: Option<char>,
    pub title: String,
    pub inception_date: Option<LocalDate>,
    pub completion_date: Option<LocalDate>,
    pub tags: Tags,
    pub dependencies: Dependencies,
    /// Complete with tags and dependencies
    pub description: Option<String>,
    pub notes: Option<Notes>,
    pub sub_tasks: Option<Tasks>,
    pub file_data: FileData,
}

impl Task {
    /// Creates a task from a line
    ///
    /// # Notes
    /// Expects the supplied line to start with `- `
    pub fn new_from_line(
        line: &str,
        file_path: &Path,
        line_number: &mut u32,
        lines: &mut Peekable<Lines>,
        indent_level: u32,
    ) -> NomosResult<Task> {
        let parent_line = *line_number;
        // Strip prefix ("- ") and validate minimum length of 9
        let mut line = make_line(line, file_path, *line_number)?;
        let status = make_status(line, file_path, *line_number)?;
        line = &line[3..].trim_start(); // Strip status
        let priority = make_priority(line, file_path, *line_number)?;
        if priority.is_some() {
            line = &line[3..].trim_start(); // Strip priority
        }
        let title = {
            let (title, rest_line) = make_title(line, file_path, *line_number)?;
            line = rest_line.trim_start(); // Just to be sure
            title
        };
        let (inception_date, completion_date) = make_dates(line);
        if inception_date.is_some() {
            line = &line[10..].trim_start(); // Strip date
        }
        if completion_date.is_some() {
            line = &line[10..].trim_start(); // Strip date
        }
        let description = &line;
        let (tags, dependencies) = make_tags_and_dependencies_from_line(line);
        let mut sub_tasks: Vec<Task> = Vec::new();
        let mut notes: Vec<Note> = Vec::new();

        let child_indent = indent_level.saturating_add(4);
        let child_prefix = " ".repeat(child_indent as usize);

        while let Some(next_line) = lines.peek() {
            if next_line.starts_with(&child_prefix) {
                let next_line = lines.next().unwrap();
                *line_number = line_number.saturating_add(1);
                let stripped = &next_line[child_indent as usize..];
                parse_line(
                    stripped,
                    file_path,
                    &mut sub_tasks,
                    &mut notes,
                    lines,
                    line_number,
                    child_indent,
                )?;
            } else {
                break;
            }
        }
        let file_data = FileData {
            path: file_path.to_path_buf(),
            line: parent_line,
        };
        let sub_tasks = if sub_tasks.is_empty() {
            None
        } else {
            Some(sub_tasks.into())
        };
        let notes = if notes.is_empty() {
            None
        } else {
            Some(notes.into())
        };
        // TODO: Validate the task
        // - May only be status::done if all sub tasks are done
        // - Title, status are not empty
        Ok(Task {
            status,
            priority,
            title: title.to_string(),
            inception_date,
            completion_date,
            tags,
            dependencies,
            description: Some(description.to_string()),
            notes,
            sub_tasks,
            file_data,
        })
    }
}

fn make_dates<'line>(line: &'line str) -> (Option<LocalDate>, Option<LocalDate>) {
    // Date is always: `YYYY-MM-DD` == 10 chars.
    if line.len() < 10 {
        (None, None)
    } else {
        let potential_date = &line[0..10];
        if let Ok(date) = TryInto::<LocalDate>::try_into(potential_date) {
            let inception_date = Some(date);
            // There is a space between the date and the completion date
            if line.len() >= 21 {
                let potential_date = &line[11..21];
                if let Ok(date) = TryInto::<LocalDate>::try_into(potential_date) {
                    let completion_date = Some(date);
                    (inception_date, completion_date)
                } else {
                    (inception_date, None)
                }
            } else {
                (inception_date, None)
            }
        } else {
            (None, None)
        }
    }
}

fn make_title<'line>(
    line: &'line str,
    file_path: &'line Path,
    line_number: u32,
) -> NomosResult<(&'line str, &'line str)> {
    match line.split_once(" :: ") {
        Some((title, rest_line)) => Ok((title, rest_line)),
        None => Err(NemesisError::new(
            "nomos::parser::task::new_from_line",
            NomosError::Parser(Parser::Task(format!(
                "Could not split title and description. Did not find title delimiter: ' :: ' in line: {line}."
            )))
        ).add_ctx(format!("Line: {line_number} in file: {file_path:?}")))
    }
}

fn make_priority<'line>(
    line: &'line str,
    file_path: &'line Path,
    line_number: u32,
) -> NomosResult<Option<char>> {
    if line.starts_with('(') {
        let potential_priority = &line[1..3];
        if potential_priority.ends_with(')') {
            let prio = potential_priority[0..1]
                .chars()
                .next()
                .expect("Priority is US-ASCII");
            if prio.is_alphabetic() {
                Ok(Some(prio))
            } else {
                return Err(NemesisError::new(
                    "nomos::parser::task::new_from_line",
                    NomosError::Parser(Parser::Task(format!(
                        "Priority is not a letter. Got: {potential_priority}"
                    ))),
                )
                .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
            }
        } else {
            return Err(NemesisError::new(
                "nomos::parser::task::new_from_line",
                NomosError::Parser(Parser::Task(format!(
                    "Found '(' in priority position but no matching ')'. Got: {line}"
                ))),
            )
            .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
        }
    } else {
        Ok(None)
    }
}

fn make_status<'line>(
    line: &'line str,
    file_path: &'line Path,
    line_number: u32,
) -> NomosResult<TaskStatus> {
    // Status is guaranteed US-ASCII and 3 chars long
    let status = &line[0..3];
    if status.starts_with('[') && status.ends_with(']') {
        match status[1..2].to_lowercase().as_str() {
            " " => Ok(TaskStatus::Open),
            "/" => Ok(TaskStatus::InProgress),
            "x" => Ok(TaskStatus::Done),
            "b" => Ok(TaskStatus::Blocked),
            "d" => Ok(TaskStatus::Deferred),
            "c" => Ok(TaskStatus::Cut),
            _ => {
                return Err(NemesisError::new(
                    "nomos::parser::task::new_from_line",
                    NomosError::Parser(Parser::Task(format!("Unknown task status: {status}"))),
                )
                .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
            }
        }
    } else {
        return Err(NemesisError::new(
            "nomos::parser::task::new_from_line",
            NomosError::Parser(Parser::Task(format!("Unknown task status: {status}"))),
        )
        .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
    }
}

fn make_line<'line>(
    line: &'line str,
    file_path: &'line Path,
    line_number: u32,
) -> NomosResult<&'line str> {
    if let Some(line) = line.strip_prefix("- ") {
        // Smalles task: "- [ ] a :: ". Strip prefix => "[ ] a :: " = 9 chars.
        if line.len() < 9 {
            return Err(NemesisError::new(
                "nomos::parser::task::new_from_line",
                NomosError::Parser(Parser::Task(format!(
                    "Line {line_number} in file {file_path:?} is too short to be a task"
                ))),
            )
            .add_ctx(format!("Got line: {line}")));
        }
        Ok(line)
    } else {
        return Err(NemesisError::new(
            "nomos::parser::task::new_from_line",
            NomosError::Parser(Parser::Task(format!(
                "Task must begin with a '- '. Found: {line}"
            ))),
        )
        .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
    }
}
