# Design Spec: Nomos Parser Fixes and Testing

* **Date**: 2026-06-19
* **Status**: Approved / Draft

---

## 1. Goal

The objective is to fix the critical parsing and line number accounting bugs in the **nomos** parser library, resolve all compiler/lint warnings, and implement a robust integration and unit testing suite in [tests/parser_tests.rs](file:///home/xqhare/Adytum/Programming/rust/nomos/tests/parser_tests.rs) to ensure future regressions are prevented.

---

## 2. Detailed Changes

### 2.1. `src/task.rs`
* **Parent Line Preservation**:
  At the beginning of `Task::new_from_line`, capture `let parent_line = *line_number;`. Assign `parent_line` to `file_data.line` at the end of the function.
* **Indentation-Aware Peek and Consume Loop**:
  Refactor the subtask/note recursion loop:
  ```rust
  let child_indent = indent_level.saturating_add(4);
  let child_prefix = " ".repeat(child_indent as usize);
  while let Some(next_line) = lines.peek() {
      if next_line.starts_with(&child_prefix) {
          // Consume the line from the iterator
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
  ```

### 2.2. `src/notes.rs`
* **Remove Double-Increment**:
  Remove `*line_number = line_number.saturating_add(1);` from `Note::new_from_line`. The parent loop or main parsing loop handles consuming and incrementing.

### 2.3. `src/utils.rs`
* **Fix Symbol Prefix Mapping**:
  In `make_tags_and_dependencies_from_line`, match the prefix characters correctly as per [v0.md](file:///home/xqhare/Adytum/Programming/rust/nomos/spec/v0.md):
  ```rust
  if word.starts_with('+') {
      tags.add_kind(&word[1..]);
  } else if word.starts_with('@') {
      tags.add_location(&word[1..]);
  } else if word.starts_with('#') {
      tags.add_generic_tag(&word[1..]);
  } else if word.contains('=') {
      ...
  }
  ```

### 2.4. `src/tags.rs`
* **Fix Clone Double Reference warning**:
  Replace `self.metadata_tags.remove(tag.clone());` with `self.metadata_tags.remove(*tag);` in `remove_metadata_tags`.

### 2.5. Documentation and Cleanliness
* Write docstrings for all public structs, fields, functions, methods, and enums to eliminate `missing_docs` compiler warnings.

---

## 3. Testing Strategy

We will create [tests/parser_tests.rs](file:///home/xqhare/Adytum/Programming/rust/nomos/tests/parser_tests.rs) which tests:
1. **Basic Task Parsing**: Status, Priority, Title, Dates, and Description.
2. **Subtask Parsing (Multiple Indentation Levels)**: Tasks with child tasks and grand-child tasks.
3. **Notes association & nesting**: Notes attached to tasks at the correct indentation.
4. **Line number validation**: Confirm that parent tasks, subtasks, and notes all report their actual starting line numbers correctly.
5. **Tag parsing**: Verify `+kind`, `@location`, `#generic`, and `key=val` are properly parsed, including values with spaces wrapped in double quotes.
6. **Cycle Detection**: Ensure that Kahn's topological sorting in `Nomos::new` detects cycles and handles dependency constraints properly.
