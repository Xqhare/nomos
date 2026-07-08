# Spec: Nomos LSP Tags Completion Fixes

## 1. Overview & Goals
The goal is to fix issues with the Autocomplete / LSP completions in Nomos files when editing tasks:
- **Kind Tags (`+`)**: Currently, only `+Game` shows up because only top-level tasks are crawled. We need to collect kind tags recursively from subtasks and notes.
- **Location Tags (`@`)**: Similarly, only location tags on top-level tasks are collected. We need to collect them recursively.
- **Metadata Key-Values (`key=value`)**: Currently, key-value completions do not work at all. We will support completing keys as `key=` and, once `=` is typed or completed, auto-triggering completion of values previously associated with that key.

## 2. Proposed Changes

### Section 1: LSP Server Capabilities (Rust)
We will rewrite `get_completions` in `src/lsp/capabilities.rs` to:
1. **Recursive Tags Crawler**: Add a helper function to recursively collect location tags, kind tags, metadata keys, and metadata values from the current project's tasks, nested subtasks, and associated notes.
2. **Context Detection via Tokens**:
   - Extract the last word/token from `before_cursor`.
   - If it starts with `@`, return all collected location tags.
   - If it starts with `+`, return all collected kind tags.
   - If it starts with `dep=`, return dependency options (already implemented).
   - If it contains `=` (but not `dep=`), extract the key name and suggest all historical values for that key.
   - Otherwise, suggest all collected metadata keys formatted as `key=`.

### Section 2: LSP triggerCharacters Registration
In `src/lsp/mod.rs`, during the `initialize` request handling, add `"="` to the list of `triggerCharacters`:
```rust
XffValue::from("+"),
XffValue::from("@"),
XffValue::from("\""),
XffValue::from(":"),
XffValue::from("="),
```

## 3. Testing
We will add programmatic integration tests in `src/lsp/capabilities.rs` to verify that:
- Kind tags are recursively collected from subtasks and notes.
- Location tags are recursively collected from subtasks and notes.
- Metadata keys (like `est=`) are suggested.
- Metadata values (like `2d` for `est=`) are suggested when `=` is typed.
