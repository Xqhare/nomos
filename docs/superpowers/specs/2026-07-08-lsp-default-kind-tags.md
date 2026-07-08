# Spec: LSP Default Kind Tags

## 1. Overview & Goals
Provide a set of standard, default kind tags when autocompleting task types with `+` in the Nomos LSP server.

## 2. Proposed Changes
In `src/lsp/capabilities.rs`, pre-populate the `kind_tags` HashSet in `get_completions` with the following static default tags:
- `bug`
- `feature`
- `library`
- `binary`
- `release`
- `documentation`
- `test`
- `refactor`

These defaults will be merged with any unique custom kind tags crawled from the project tasks recursively.

## 3. Testing
We will add a new assertion in our existing test `test_recursive_completions` in `src/lsp/capabilities.rs` to verify that defaults like `bug` and `refactor` are returned in the kind tags completion results.
