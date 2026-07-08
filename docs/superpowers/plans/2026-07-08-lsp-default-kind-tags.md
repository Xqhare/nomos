# LSP Default Kind Tags Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pre-populate autocomplete suggestions with default kind tags (`bug`, `feature`, `library`, etc.) when the user type `+`.

**Architecture:** Initialize the `kind_tags` HashSet in `get_completions` with the default tags list.

**Tech Stack:** Rust.

---

### Task 1: Add Unit Test Assertions for Default Kind Tags

**Files:**
- Modify: `src/lsp/capabilities.rs:400-405`

- [ ] **Step 1: Write failing assertions**
  Add assertions to the existing `test_recursive_completions` test case in `src/lsp/capabilities.rs` to check that default tags like `bug` and `refactor` are suggested when completions for `+` are requested.

  Find the following code block in `src/lsp/capabilities.rs`'s `test_recursive_completions`:
  ```rust
          // Test recursive kind tags (including subtask kind tag)
          let comps_kind = get_completions(&nomos, "+", 1, "my_proj");
          let items_kind = comps_kind.as_array().unwrap();
          let kind_labels: Vec<&str> = items_kind.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap().as_str()).collect();
          assert!(kind_labels.contains(&"parent_kind"));
          assert!(kind_labels.contains(&"sub_kind"));
  ```

  And change it to add the assertions for `bug` and `refactor`:
  ```rust
          // Test recursive kind tags (including subtask kind tag)
          let comps_kind = get_completions(&nomos, "+", 1, "my_proj");
          let items_kind = comps_kind.as_array().unwrap();
          let kind_labels: Vec<&str> = items_kind.iter().map(|item| item.as_object().unwrap().get("label").unwrap().as_string().unwrap().as_str()).collect();
          assert!(kind_labels.contains(&"parent_kind"));
          assert!(kind_labels.contains(&"sub_kind"));
          assert!(kind_labels.contains(&"bug"));
          assert!(kind_labels.contains(&"refactor"));
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test lsp::capabilities::tests::test_recursive_completions`
  Expected: FAIL (assertion failed for `bug` or `refactor`)

- [ ] **Step 3: Commit the failing test**
  Run:
  ```bash
  git add src/lsp/capabilities.rs
  git commit -m "test(lsp): assert default kind tags are suggested"
  ```

---

### Task 2: Pre-populate Default Kind Tags in Autocomplete

**Files:**
- Modify: `src/lsp/capabilities.rs:104-108`

- [ ] **Step 1: Initialize kind_tags with defaults**
  Modify [src/lsp/capabilities.rs](file:///home/xqhare/Adytum/Programming/rust/nomos/src/lsp/capabilities.rs) inside `get_completions`:

  Target:
  ```rust
      // Collect all tags recursively from the Nomos state
      let mut kind_tags = HashSet::new();
      let mut location_tags = HashSet::new();
      let mut metadata_keys = HashSet::new();
  ```

  Replacement:
  ```rust
      // Collect all tags recursively from the Nomos state
      let mut kind_tags = HashSet::from([
          "bug".to_string(),
          "feature".to_string(),
          "library".to_string(),
          "binary".to_string(),
          "release".to_string(),
          "documentation".to_string(),
          "test".to_string(),
          "refactor".to_string(),
      ]);
      let mut location_tags = HashSet::new();
      let mut metadata_keys = HashSet::new();
  ```

- [ ] **Step 2: Run tests to verify they pass**
  Run: `cargo test`
  Expected: All tests pass.

- [ ] **Step 3: Build and install updated binary locally**
  Run: `cargo install --path . --root /home/xqhare/.local`
  Expected: Installed successfully.

- [ ] **Step 4: Commit**
  Run:
  ```bash
  git add src/lsp/capabilities.rs
  git commit -m "fix(lsp): add default kind tags to autocomplete suggestions"
  ```
