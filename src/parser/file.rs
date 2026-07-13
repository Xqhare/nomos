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

/// Parses a file into Tasks.
pub fn parse_file<P: Into<PathBuf>>(file_path: P, project: Option<String>) -> NomosResult<Tasks> {
    let file_path = file_path.into();
    let file = read_file(&file_path)?;
    parse_string(&file, &file_path, project)
}

/// Parse a string into tasks
///
/// Designed this way for testability
pub(crate) fn parse_string(
    file: &str,
    file_path: &Path,
    project: Option<String>,
) -> NomosResult<Tasks> {
    let mut lines = file.lines().peekable();
    // Overallocates, but shrinks when needed at the end
    let mut out: Vec<Task> = Vec::with_capacity(lines.size_hint().0);
    let mut line_number: u32 = 1;

    // Detect and consume the HTML comment header `<!-- nomos: X -->` if it is the first non-empty line
    while let Some(line) = lines.peek() {
        if line.is_empty() {
            lines.next();
            line_number = line_number.saturating_add(1);
        } else {
            let trimmed = line.trim();
            if trimmed.starts_with("<!-- nomos:") && trimmed.ends_with("-->") {
                lines.next();
                line_number = line_number.saturating_add(1);
            }
            break;
        }
    }

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
            project.clone(),
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
    project: Option<String>,
) -> NomosResult<()> {
    if line.starts_with("- ") {
        // Task
        task_out.push(Task::new_from_line(
            line,
            &file_path,
            line_number,
            lines,
            indent_level,
            project,
        )?);
    } else if line.starts_with("* ") {
        if indent_level > 0 {
            note_out.push(Note::new_from_line(line, &file_path, line_number)?);
        }
    } else {
        // Ignored line
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::TaskStatus;
    use std::path::Path;

    #[test]
    fn test_parse_nested_tasks_and_notes() {
        let content = "\
- [ ] Task 1 :: 2026-05-22 description +kind1 @loc1 key1=val1
    - [/] Subtask 1.1 :: 2026-05-23 sub-description +kind2 @loc2
    * Note 1.2 +kind3 #generic1
- [x] Task 2 :: description dep=\"Task 1\"
";
        let path = Path::new("test.md");
        let tasks = parse_string(content, path, Some("project1".to_string())).unwrap();
        let mut iter = tasks.iter();

        let t1 = iter.next().unwrap();
        assert_eq!(t1.title, "Task 1");
        assert_eq!(t1.status, TaskStatus::Open);
        assert_eq!(t1.file_data.line, 1);

        // Test tags for Task 1
        assert!(t1.tags.kind_tags.contains(&"kind1".to_string()));
        assert!(t1.tags.location_tags.contains(&"loc1".to_string()));

        let subtasks = t1.sub_tasks.as_ref().unwrap();
        let mut sub_iter = subtasks.iter();
        let st1 = sub_iter.next().unwrap();
        assert_eq!(st1.title, "Subtask 1.1");
        assert_eq!(st1.status, TaskStatus::InProgress);
        assert_eq!(st1.file_data.line, 2);
        assert!(st1.tags.kind_tags.contains(&"kind2".to_string()));
        assert!(st1.tags.location_tags.contains(&"loc2".to_string()));

        let notes = t1.notes.as_ref().unwrap();
        let mut note_iter = notes.notes.iter();
        let n1 = note_iter.next().unwrap();
        assert_eq!(n1.text, "Note 1.2 +kind3 #generic1");
        assert_eq!(n1.line, 3);
        assert!(n1.tags.kind_tags.contains(&"kind3".to_string()));
        assert!(n1.tags.generic_tags.contains(&"generic1".to_string()));

        let t2 = iter.next().unwrap();
        assert_eq!(t2.title, "Task 2");
        assert_eq!(t2.status, TaskStatus::Done);
        assert_eq!(t2.file_data.line, 4);
    }

    #[test]
    fn test_quote_stripping_in_dependencies() {
        use crate::utils::make_tags_and_dependencies_from_line;
        let line = "- [ ] Task :: dep=thoth:\"Add emoji support\"";
        let (_tags, deps) = make_tags_and_dependencies_from_line(line);
        let dep = deps.iter().next().unwrap();
        assert_eq!(dep.title, "Add emoji support");
        assert_eq!(dep.project, Some("thoth".to_string()));
    }

    #[test]
    fn test_v1_optional_delimiter_and_digit_prio() {
        let content = "\
- [ ] (1) Simple task with no delimiter
- [ ] (2) Another task :: 2026-07-12
";
        let path = Path::new("test_v1.md");
        let tasks = parse_string(content, path, Some("proj".to_string())).unwrap();
        let mut iter = tasks.iter();

        let t1 = iter.next().unwrap();
        assert_eq!(t1.title, "Simple task with no delimiter");
        assert_eq!(t1.priority, Some('1'));

        let t2 = iter.next().unwrap();
        assert_eq!(t2.title, "Another task");
        assert_eq!(t2.priority, Some('2'));
    }

    #[test]
    fn test_relaxed_markdown_parsing() {
        let content = "\
# README
This is a standard markdown file description.

- [ ] Real Task :: +tag
    * Valid note
    Some random paragraph to ignore under subtask level.
- [ ] Another Real Task
";
        let path = Path::new("README.md");
        let tasks = parse_string(content, path, Some("proj".to_string())).unwrap();
        assert_eq!(tasks.iter().count(), 2);
    }

    #[test]
    fn test_html_comment_header_parsing() {
        let content = "\
<!-- nomos: 1 -->
- [ ] Comment Task
";
        let path = Path::new("tasks.nomos");
        let tasks = parse_string(content, path, Some("proj".to_string())).unwrap();
        assert_eq!(tasks.iter().count(), 1);
        let t = tasks.iter().next().unwrap();
        assert_eq!(t.title, "Comment Task");
        assert_eq!(t.file_data.line, 2);
    }

    #[test]
    fn test_short_task_without_delimiter() {
        let content = "- [ ] a\n";
        let path = Path::new("test_short.nomos");
        let tasks = parse_string(content, path, Some("proj".to_string())).unwrap();
        let mut iter = tasks.iter();
        let t = iter.next().unwrap();
        assert_eq!(t.title, "a");
    }
    #[test]
    fn test_short_task_without_delimiter_no_newline() {
        let content = "- [ ] a";
        let path = Path::new("test_short.nomos");
        let tasks = parse_string(content, path, Some("proj".to_string())).unwrap();
        let mut iter = tasks.iter();
        let t = iter.next().unwrap();
        assert_eq!(t.title, "a");
    }
}
