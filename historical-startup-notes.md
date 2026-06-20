# Nomos: Distributed Project Management System

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

# CLI Interactive Lens

Using *Eshu* create a cli program to quickly get and print all tasks that can be worked on right now, the "next" one to work on, along with sorting and the like

```bash
$ program_name list -a
$ program_name list --project==aequa
$ program_name next
$ program_name next --amount=2
```

## TUI

The TUI will be launched via a CLI subcommand.

Built with the **Talos** library, the TUI is the interactive dashboard to track roadmaps, tasks, issues, and cross-project blockers across a developer's workspace.

### Interface Design
- **Tree View**: Groups tasks by project (root node), parent task (branch), and subtask/note (leaf).
- **Navigation**: Vim bindings (`h/j/k/l`) to traverse the task hierarchy.
- **Actions**:
  - `Space`: Cycles task status (Open -> In Progress -> Done).
  - `Enter`: Displays task details (associated notes, tags, metadata).
  - `e`: Edits the task line inline.
  - `n`: Adds a new subtask or note line.

### In-Place Back-Propagation
To preserve raw text formatting and avoid file rewrite overhead:
1. During parsing, Nomos tracks the byte-offset (start and end byte positions) of every task line.
2. If the edit does not change the line length (e.g., toggling `[ ]` to `[x]`), the file is modified in-place by seeking to the start byte offset and rewriting the status bytes.
3. If the edit changes the line length, Loki reads up to the start offset, appends the modified line, appends the rest of the file from the end offset, and writes the entire stream back to disk.

---

# V0 - Initial Thoughts; Refined Above

As my ecosystem grows, I keep running into one main issue: How do I keep track of open todos / Roadmap items / bugs?

Atm they are somewhat centralised in the project specific readme, but that is not ideal nor really standardised.

BLOCKED UNTIL XFF V4 RELEASE (requires `XffValue::Graph`)

## Reqirements

- Seperation of concerns: Tasks, Roadmap, Bugs, etc.
- Distributed: Each project repo has its own tracking sub-system.
- Easy to use: A simple, human-readable format. Should be usable without bespoke tools.

Some features on a roadmap, bugs or tasks may be needed for other projects to move forward. This should be possible to track also.
Example: For `nabu` I need new `XffValue` types. These live in `aequa`. So I would love to be able to track these across projects.
Maybe just a simple `depends_on=aequa:NewValues`? Pushes a lot of the work on the users brain (What is the NewValues issue inside aequa now?)

### Bespoke Tooling

- Must be flexible: Find new projects, detect deletions of projects etc.
- Low friction: Not only a place to view tasks, but also to add new tasks, update tasks, etc.

## More names

### Anu
The ancient Celtic mother goddess (or the Sumerian sky god). It represents the supreme source or overarching sky.

### Mithra
An ancient Iranian deity (yazata) of covenants, light, oaths, justice, the Sun, contracts, and friendship. In addition to being the divinity of contracts, Mithra is also a judicial figure, an all-seeing protector of Truth (Asha), and the guardian of cattle, the harvest, and the Waters.

### Mimir
The Norse god/figure of knowledge and wisdom.

### Themis
In Greek mythology and religion, Themis is the goddess and personification of justice, divine order, law, and custom.

## Loki & Nomos

### Naming

#### Nomos

In Greek mythology, Nomos (Νόμος) was personified as the daemon (spirit) of laws and ordinances.

#### Loki

The Norse trickster god. Bugs, issues, and unexpected edge cases are classic "mischief."

Build a completely custom, distributed solution:

### 1. The Protocol: Nomos
A hierarchical, line-based task protocol that remains human-readable in Neovim but is structured enough for automated parsing.
- **Hierarchy:** Defined by indentation (whitespace sensitive).
- **Tasks:** Lines starting with status indicators: `[ ]` or `[x]`.
- **Notes:** Indented text belonging to the task above it.
    - 4 spaces per tab
- **Distributed:** Each project repo maintains its own `TODO.nomos` or `TODO.md`.

### 2. The Lens: Loki (TUI)
A high-performance TUI built using the **Talos** library.
- **View:** Aggregates all tasks across the ecosystem into a unified, searchable "Tree View."
- **Back-Propagation:** When a task is toggled in the TUI, it uses the stored byte offsets to write changes directly back to the source file in the project repo.
- **Zero-Sync Issue:** The database is a "build-once, read-mostly" cache; the truth always resides in the repo's text files.

### Why this works:
- **Zero Dependencies:** Uses internal crates (`nabu`, `aequa`, `talos`, `horae`).
- **Data Sovereignty:** Tasks move with the code. If a repo is deleted or moved, the central index simply updates.
- **Dogfooding:** Provides a perfect real-world use case for the upcoming Talos `Tree View` widget. `XffValue` would be used as in memory data storage anyways, maybe I can shoehorn `Nabu` (xff read&write to disk) into this somehow (Loki will need some kind of permanent config I am sure (Reason to implement Colour picker widget? For custom styling / highlighting maybe?) :D).

### Nomos

As I am already creating my of `todo.txt` inspired format, I will use that as a starting point:

```text
x (A) 2022-01-01 2025-12-31 Task text +projectTag @ContextTag Key=Value
```

Would be transformed into:

```markdown
[x] (A) 2022-01-01 2025-12-31 Task text +projectTag @ContextTag Key=Value
    Second line of the task
    [x] 2022-01-01 2025-12-30 Subtask
        [x] (B) 2022-01-01 2025-12-29 Subsubtask
[ ] ( ) 2022-01-01 Open Task
[] () 2022-01-01 Open Task also valid
```

## Xqhare-idiomatic

Good point raised by Gemini, what feels most "Xqhare-idiomatic"?
The answer is simple: Completely bespoke system built from scratch.

```markdown
[ ] TaskTitle0 :: 2026-05-16 Bump version +release
[ ] (A) TaskTitle1 :: 2026-05-16 Fix bug +bug @lib.rs Key=Value
    - Second line of task, starting with indentation + `-` sign for visual clarity
    - [x] (B) SubtaskTitle :: 2026-05-15 2026-05-16 Find bug +bug (again with indentation + `-`)
[ ] TaskTitle2 :: Ship new version +release dep=<repo:TaskTitle1, repo:TaskTitle0>
# Alternatively (omitted `repo:` prefix for same repo):
[ ] TaskTitle2 :: Ship new version +release dep=TaskTitle1 dep=TaskTitle0
```

Bare minimum for a valid task:

Status | Title | Text:

```markdown
[ ] TaskTitle :: Text
```

Maximum for a valid task:

Status | Priority | Title | Inception date | Completion date | Text:

```markdown
[ ] (A) TaskTitle :: 2026-05-16 2026-05-17 Text
```

### Tags

The three tag kinds from `todo.txt` would be translated to:

- `+projectTag` => `+kindTag` meaning the kind of thing this task is. (bugfix, feature, etc.)
- `@ContextTag` => `@locationTag` meaning the location of the task in the project. (lib.rs, src/main.rs, otherProjectName etc.)
- `Special:Tag` => `keyTag=valueTag` continues to be a key-value store, but using a `=` to separate key and value.

