# Contributing

$NAME is open to contributions by anyone. As long as you are a human, you are welcome.\
Wankers over Clankers after all. This doesn’t mean you can’t use AI to assist you - However, you are expected to be able to explain your code and any decisions made.

## Contribution Rules

1. I (Xqhare) am the benevolent dictator of this project.
2. Anyone can open a pull request.
3. You may only create an issue after you have created a pull request addressing that issue.
    - This rule only applies to first time contributors.
4. All other issues will be purged and ignored.
5. ~~❌ US Citizens~~ US Citizens are forgiven for now, but they're on a short leash. Why? Because its funny.

These are the rules. 
Don't like them? Fork the project and move on with your life.

## Documentation Style

This project uses [rustdoc](https://doc.rust-lang.org/rustdoc/) for documentation.

All members of the public API must have documentation.

In general documentation should be written in the following format:

```text

[One Sentence Description]

[Longer Description]

# Arguments

* `[Argument Name]` - [Description]

# Returns

[Description]

# Errors

[Description]

# Panics

[Description]

# Example

[Code Example]
```

Only use headings that are applicable to the documentation you are writing.

For example:

```rust

/// Public API entry point
///
/// This is the entry point for the public API. Call this function first to set up the API.
/// All other functions are called through the returned API object.
///
/// # Arguments
///
/// * `id` - The ID of the API.
/// * `name` - The name of the API.
///
/// # Returns
///
/// A Result containing the `API` object.
///
/// # Errors
///
/// An error will be returned if the API could not be set up because of IO or permission errors.
///
/// # Panics
///
/// This function will panic if the API could not be set up because of some other reason.
///
/// # Example
///
/// ```rust
///
/// let api = public_api(1, "My API".to_string());
/// assert!(api.is_ok());
///
/// ```
pub fn public_api(id: u32, name: String) -> Result<API, Error> {}

```

### Code Examples

All code examples should make use of the [assert](https://doc.rust-lang.org/std/macro.assert.html) macro to increase the test coverage.

## Git Commit Formatting
Commit messages should follow the format:
`git commit -m "$VERB($LOCATION): $DESCRIPTION"`

**Example:** `git commit -m "add(lib): public API"`

**Verbs:**
- `add`: Add something new to something already existing.
- `change`: Change something already existing. Use if changes in behavior are made.
- `remove`: Remove something from something already existing.
- `feat`: Add a new feature.
- `fix`: Fix a bug.
- `test`: Add or update tests.
- `doc`: Add or update documentation.
- `chore`: Change the build process or auxiliary tools like formatting.
- `update`: Update something already existing. Use if no change in behavior is made.

**Location:**
- A shortened version of the file path, excluding the top-level `src/`.
- `mod.rs` files are referred to by their directory name (e.g., `src/error/mod.rs` becomes `error`).

**Description:**
- A short description of the change.
- You may use imperative, present, or past tense.

### Special Cases
- **Automated changes:** For large-scale changes like `cargo fmt`, use: `git commit -a -m "ran cargo fmt"`.
- **Clippy fixes:** For clippy-related changes, commit each completed file separately without `$LOCATION` or `$VERB`: `git commit -m "make clippy happy"`.
- **Version bumps:**
    - Patch: `git commit -m "patch: 0.0.1 - description"`
    - Minor: `git commit -m "minor_ver: 0.1.0 - description"`
    - Major: `git commit -m "major_ver: 1.0.0 - description"`
