use std::path::Path;

use nemesis::NemesisError;

use crate::{
    error::{NomosError, NomosResult, Parser},
    tags::Tags,
    utils::make_tags_and_dependencies_from_line,
};

/// A list of notes
#[derive(Debug, Clone)]
pub struct Notes {
    /// All notes
    pub notes: Vec<Note>,
}

impl Notes {
    /// Adds a note
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }
    /// Removes a note
    pub fn remove_note(&mut self, index: usize) {
        self.notes.swap_remove(index);
    }
    /// Returns an iterator over the notes
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Note> {
        self.notes.iter_mut()
    }
    /// Returns an iterator over the notes
    pub fn iter(&self) -> impl Iterator<Item = &Note> {
        self.notes.iter()
    }
}

impl From<Vec<Note>> for Notes {
    fn from(value: Vec<Note>) -> Self {
        Self { notes: value }
    }
}

/// A note
#[derive(Debug, Clone)]
pub struct Note {
    /// The text of the note with tags
    pub text: String,
    /// All tags of the note
    pub tags: Tags,
    /// The line number of the note
    pub line: u32,
}
impl Note {
    /// Creates a task from a line
    ///
    /// # Notes
    /// Expects the supplied line to start with `* `
    pub fn new_from_line(line: &str, file_path: &Path, line_number: &mut u32) -> NomosResult<Note> {
        let line = make_line(line, file_path, *line_number)?;

        let (tags, dependencies) = make_tags_and_dependencies_from_line(line);
        if dependencies.iter().count() > 0 {
            return Err(NemesisError::new(
                "nomos::parser::note::new_from_line",
                NomosError::Parser(Parser::Note(format!(
                    "Note cannot have dependencies: {line}"
                ))),
            )
            .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
        } else {
            Ok(Note {
                text: line.to_string(),
                tags,
                line: *line_number,
            })
        }
    }
}

fn make_line<'line>(
    line: &'line str,
    file_path: &Path,
    line_number: u32,
) -> NomosResult<&'line str> {
    if let Some(line) = line.strip_prefix("* ") {
        Ok(line.trim_start())
    } else {
        return Err(NemesisError::new(
            "nomos::parser::note::new_from_line",
            NomosError::Parser(Parser::Note(format!(
                "Note must begin with a '* '. Found: {line}"
            ))),
        )
        .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
    }
}
