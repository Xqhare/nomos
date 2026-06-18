# Parser Bug Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix subtask/note parsing, line tracking, tag prefix mapping, and compiler warnings in the `nomos` crate.

**Architecture:** We will implement unit tests inside `src/parser/file.rs` to capture existing parsing failures. Then we will modify `src/task.rs`, `src/notes.rs`, `src/utils.rs`, and `src/tags.rs` to fix the bugs and ensure all unit tests pass.

**Tech Stack:** Rust (std library, `athena`, `mawu`, `nemesis`, `horae` crates).

---

### Task 1: Fix Clone Double Reference warning in `src/tags.rs`

**Files:**
- Modify: `src/tags.rs:50-53`

- [ ] **Step 1: Write the fix**
  Replace the double-reference clone with a direct dereference.
  ```rust
  pub fn remove_metadata_tags(&mut self, tags: &[&str]) {
      for tag in tags {
          self.metadata_tags.remove(*tag);
      }
  }
  ```
- [ ] **Step 2: Run `cargo check` to verify the warning is resolved**
  Run: `cargo check`
  Expected: Code compiles, and the double-reference warning for `src/tags.rs` is gone.
- [ ] **Step 3: Commit**
  ```bash
  git add src/tags.rs
  git commit -m "fix(tags): resolve double reference clone warning"
  ```

---

### Task 2: Implement Failing Unit Tests in `src/parser/file.rs`

**Files:**
- Modify: `src/parser/file.rs:99` (append tests module)

- [ ] **Step 1: Write failing unit tests**
  Add unit tests at the bottom of `src/parser/file.rs` that verify:
  1. Correct parsing of nested subtasks and notes.
  2. Correct line number tracking for nested items.
  3. Correct parsing of `+kind`, `@location`, and `#generic` tags.
  ```rust
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
          let tasks = parse_string(content, path).unwrap();
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
  }
  ```
- [ ] **Step 2: Run tests to verify they fail**
  Run: `cargo test`
  Expected: FAIL (either due to recursive parsing panic, incorrect tags, or incorrect line numbers)
- [ ] **Step 3: Commit**
  ```bash
  git add src/parser/file.rs
  git commit -m "test(parser): add unit tests for nested parsing and tags"
  ```

---

### Task 3: Fix Tag Prefix Mapping in `src/utils.rs`

**Files:**
- Modify: `src/utils.rs:81-110`

- [ ] **Step 1: Implement prefix corrections**
  Correct prefix character checks to match the spec:
  ```rust
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
  ```
- [ ] **Step 2: Run tests to check progress**
  Run: `cargo test`
  Expected: Tests still fail, but tag checks in tests should start passing.
- [ ] **Step 3: Commit**
  ```bash
  git add src/utils.rs
  git commit -m "fix(parser): correct prefix symbol mapping for tags"
  ```

---

### Task 4: Fix Subtask Recursion and Line Numbering

**Files:**
- Modify: `src/task.rs:71-149`
- Modify: `src/notes.rs:42-63`

- [ ] **Step 1: Remove double-increment in Note parser**
  Update `src/notes.rs` by removing `*line_number = line_number.saturating_add(1);` from `Note::new_from_line`.
  ```rust
  impl Note {
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
  ```
- [ ] **Step 2: Fix nested parsing loop and track parent line number**
  Update `Task::new_from_line` in `src/task.rs` to track parent line and parse stripped child lines with indentation prefix checks.
  ```rust
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
  ```
- [ ] **Step 3: Run unit tests to verify they pass**
  Run: `cargo test`
  Expected: PASS
- [ ] **Step 4: Commit**
  ```bash
  git add src/task.rs src/notes.rs
  git commit -m "fix(parser): resolve subtask nesting recursion and line tracking bugs"
  ```
