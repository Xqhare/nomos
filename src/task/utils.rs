use std::path::Path;

use athena::local_date::LocalDate;
use nemesis::NemesisError;

use crate::{
    error::{NomosError, NomosResult, Parser},
    utils::TaskStatus,
};

pub fn make_dates<'line>(line: &'line str) -> (Option<LocalDate>, Option<LocalDate>) {
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

pub fn make_title<'line>(
    line: &'line str,
    file_path: &'line Path,
    line_number: u32,
) -> NomosResult<(&'line str, &'line str)> {
    let mut match_pattern = " :: ";
    // Slight relaxation of the parsing rules for small tasks with no following description
    // Easy to forget a space at the end; Intent should be clear enough
    if line.ends_with(" ::") {
        match_pattern = " ::";
    }
    match line.split_once(match_pattern) {
        Some((title, rest_line)) => Ok((title, rest_line)),
        None => Err(NemesisError::new(
            "nomos::parser::task::new_from_line",
            NomosError::Parser(Parser::Task(format!(
                "Could not split title and description. Did not find title delimiter: ' :: ' in line: {line}."
            )))
        ).add_ctx(format!("Line: {line_number} in file: {file_path:?}")))
    }
}

pub fn make_priority<'line>(
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

pub fn make_status<'line>(
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

pub fn make_line<'line>(
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
