use std::{
    iter::Peekable,
    path::{Path, PathBuf},
    str::Lines,
};

use nemesis::NemesisError;

use crate::{
    error::{NomosError, NomosResult, Parser},
    notes::Note,
    task::{Task, Tasks},
    utils::read_file,
};

pub fn parse_file<P: Into<PathBuf>>(file_path: P) -> NomosResult<Tasks> {
    let file_path = file_path.into();
    let file = read_file(&file_path)?;
    parse_string(&file, &file_path)
}

/// Parse a string into tasks
///
/// Designed this way for testability
pub(crate) fn parse_string(file: &str, file_path: &Path) -> NomosResult<Tasks> {
    let mut lines = file.lines().peekable();
    // Overallocates, but shrinks when needed at the end
    let mut out: Vec<Task> = Vec::with_capacity(lines.size_hint().0);
    let mut line_number: u32 = 1;

    while let Some(line) = lines.next() {
        if line.is_empty() {
            line_number = line_number.saturating_add(1);
            continue;
        }
        let mut notes_out: Vec<Note> = Vec::new();
        parse_line(
            line,
            file_path,
            &mut out,
            &mut notes_out,
            &mut lines,
            &mut line_number,
            0,
        )?;
        if notes_out.len() > 0 {
            // Dangling note
            return Err(NemesisError::new(
                "nomos::parser::file::parse_string",
                NomosError::Parser(Parser::Note(format!(
                    "Dangling note or Invalid line: {line}"
                ))),
            )
            .add_ctx(format!(
                "Line: {line_number} in file: {file_path:?} (line: {line})"
            )));
        }
        line_number = line_number.saturating_add(1);
    }
    out.shrink_to_fit();
    Ok(out.into())
}

pub(crate) fn parse_line(
    line: &str,
    file_path: &Path,
    task_out: &mut Vec<Task>,
    note_out: &mut Vec<Note>,
    lines: &mut Peekable<Lines>,
    line_number: &mut u32,
    indent_level: u32,
) -> NomosResult<()> {
    if line.starts_with("- ") {
        // Task
        task_out.push(Task::new_from_line(
            line,
            &file_path,
            line_number,
            lines,
            indent_level,
        )?);
    } else {
        if indent_level == 0 {
            // Dangling note or invalid line
            return Err(NemesisError::new(
                "nomos::parser::file::parse_line",
                NomosError::Parser(Parser::Note(format!(
                    "Dangling note or Invalid line: {line}"
                ))),
            )
            .add_ctx(format!("Line: {line_number} in file: {file_path:?}")));
        } else {
            // Note
            note_out.push(Note::new_from_line(line, &file_path, line_number)?);
        }
    }
    Ok(())
}
