use std::path::{Path, PathBuf};

use nemesis::NemesisError;

use crate::{error::NomosResult, tags::Tags};

#[derive(Debug, Clone)]
pub struct FileData {
    pub path: PathBuf,
    pub line: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
    Blocked,
    Deferred,
    Cut,
}

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

#[derive(Debug, Clone)]
pub struct Dependency {
    pub title: String,
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
        if word.starts_with('@') {
            tags.add_kind(&word[1..]);
        } else if word.starts_with('#') {
            tags.add_location(&word[1..]);
        } else if word.contains('=') {
            let (key, value) = word.split_once('=').unwrap();
            if key == "dep" {
                if value.contains(':') {
                    let (project_name, dep_task_title) = value.split_once(':').unwrap();
                    dependencies.add(Dependency {
                        title: dep_task_title.to_string(),
                        project: Some(project_name.to_string()),
                    });
                } else {
                    dependencies.add(Dependency {
                        title: value.to_string(),
                        project: None,
                    });
                }
            }
            tags.add_metadata_tag(key, value);
        }
    }
    (tags, dependencies)
}
