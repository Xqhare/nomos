# Loki & Nomos: Distributed Project Management System

Nomos is a decentralized, text-based project management system built in Rust. It utilizes a human-readable markdown-compatible task protocol (**Nomos**).

## 1. Overview & Goals

- **Data Sovereignty**: Tasks live directly with the code in markdown/text files.
- **Tool-Agnostic**: Read and write tasks in Neovim, standard Markdown previewers, or via custom CLI/TUI.
- **Zero External Dependencies**: Prioritizes the Rust standard library and internal custom crates (`Mawu` for JSON parsing).
- **Cross-Project Linkage**: Resolve dependencies and priority order across multiple projects using Kahn's topological sort.

---

## 2. Nomos Syntax Specification

Nomos files (e.g., `TODO.md`) are 100% compliant with standard Markdown list rendering. To distinguish tasks from notes at a glance in plain text, specific list markers are enforced:

### List Markers
- `-` (Hyphen): Declares a **Task**. Must be followed by a status bracket (e.g., `- [ ]`).
- `*` (Asterisk): Declares a **Note** containing details, comments, or design thoughts.
- `+` (Plus), `=` (Equals), `~` (Tilde), `#` (Number Sign), `<` (Less Than), and `>` (Greater Than): Reserved for future system extensions.

### Grammar
```text
- [Status] (Priority) Title :: [InceptionDate] [CompletionDate] Description +kindTag @locationTag keyTag=valueTag dep="Dependency Title"
```

- **Status**:
  - `[ ]` = Open
  - `[/]` = In Progress
  - `[x]` = Done
  - `[B]` = Blocked
  - `[D]` = Deferred
  - `[C]` = Cut / Dropped / Excluded
- **Priority** (Optional): A single uppercase letter, e.g., `(A)`.
- **Title**: The string preceding the `::` delimiter (excluding the status and priority brackets). Can contain spaces.
- **Delimiter**: `::` separates the core identity (title) from metadata and description.
- **Dates** (Optional): Immediate dates following `::` in `YYYY-MM-DD` format.
  - One date: Inception.
  - Two dates: Inception & Completion.
- **Tags**:
  - `+kind`: Semantic categorization (e.g., `+bug`, `+feature`, `+release`).
  - `@location`: Specific file/subsystem (e.g., `@lib.rs`, `@executor`).
  - `key=value`: Custom metadata (e.g., `est=2d`, `owner=xqhare`).
- **Dependencies**: Declared via `dep="Title"` (same project) or `dep="project_name:Title"` (cross-project). Multiple dependencies are allowed.

### Hierarchy & Subtask Dependencies
Hierarchy is established using indentation (4 spaces per level):
```markdown
- [ ] (A) Integrate CLI toolkit :: 2026-05-22 Integrate Eshu +feature @src/main.rs
    - [ ] Setup argument builder :: Write command definitions
    - [x] Parse subcommands :: Test parser against standard inputs
    * Remember to check for std::env::args_os compatibility
    * Make sure we don't pull in any external parser dependencies
- [B] Run Kahn Sort :: dep="Integrate CLI toolkit" +feature @src/graph.rs
```

**Subtask Dependency Rules**:
1. A parent task is implicitly dependent on all its child subtasks. A parent task cannot be marked completed (`[x]`) until all its child subtasks are completed.
2. In Kahn's topological sort, child subtasks are scheduled *before* their parent task.
3. Notes (`*`) are associated with the task directly above them at their respective indentation level and do not affect dependency ordering.

---

## 3. Nomos Spec & Parsing Engine

The `nomos` library is the parsing engine for Nomos files. It leverages the **Mawu** JSON library.

### Configurations
- **Global Config** (`~/.config/nomos/config.json`):
  ```json
  {
    "search_bases": [
      "~/Adytum/Programming/rust",
      "~/Adytum/Programming/python"
    ]
  }
  ```
- **Project Config** (`nomos.json` in a project root):
  ```json
  {
    "task_files": [
      "TODO.md",
      "docs/roadmap.md"
    ]
  }
  ```

### Discovery & Crawler
1. Crawls all directories under the global config's `search_bases`.
2. Identifies a folder as a project if it contains a `nomos.json` file.
3. If no `nomos.json` exists in a folder that has a standard project marker (like `Cargo.toml` or `.git`), it defaults to scanning `TODO.md` in that root and uses the directory name as the project identifier.

### Dependency Graph & Topological Sorting
1. Parses all task files from discovered projects.
2. Constructs a global Directed Acyclic Graph (DAG), resolving exact-match titles (e.g. `aequa:Task Name`).
3. Executes **Kahn's Algorithm** to sort tasks:
   - Identifies nodes with an in-degree of 0 (no unresolved dependencies).
   - Resolves subtasks before parent tasks (or vice-versa).
   - Checks for dependency cycles and outputs clear errors if cycles are found (e.g., `aequa:A -> bres:B -> aequa:A`).
4. Sorts tasks at the same dependency depth by their priority (`(A)` to `(Z)`).

### API

`Nomos` is a **public** library providing a **public API** for other projects to use Nomos files or structures.
It needs to serve as the base for the **Loki** TUI and any potential CLI toolkits.

---

