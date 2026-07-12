use std::path::{Path, PathBuf};

use nemesis::{NemesisError, NemesisResultExt};

use crate::{
    error::NomosResult,
    tags::Tags,
    task::{Task, Tasks},
};

/// File data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileData {
    /// Full file path
    pub path: PathBuf,
    /// Line number of entry
    pub line: u32,
}

/// Task status
#[derive(Debug, PartialEq, Clone)]
pub enum TaskStatus {
    /// Open
    Open,
    /// In progress
    InProgress,
    /// Done
    Done,
    /// Blocked
    Blocked,
    /// Deferred
    Deferred,
    /// Cut
    Cut,
}

/// Dependencies of a task
#[derive(Debug, Clone)]
pub struct Dependencies(Vec<Dependency>);

impl Dependencies {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn iter(&self) -> impl Iterator<Item = &Dependency> {
        self.0.iter()
    }
    pub fn remove(&mut self, index: usize) {
        self.0.swap_remove(index);
    }
    pub fn add(&mut self, dependency: Dependency) {
        self.0.push(dependency);
    }
}

/// Dependency of a task
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Task title that is depended on
    pub title: String,
    /// Project name if not same project
    pub project: Option<String>,
}

pub fn read_file<P: AsRef<Path>>(path: P) -> NomosResult<String> {
    match std::fs::read_to_string(path.as_ref()) {
        Ok(content) => Ok(content),
        Err(err) => Err(
            NemesisError::new("nomos::utils::read_file", err).add_ctx(&format!(
                "Failed to read and cast into a String. File path: {:?}",
                path.as_ref()
            )),
        ),
    }
}

pub fn read_file_u8<P: AsRef<Path>>(path: P) -> NomosResult<Vec<u8>> {
    match std::fs::read(path.as_ref()) {
        Ok(content) => Ok(content),
        Err(err) => Err(
            NemesisError::new("nomos::utils::read_file", err).add_ctx(&format!(
                "Failed to read and cast into a Vec<u8>. File path: {:?}",
                path.as_ref()
            )),
        ),
    }
}

pub fn save_file_u8<P: AsRef<Path>>(path: P, content: &[u8]) -> NomosResult<()> {
    match std::fs::write(path.as_ref(), content) {
        Ok(_) => Ok(()),
        Err(err) => Err(
            NemesisError::new("nomos::utils::save_file", err).add_ctx(&format!(
                "Failed to write to file. File path: {:?}",
                path.as_ref()
            )),
        ),
    }
}

/// Splits a line by whitespace, also supports the POSIX double quoted string syntax
pub fn split_into_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_double_quotes = false;
    for c in line.chars() {
        if c == '"' {
            in_double_quotes = !in_double_quotes;
        } else if c.is_whitespace() && !in_double_quotes {
            if !word.is_empty() {
                words.push(word);
                word = String::new();
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

pub fn make_tags_and_dependencies_from_line(line: &str) -> (Tags, Dependencies) {
    let words = split_into_words(line);
    let mut tags = Tags::new();
    let mut dependencies = Dependencies::new();
    for word in words {
        if word.starts_with('+') {
            tags.add_kind(&word[1..]);
        } else if word.starts_with('@') {
            tags.add_location(&word[1..]);
        } else if word.starts_with('#') {
            tags.add_generic_tag(&word[1..]);
        } else if word.contains('=') {
            let (key, value) = word.split_once('=').unwrap();
            if key == "dep" {
                if value.contains(':') {
                    let (project_name, dep_task_title) = value.split_once(':').unwrap();
                    dependencies.add(Dependency {
                        title: dep_task_title.replace('"', ""),
                        project: Some(project_name.replace('"', "")),
                    });
                } else {
                    dependencies.add(Dependency {
                        title: value.replace('"', ""),
                        project: None,
                    });
                }
            }
            tags.add_metadata_tag(key, value);
        }
    }
    (tags, dependencies)
}

pub fn calc_line_size(task: &Task) -> u32 {
    let mut size = 1u32;
    if let Some(_notes) = &task.notes {
        for _note in task.notes.iter() {
            size = size.saturating_add(1);
        }
    }
    if let Some(sub_tasks) = &task.sub_tasks {
        for sub_task in sub_tasks.iter() {
            size = size.saturating_add(calc_line_size(sub_task));
        }
    }

    size
}

/// Shifts all lines in a task, starting at a specific line by an offset.
pub fn shift_task_lines_by_offset(
    tasks: &mut Tasks,
    offset: i64,
    starting_line: u32,
) -> NomosResult<()> {
    for task in tasks.iter_mut() {
        if task.file_data.line >= starting_line {
            // Casting to make maths easier
            let new_ln = (task.file_data.line as i64).saturating_add(offset);
            task.file_data.line = new_ln as u32;
            sub_at_line(task.to_string().as_str(), new_ln, &task.file_data.path)?;
            if let Some(sub_tasks) = &mut task.sub_tasks {
                shift_task_lines_by_offset(sub_tasks, offset, new_ln as u32)?;
            }
            if let Some(notes) = &mut task.notes {
                for note in notes.iter_mut() {
                    if note.line >= starting_line {
                        let new_ln = (note.line as i64).saturating_add(offset);
                        note.line = new_ln as u32;
                        let complete_note = format!(
                            "{}* {}",
                            padding(task.parents_amount.saturating_add(1).saturating_mul(4)),
                            note.text
                        );
                        sub_at_line(&complete_note, new_ln, &task.file_data.path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn padding(amount: u32) -> String {
    const SPACE: &str = " ";
    if amount == 0 {
        return String::new();
    }
    SPACE.repeat(amount as usize)
}

// This could be done better via byte offsets instead of tracking the ln
pub fn sub_at_line(text: &str, line_number: i64, file_path: &Path) -> NomosResult<()> {
    let file = read_file(file_path).map_err(|err| {
        NemesisError::new("nomos::utils::sub_at_line", err)
            .add_ctx("Failed to read file")
            .add_ctx(format!("Line: {line_number} in file: {file_path:?}"))
    })?;
    let mut file = file.lines().collect::<Vec<&str>>();

    file[line_number.saturating_sub(1) as usize] = text;

    save_file_u8(file_path, &file.join("\n").as_bytes()).add_ctx("Sub_at_line failed.")
}
