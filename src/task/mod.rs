use std::{fmt::Display, iter::Peekable, path::Path, str::Lines};

use athena::local_date::LocalDate;
use nemesis::NemesisError;

use crate::{
    NomosError,
    error::{NomosResult, Parser},
    notes::{Note, Notes},
    parser::parse_line,
    tags::Tags,
    task::{
        utils::{make_dates, make_line, make_priority, make_status, make_title},
        validate::validate_task,
    },
    utils::{Dependencies, FileData, TaskStatus, make_tags_and_dependencies_from_line, padding},
};

mod utils;
mod validate;

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
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Task> {
        self.0.iter_mut()
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

/// A task
#[derive(Debug, Clone)]
pub struct Task {
    /// Amount of parents
    pub(crate) parents_amount: u32,
    /// Task status, Mandatory
    pub status: TaskStatus,
    /// Task priority, Optional
    pub priority: Option<char>,
    /// Title, Mandatory must be unique in the same project
    pub title: String,
    /// Inception date, Optional
    pub inception_date: Option<LocalDate>,
    /// Completion date, Optional (requires inception date to be set)
    pub completion_date: Option<LocalDate>,
    /// Tags, Optional
    pub tags: Tags,
    /// Dependencies, Optional
    pub(crate) dependencies: Dependencies,
    /// Complete with tags and dependencies
    pub description: Option<String>,
    /// All Notes, Optional
    pub notes: Option<Notes>,
    /// All sub-tasks, Optional
    pub sub_tasks: Option<Tasks>,
    /// File data
    pub file_data: FileData,
    /// Project name
    pub project: String,
}

/// Display implementation
///
/// Implements standard `{}` formatting as nomos compliant, and `{#}` formatting as a pretty string
/// for display in the terminal
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", padding(self.parents_amount.wrapping_mul(4)))?;
        if f.alternate() {
            match self.status {
                TaskStatus::Open => write!(f, " [ ] <Open>")?,
                TaskStatus::InProgress => write!(f, " [/] <In Progress>")?,
                TaskStatus::Done => write!(f, " [x] <Done>")?,
                TaskStatus::Blocked => write!(f, " [B] <Blocked>")?,
                TaskStatus::Deferred => write!(f, " [D] <Deferred>")?,
                TaskStatus::Cut => write!(f, " [C] <Cut>")?,
            };
            if let Some(prio) = self.priority {
                write!(f, " Priority {prio}")?;
            };
            if let Some(inception_date) = self.inception_date {
                write!(f, " Inception date: {inception_date}")?;
            }
            if let Some(completion_date) = self.completion_date
                && self.status == TaskStatus::Done
            {
                write!(f, " Completion date: {completion_date}")?;
            }
            write!(f, "\n")?;
            write!(f, "{}", padding(self.parents_amount.wrapping_mul(4)))?;
            write!(f, "{}", self.title)?;
            if let Some(description) = &self.description
                && !description.is_empty()
            {
                write!(f, "\n")?;
                let pad = if self.parents_amount == 0 {
                    2
                } else {
                    self.parents_amount.wrapping_mul(6)
                };
                write!(f, "{}", padding(pad))?;
                write!(f, "{}", description)?;
            }
            if let Some(notes) = &self.notes
                && !notes.iter().count().eq(&0)
            {
                write!(f, "\n")?;
                for (i, note) in notes.iter().enumerate() {
                    write!(f, "{}", padding(self.parents_amount.wrapping_mul(8)))?;
                    write!(f, "* {}", note.text)?;
                    if !i.eq(&notes.iter().count().saturating_sub(1)) {
                        write!(f, "\n")?;
                    }
                }
            }
            if let Some(sub_tasks) = &self.sub_tasks
                && !sub_tasks.iter().count().eq(&0)
            {
                write!(f, "\n")?;
                for (i, sub_task) in sub_tasks.iter().enumerate() {
                    write!(f, "{:#}", sub_task)?;
                    if !i.eq(&sub_tasks.iter().count().saturating_sub(1)) {
                        write!(f, "\n")?;
                    }
                }
            }
        } else {
            match self.status {
                TaskStatus::Open => write!(f, "- [ ] ")?,
                TaskStatus::InProgress => write!(f, "- [/] ")?,
                TaskStatus::Done => write!(f, "- [x] ")?,
                TaskStatus::Blocked => write!(f, "- [B] ")?,
                TaskStatus::Deferred => write!(f, "- [D] ")?,
                TaskStatus::Cut => write!(f, "- [C] ")?,
            };
            if let Some(char) = self.priority {
                write!(f, "({char}) ")?;
            };
            write!(f, "{}", self.title)?;
            let has_metadata = self.inception_date.is_some()
                || self.completion_date.is_some()
                || self.description.as_deref().map_or(false, |d| !d.trim().is_empty())
                || self.notes.is_some()
                || self.sub_tasks.is_some();
            if has_metadata {
                write!(f, " :: ")?;
                if let Some(inception_date) = self.inception_date {
                    write!(f, "{} ", inception_date.to_string())?;
                }
                if let Some(completion_date) = self.completion_date
                    && self.status == TaskStatus::Done
                {
                    write!(f, "{} ", completion_date.to_string())?;
                }
                if let Some(description) = &self.description {
                    write!(f, "{}", description)?;
                }
            }
            if let Some(notes) = &self.notes
                && !notes.iter().count().eq(&0)
            {
                write!(f, "\n")?;
                for note in notes.iter() {
                    write!(f, "{}", padding(self.parents_amount.wrapping_mul(8)))?;
                    write!(f, "* {}", note.text)?;
                }
            }
            if let Some(sub_tasks) = &self.sub_tasks
                && !sub_tasks.iter().count().eq(&0)
            {
                write!(f, "\n")?;
                for sub_task in sub_tasks.iter() {
                    write!(f, "{}", sub_task)?;
                }
            }
        }
        Ok(())
    }
}

impl Task {
    /// Returns a pretty string.
    ///
    /// # Notes
    /// Uses the Display implementation.
    /// To constuct a valid `nomos` task use the implementation of `Display` for `Task` via `println!("{task}");`
    ///
    /// # Usage
    /// `println!("{}", task.pretty_string());`
    pub fn pretty_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", padding(self.parents_amount.wrapping_mul(4)))?;

        Ok(())
    }
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
        project: Option<String>,
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
                    project.clone(),
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
        // Assumes indent level is a multiple of 4
        let parents_amount = indent_level.saturating_div(4);

        if project.is_none() {
            return Err(NemesisError::new(
                "nomos::parser::task::new_from_line",
                NomosError::Parser(Parser::Task("Task must be in a project".to_string())),
            )
            .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
        }

        let task = Task {
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
            project: project.expect("Validated above"),
            parents_amount,
        };
        validate_task(&task)?;
        Ok(task)
    }
}
