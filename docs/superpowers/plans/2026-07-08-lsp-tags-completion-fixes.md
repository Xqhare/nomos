# LSP Tags Completion Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix Nomos LSP autocomplete completions for kind tags (`+`), location tags (`@`), and metadata key-value pairs (`key=value`) by recursively collecting tags/keys and matching prefixes in the cursor token.

**Architecture:** Crawl all tasks recursively to collect all kinds, locations, metadata keys, and metadata values. Analyze the last token in the cursor line to match context. Add '=' as a trigger character in `initialize` capabilities.

**Tech Stack:** Rust, Neovim LSP.

---

### Task 1: Add Unit Tests for Recursive Tag and Key-Value Autocomplete

**Files:**
- Modify: `src/lsp/capabilities.rs:370-373`

- [ ] **Step 1: Write the failing tests**
  Add new tests inside the inline `tests` module in `src/lsp/capabilities.rs` verifying nested tags and key-value value completion.

  Add this code to `src/lsp/capabilities.rs` under `mod tests`:
  ```rust
      #[test]
      fn test_recursive_completions() {
          let temp_dir = env::temp_dir().join(format!("nomos_test_recursive_{}", std::process::id()));
          fs::create_dir_all(&temp_dir).unwrap();

          let task_file = temp_dir.join("tasks.md");
          fs::write(
              &task_file,
              "- [ ] Parent Task :: +parent_kind @parent_loc\n    - [ ] Subtask :: +sub_kind @sub_loc est=3d owner=xqhare\n"
          ).unwrap();

          let config_file = temp_dir.join("config.json");
          let config_content = format!(
              r#"{{"search_bases":[], "files":{{"my_proj":"{}"}}}}"#,
              task_file.to_str().unwrap().replace('\\', "/")
          );
          fs::write(&config_file, config_content).unwrap();

          let nomos = Nomos::new(&config_file).ok();
          assert!(nomos.is_some());

          // Test recursive kind tags (including subtask kind tag)
          let comps_kind = get_completions(&nomos, "+", 1, "my_proj");
          let items_kind = comps_kind.as_array().unwrap();
          let kind_labels: Vec<&str> = items_kind.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap()).collect();
          assert!(kind_labels.contains(&"parent_kind"));
          assert!(kind_labels.contains(&"sub_kind"));

          // Test recursive location tags (including subtask location tag)
          let comps_loc = get_completions(&nomos, "Buy @", 5, "my_proj");
          let items_loc = comps_loc.as_array().unwrap();
          let loc_labels: Vec<&str> = items_loc.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap()).collect();
          assert!(loc_labels.contains(&"parent_loc"));
          assert!(loc_labels.contains(&"sub_loc"));

          // Test metadata key suggestions
          let comps_keys = get_completions(&nomos, "es", 2, "my_proj");
          let items_keys = comps_keys.as_array().unwrap();
          let key_labels: Vec<&str> = items_keys.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap()).collect();
          assert!(key_labels.contains(&"est="));
          assert!(key_labels.contains(&"owner="));

          // Test metadata value suggestion when '=' is typed
          let comps_vals = get_completions(&nomos, "est=", 4, "my_proj");
          let items_vals = comps_vals.as_array().unwrap();
          let val_labels: Vec<&str> = items_vals.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap()).collect();
          assert!(val_labels.contains(&"3d"));

          let _ = fs::remove_dir_all(&temp_dir);
      }
  ```

- [ ] **Step 2: Run tests to verify they fail/don't compile**
  Run: `cargo test lsp::capabilities::tests::test_recursive_completions`
  Expected: FAIL (either compilation errors for missing key-value autocomplete logic, or failure asserting subtask tags are present)

- [ ] **Step 3: Commit the failing test**
  Run:
  ```bash
  git add src/lsp/capabilities.rs
  git commit -m "test(lsp): add failing unit tests for recursive tags and key-value completion"
  ```

---

### Task 2: Implement Recursive Crawler and Token Analyzer

**Files:**
- Modify: `src/lsp/capabilities.rs:80-170`

- [ ] **Step 1: Write implementation for recursive collection and prefix parsing**
  Replace `get_completions` in `src/lsp/capabilities.rs` with the recursive crawling helper and token analyzer.

  ```rust
  /// Generate LSP completions
  pub fn get_completions(
      nomos: &Option<Nomos>,
      current_line: &str,
      character_pos: usize,
      current_project: &str,
  ) -> XffValue {
      let mut items = Vec::new();

      let before_cursor = if character_pos <= current_line.len() {
          &current_line[..character_pos]
      } else {
          current_line
      };

      // Split the before_cursor string by whitespace to get the last word
      let last_word = before_cursor
          .split(|c: char| c.is_whitespace())
          .last()
          .unwrap_or("");

      // Collect all tags recursively from the Nomos state
      let mut kind_tags = HashSet::new();
      let mut location_tags = HashSet::new();
      let mut metadata_keys = HashSet::new();

      if let Some(n) = nomos {
          fn collect_tags_recursively(
              task: &Task,
              kind_tags: &mut HashSet<String>,
              location_tags: &mut HashSet<String>,
              metadata_keys: &mut HashSet<String>,
          ) {
              for kind in &task.tags.kind_tags {
                  kind_tags.insert(kind.clone());
              }
              for loc in &task.tags.location_tags {
                  location_tags.insert(loc.clone());
              }
              for key in task.tags.metadata_tags.keys() {
                  metadata_keys.insert(key.clone());
              }

              if let Some(notes) = &task.notes {
                  for note in notes.iter() {
                      for kind in &note.tags.kind_tags {
                          kind_tags.insert(kind.clone());
                      }
                      for loc in &note.tags.location_tags {
                          location_tags.insert(loc.clone());
                      }
                      for key in note.tags.metadata_tags.keys() {
                          metadata_keys.insert(key.clone());
                      }
                  }
              }

              if let Some(sub_tasks) = &task.sub_tasks {
                  for sub_task in sub_tasks.iter() {
                      collect_tags_recursively(sub_task, kind_tags, location_tags, metadata_keys);
                  }
              }
          }

          for task in n.get_tasks().iter() {
              collect_tags_recursively(task, &mut kind_tags, &mut location_tags, &mut metadata_keys);
          }
      }

      if last_word.starts_with('@') {
          // Location Tag completion
          for loc in location_tags {
              let mut item = Object::new();
              item.insert("label", XffValue::from(loc));
              item.insert("kind", XffValue::from(14)); // Keyword/Tag
              items.push(XffValue::from(item));
          }
      } else if last_word.starts_with('+') {
          // Kind Tag completion
          for kind in kind_tags {
              let mut item = Object::new();
              item.insert("label", XffValue::from(kind));
              item.insert("kind", XffValue::from(14)); // Keyword/Tag
              items.push(XffValue::from(item));
          }
      } else if last_word.starts_with("dep=") {
          // Dependency completion
          if let Some(n) = nomos {
              let dep_str = &last_word[4..];
              if dep_str.contains(':') {
                  let parts: Vec<&str> = dep_str.split(':').collect();
                  let target_project = parts[0].trim_matches('"').trim();
                  for task in n.get_tasks().iter() {
                      if task.project == target_project {
                          let mut item = Object::new();
                          item.insert("label", XffValue::from(format!("\"{}\"", task.title)));
                          item.insert("kind", XffValue::from(18)); // Reference/Task
                          items.push(XffValue::from(item));
                      }
                  }
              } else {
                  let mut projects = HashSet::new();
                  for task in n.get_tasks().iter() {
                      projects.insert(task.project.clone());
                      if task.project == current_project {
                          let mut item = Object::new();
                          item.insert("label", XffValue::from(format!("\"{}\"", task.title)));
                          item.insert("kind", XffValue::from(18)); // Reference/Task
                          items.push(XffValue::from(item));
                      }
                  }
                  for proj in projects {
                      if proj != current_project {
                          let mut item = Object::new();
                          item.insert("label", XffValue::from(format!("{}:", proj)));
                          item.insert("kind", XffValue::from(9)); // Module/Project
                          items.push(XffValue::from(item));
                      }
                  }
              }
          }
      } else if last_word.contains('=') {
          // Metadata Value completion (e.g. key=value)
          let (key, _val) = last_word.split_once('=').unwrap();
          let mut values = HashSet::new();
          if let Some(n) = nomos {
              fn collect_metadata_values_recursively(
                  task: &Task,
                  target_key: &str,
                  values: &mut HashSet<String>,
              ) {
                  if let Some(v) = task.tags.metadata_tags.get(target_key) {
                      values.insert(v.clone());
                  }
                  if let Some(notes) = &task.notes {
                      for note in notes.iter() {
                          if let Some(v) = note.tags.metadata_tags.get(target_key) {
                              values.insert(v.clone());
                          }
                      }
                  }
                  if let Some(sub_tasks) = &task.sub_tasks {
                      for sub_task in sub_tasks.iter() {
                          collect_metadata_values_recursively(sub_task, target_key, values);
                      }
                  }
              }
              for task in n.get_tasks().iter() {
                  collect_metadata_values_recursively(task, key, &mut values);
              }
          }
          for v in values {
              let mut item = Object::new();
              item.insert("label", XffValue::from(v));
              item.insert("kind", XffValue::from(12)); // Value completion
              items.push(XffValue::from(item));
          }
      } else {
          // Metadata Key completion (suggesting key=)
          for key in metadata_keys {
              let mut item = Object::new();
              item.insert("label", XffValue::from(format!("{}=", key)));
              item.insert("kind", XffValue::from(14)); // Keyword/Tag
              items.push(XffValue::from(item));
          }
      }

      XffValue::from(items)
  }
  ```

- [ ] **Step 2: Run tests to verify they pass**
  Run: `cargo test lsp::capabilities::tests::test_recursive_completions`
  Expected: PASS

- [ ] **Step 3: Commit the implementation**
  Run:
  ```bash
  git add src/lsp/capabilities.rs
  git commit -m "fix(lsp): recursively collect tags and implement metadata key-value completions"
  ```

---

### Task 3: Register `=` as a Trigger Character

**Files:**
- Modify: `src/lsp/mod.rs:97-106`

- [ ] **Step 1: Add `=` to trigger characters**
  Modify [src/lsp/mod.rs](file:///home/xqhare/Adytum/Programming/rust/nomos/src/lsp/mod.rs) to add `"="` to `triggerCharacters`.

  Target:
  ```rust
                  let mut completion = Object::new();
                  completion.insert(
                      "triggerCharacters",
                      XffValue::from(vec![
                          XffValue::from("+"),
                          XffValue::from("@"),
                          XffValue::from("\""),
                          XffValue::from(":"),
                      ]),
                  );
  ```

  Replacement:
  ```rust
                  let mut completion = Object::new();
                  completion.insert(
                      "triggerCharacters",
                      XffValue::from(vec![
                          XffValue::from("+"),
                          XffValue::from("@"),
                          XffValue::from("\""),
                          XffValue::from(":"),
                          XffValue::from("="),
                      ]),
                  );
  ```

- [ ] **Step 2: Verify it builds and all tests pass**
  Run: `cargo test`
  Expected: All tests pass.

- [ ] **Step 3: Install the updated nomos-lsp locally**
  Run: `cargo install --path . --root /home/xqhare/.local`
  Expected: Installed package successful.

- [ ] **Step 4: Commit**
  Run:
  ```bash
  git add src/lsp/mod.rs
  git commit -m "fix(lsp): register '=' as a trigger character in capabilities"
  ```
