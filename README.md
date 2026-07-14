# Nomos

The base for my project management solution

It follows my "All code written by me or part of rust's standard library and libc" philosophy.
You can learn more about that [here](https://blog.xqhare.net/posts/why_solve_problems/).

> [!note]
> As of right now, Nomos is a work in progress.
> The parser is stableish, but consider the entire project a minimum viable product at best.

## Roadmap

`Nomos` uses my [nomos](https://github.com/xqhare/nomos) project management system.

The roadmap for this project can be found in the [nomos.nomos](nomos.nomos) file.

All nomos files follow the syntax defined [here](https://github.com/Xqhare/nomos/blob/master/spec/).

## Features

- _**No dependencies**_: All code is written by me or part of std.
- LSP Server
- Command Line Tool
- Parsing Library
- Nomos File Specification
- Tree-sitter Grammar for Syntax Highlighting

## Naming

As with all my projects, Nomos is named after an ancient deity.
Learn more about my naming scheme [here](https://blog.xqhare.net/posts/explaining_the_pantheon/).

Nomos was a lesser Greek deity of laws, statutes, and ordinances.

## Usage

Nomos is available to be used both as a command line tool and as a Rust library.
It also provides an LSP server as well as tree-sitter grammar and syntax highlighting for Neovim.

### AI Agent Skill

The AI agent skill for Nomos is a work in progress.

It is currently available in the [xqhare/skills](https://github.com/Xqhare/skills) repository, split into a [CLI](https://github.com/Xqhare/skills/tree/master/nomos-cli) and [general](https://github.com/Xqhare/skills/tree/master/nomos) version.

> [!important]
> The `general` skill expects a git alias `git nomos` to be available.
> You can copy it from [here](https://github.com/xqhare/nomos/blob/master/examples/git_config.toml) and paste it into your `~/.gitconfig`.

> [!important]
> The `CLI` version expects the `nomos` program to be available in your PATH. (like `~/.local/bin/` or `~/.cargo/bin/`).
> See the [Command Line Tool](#command-line-tool) section for more information.

### Neovim Integration

Inside the [example](https://github.com/Xqhare/nomos/blob/master/examples/) directory, you can find a Neovim configuration that uses Nomos as a LSP server and integrates the tree sitter for syntax highlighting.
You will also find a `nomos.so` and `highlights.scm` file in the same directory.

Link or move them to `~/.local/share/nvim/site/parser/nomos.so` and `~/.config/nvim/queries/nomos/highlights.scm` respectively.

#### LSP Server

To use this server, you can run it from the command line:

```bash
cargo run --release --bin nomos-lsp
```

And then connect to it with your favorite LSP client.

For further information, see the [LSP specification](https://microsoft.github.io/language-server-protocol/specification)
and the documentation of the LSP client you are using.
You can find an example nvim configuration [here](https://github.com/Xqhare/nomos/blob/master/examples/nvim_lsp.lua).

### Command Line Tool

Clone the repository and run `cargo install --path .`

On first execution, Nomos creates a `config.json` file inside `~/.config/nomos` (or `~/.nomos` as a fallback).

To get started:

1. Run `nomos` by itself to initialize the config file.
2. Update the `search_bases` key in `config.json` with the paths pointing to root directories containing the projects you want to track.
3. You can also optionally update the `files` key with `"project_name": "path/to/specific/file.nomos"` if you want to track individual files directly.

An example global `config.json` containing all valid key-value pairs:
```json
{
  "search_bases": [
    "/home/xqhare/Adytum/Programming/rust/"
  ],
  "files": {
    "personal_tasks": "/home/xqhare/Adytum/personal.nomos"
  },
  "version": 1,
  "default_prio_level": 5
}
```

For example, if the path `~/projects/rust` is present in `search_bases`, Nomos crawls each of its subdirectories:

- It looks for a `nomos.json` file. If found, it reads each file specified by the key `task_files`.
- If no `nomos.json` is found, it falls back to looking for files named `nomos`, `todo`, `tasks`, or `roadmap` with a `.nomos` extension (or legacy `.md`/`.txt` files) in the project directory.
- It parses all discovered files and creates a task for each one.

An example project-specific `nomos.json` file containing all valid key-value pairs:
```json
{
  "task_files": [
    "TODO.nomos",
    "docs/roadmap.nomos"
  ],
  "version": 1,
  "default_prio_level": 5
}
```

#### Version Resolution

When processing a task file, Nomos resolves the format version using the following precedence pipeline:

1. **In-File Metadata Override**: If the first non-empty line of the file is an HTML comment matching `<!-- nomos: X -->` (where `X` is `0` or `1`), version `X` is used.
2. **Project Configuration (`nomos.json`)**: If a `version` key exists in the project configuration (e.g. `"version": 1`), it applies to all task files in that project.
3. **Global Configuration (`config.json`)**: If a `version` key exists in the global configuration, it acts as the default fallback.
4. **Extension Inference**: If no version is explicitly configured, files with a `.md` extension are parsed using **v0** rules, and files with a `.nomos` extension are parsed using **v1** rules.

#### Running CLI Commands

1. Confirm your configuration setup with:
   ```bash
   nomos validate
   ```
2. Once validated, retrieve all tasks sorted by priority and dependencies:
   ```bash
   nomos all
   ```
3. Or see the next tasks to work on:
   ```bash
   nomos next
   ```
4. Migrate legacy v0 projects/files to v1:
   ```bash
   nomos update
   ```

For more CLI options, run:
```bash
nomos --help
```

---

### Task File Syntax

> [!note]
> Nomos v1 supersedes the historic v0 version. Legacy v0 files using alphabetic priorities (A-Z) and mandatory delimiters can be automatically migrated to v1 (`.nomos`) using the `$ nomos update` CLI command.

Nomos task files are standard Markdown lists, allowing tasks to be embedded directly within documents like `README.md` (non-task lines are skipped by the relaxed parser).
- Tasks start with a hyphen (`-`) and must be followed by a status bracket (e.g., `- [ ]`).
- Notes start with an asterisk (`*`) and provide details for the preceding task.

#### Tasks

The syntax of a task is:
```text
- [Status] (Priority) Title :: [InceptionDate] [CompletionDate] Description +kindTag @locationTag keyTag=valueTag dep="Dependency Title"
```

- **Status**: Can be one of:
  - `[ ]` Open
  - `[/]` In Progress
  - `[x]` Completed
  - `[B]` Blocked
  - `[C]` Cut
  - `[D]` Deferred
- **Priority** (Optional): A single digit `(0)` through `(9)` enclosed in parentheses, where `(1)` is the highest priority, `(9)` is the lowest, and `(0)` is reserved for Extremely Important. If omitted, the scheduler treats the task as priority `5` (Important).
- **Title**: The title of the task. It must be unique within its project.
- **Delimiter**: A double colon with leading and trailing whitespace (` :: `) is optional, and is only required if dates, descriptions, tags, or dependencies are present. If omitted, the entire remaining text is parsed as the title.
- **Dates** (Optional): Optional date fields in `YYYY-MM-DD` format.
  - One date: `[InceptionDate]`
  - Two dates: `[InceptionDate] [CompletionDate]`. Note: `CompletionDate` can only be set if the status is resolved (`[x]` or `[C]`).
- **Description**: The description follows the date fields. It can contain tags and dependencies.

#### Tags

Nomos supports three types of tags:
- `+Kind`: Semantic categorization (e.g., `+bug`, `+feature`), defined with a leading `+` without whitespace.
- `@Location`: Location/subsystem target (e.g., `@src/main.rs`), defined with a leading `@`.
- `Key=Value`: Custom key-value pairs for metadata (e.g., `est=2d`, `owner=Xqhare`).

#### Dependencies & Kahn Sorting

Dependencies are represented as special key-value pairs:
- `dep="Dependency Title"` (same project)
- `dep="project_name:Dependency Title"` (cross-project)

Nomos schedules all tasks and subtasks using Kahn's topological sort:
- **Parent-Child Dependency**: A parent task implicitly depends on its child subtasks. Child subtasks are sorted and scheduled *before* their parent task.
- **Done vs. Cut**: A task marked as `Done` (`[x]`) can only explicitly depend on other completed (`Done`) tasks. If a dependency was `Cut` (`[C]`), the dependent task is blocked from completion. However, a parent task is allowed to be completed even if its subtasks were `Cut`.

#### Notes

Notes start with `*`. They must be associated with the task immediately above them at their respective indentation level and cannot define dependencies. Under v1, other indented prose lines that do not start with `- ` or `* ` are ignored.

#### Task Children / Hierarchy

A parent task can have child tasks immediately following it, indented by 4 spaces. Indentation level is syntactically significant.

Example of a Nomos task file:

```markdown
- [ ] (1) Integrate CLI toolkit :: 2026-05-22 Integrate Eshu +feature @src/main.rs
    - [ ] Setup argument builder :: Write command definitions
    - [x] Parse subcommands :: Test parser against standard inputs
    * Remember to check for std::env::args_os compatibility
    * Make sure we don't pull in any external parser dependencies
- [B] Run Kahn Sort :: dep="Integrate CLI toolkit" +feature @src/graph.rs
```

---

### Rust Library Usage

#### Importing

Add the following to your `Cargo.toml`:

```toml
[dependencies]
nomos = { git = "https://github.com/xqhare/nomos" }
```

#### Example

```rust,no_run
use nomos::Nomos;
use nomos::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Nomos using the global config file path
    let nomos = Nomos::new("config.json")?;

    // Retrieve all tasks parsed across crawl paths, sorted topographically
    let tasks = nomos.get_tasks();

    for task in tasks.iter() {
        println!("Task: {} [Status: {:?}]", task.title, task.status);
        if let Some(sub_tasks) = &task.sub_tasks {
            for sub_task in sub_tasks.iter() {
                println!("  -> Subtask: {}", sub_task.title);
            }
        }
    }

    Ok(())
}
```

## License

Nomos is distributed under the [MIT](https://github.com/xqhare/nomos/blob/master/LICENSE) license.

## Contributing

See [CONTRIBUTING](https://github.com/xqhare/nomos/blob/master/CONTRIBUTING.md) for contribution guidelines.
