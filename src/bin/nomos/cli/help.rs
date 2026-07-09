const LINEBREAK: &str = "\n";
const DOUBLE_LINEBREAK: &str = "\n\n";
const PARAGRAPH: &str = "\n\n\n";
const BREAK_INDENT: &str = "\n\t";
const BREAK_DOUBLE_INDENT: &str = "\n\t\t";
const DOUBLE_BREAK_INDENT: &str = "\n\n\t";

pub fn make_about() -> String {
    let mut string = String::with_capacity(512); // Half a kb should be more than enough

    make_welcome(&mut string);

    make_usage(&mut string);

    string
}

fn make_welcome(string: &mut String) {
    string.push_str(
        "Nomos is a decentralized, text-based project management system built in Rust. It utilizes a human-readable markdown-compatible task protocol of the same name."
    );
    string.push_str(LINEBREAK);
    string.push_str("The goal of Nomos is to provide a simple, yet powerful tool for tracking tasks across projects.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos design ensures that project specific data is kept directly with the code. It also supports inter- and intra-project linkage of tasks, while remaining tool-agnoistic.");
    string.push_str(LINEBREAK);
    string.push_str("In layman terms this means, that tasks can depend on another task, whether it resides in the same or a different project.");
    string.push_str(PARAGRAPH);
}

fn make_usage(string: &mut String) {
    string.push_str("Usage:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos is split into three distinct parts:\n\t- The CLI / Software\n\t- The Parser / Backend\n\t- The Syntax of the Markdown file");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("This help page has little information about the parser itself, please refer to the repository (at: ");
    string.push_str(env!("CARGO_PKG_REPOSITORY"));
    string.push_str(" ) for more information about it.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("You can find example files for all files talked about on this page in the Nomos repository (at: https://github.com/Xqhare/nomos/blob/master/examples )");
    make_usage_file(string);
    make_usage_cli(string);
    make_further_information(string);
}

fn make_usage_file(string: &mut String) {
    string.push_str("File Syntax:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Example of a Nomos file:");
    string.push_str(LINEBREAK);
    string.push_str("- [ ] (A) Integrate CLI toolkit :: 2026-05-22 Integrate Eshu +feature @src/main.rs\n\t- [ ] Setup argument builder :: Write command definitions\n\t- [x] Parse subcommands :: Test parser against standard inputs\n\t* Remember to check for std::env::args_os compatibility\n\t* Make sure we don't pull in any external parser dependencies\n- [B] Run Kahn Sort :: dep=\"Integrate CLI toolkit\" +feature @src/graph.rs");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos uses newline separated tasks and notes.");
    string.push_str(LINEBREAK);
    string.push_str("Tasks start with a hyphen, '-', and notes start with an asterisk, '*'.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("The syntax of a task is as follows:");
    string.push_str(BREAK_INDENT);
    string.push_str("- [Status] (Priority) Title :: [InceptionDate] [CompletionDate] Description +kindTag @locationTag keyTag=valueTag dep=\"Dependency Title\"");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("A note follows this syntax:");
    string.push_str(BREAK_INDENT);
    string.push_str("* Note Body +kindTag @locationTag keyTag=valueTag");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("A note may not define a dependency.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("You can find more information about tasks, notes and tags further down this help message, or inside the repository.");
    string.push_str(PARAGRAPH);
}

fn make_usage_cli(string: &mut String) {
    string.push_str("CLI Usage:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("On first execution Nomos creates a `config.json` file inside `~/.config/nomos`, or as a fallback inside `~/.nomos`.");
    string.push_str(LINEBREAK);
    string.push_str("Update the `search_bases` key in the `config.json` file with paths pointing to the root directories containing the projects you want to track.");
    string.push_str(LINEBREAK);
    string.push_str("Inside the `config.json` file, you can also update the `files` key with (\"project_name\": \"path/to/specific/file.md\") if you want to track specific files.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Example:");
    string.push_str(LINEBREAK);
    string.push_str(
        "If the path `~/projects/rust` is present, Nomos will crawl each subdirectory of it.",
    );
    string.push_str(LINEBREAK);
    string.push_str("Nomos will look for a `nomos.json` inside the directory. If found it will read each file it finds held by the key `task_files`.");
    string.push_str(LINEBREAK);
    string.push_str("If no `nomos.json` is found, Nomos will look for [nomos, todo, tasks, roadmap] files with either a `.txt` or `.md` extension inside that directory.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos will then parse each file and create a task for each task found.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("An example `nomos.json` file:");
    string.push_str(LINEBREAK);
    string.push_str("{\n\t\"task_files\": [\n\t\t\"TODO.md\",\n\t\t\"docs/roadmap.md\"\n  ]\n}");
    string.push_str(PARAGRAPH);
}

fn make_further_information(string: &mut String) {
    string.push_str("Further Information:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Tasks");
    string.push_str(BREAK_INDENT);
    string.push_str("The syntax of a task is:");
    string.push_str(BREAK_INDENT);
    string.push_str("- [Status] (Priority) Title :: [InceptionDate] [CompletionDate] Description +kindTag @locationTag keyTag=valueTag dep=\"Dependency Title\"");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("The status of a task can be one of the following:");
    string.push_str(BREAK_INDENT);
    string.push_str("- [ ] Open\n\t- [/] In Progress\n\t- [x] Completed\n\t- [B] Blocked\n\t- [C] Cut\n\t- [D] Deferred");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str(
        "The priority of a task can be A through Z (enclosed in brackets), or may be omitted.",
    );
    string.push_str(BREAK_INDENT);
    string.push_str("A is the highest priority, and Z is the lowest.");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("The title is all text after the status and priority, and before the double colon delimiter.");
    string.push_str(BREAK_INDENT);
    string.push_str(
        "The title must be unique within the project. It is used to define dependencies between tasks.",
    );
    string.push_str(BREAK_INDENT);
    string.push_str("It has no maximum length.");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("The delimiter is a double colon (with leading and trailing whitespaces: ' :: '). Please note that the delimiter is required.");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("Immediatly following the delimiter are the following optional date fields:");
    string.push_str(BREAK_DOUBLE_INDENT);
    string.push_str("- InceptionDate (YYYY-MM-DD)\n\t\t- CompletionDate (YYYY-MM-DD)");
    string.push_str(BREAK_INDENT);
    string.push_str("The InceptionDate is the date the task was created, and the CompletionDate is the date the task was completed. A CompletionDate may only be set if the status of a task is done ('[x]')");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("Everything after the two date fields is the description of the task.");
    string.push_str(BREAK_INDENT);
    string.push_str("The description also may contain tags and dependencies.");
    string.push_str(BREAK_INDENT);
    string.push_str("It has no maximum length.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Tags");
    string.push_str(BREAK_INDENT);
    string.push_str("Nomos supports three different types of tags:");
    string.push_str(BREAK_DOUBLE_INDENT);
    string.push_str("- +Kind\n\t\t- @Location\n\t\t- Key=Value");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("A `+KindTag` is defined with a leading plus '+'. It is used for Sematnic categorisation (like `+bug`, `+feature`) but may be any text without whitespace.");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("An `@LocationTag` is defined with a leading at symbol '@'. It is used to define a location of a task (like `@src/main.rs`)");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("A `Key=Value` is defined with a leading key and a value separated by an equal sign '='. It is used to define a key-value pair (like `keyTag=valueTag`) for custom metadata (e.g. `est=2d` or `owner=Xqhare`)");
    string.push_str(DOUBLE_BREAK_INDENT);
    string.push_str("Dependencies are special Key Value Pairs (like `dep=\"Dependency Title\"` or `dep=other_project:\"TaskTitle\"`).");
    string.push_str(BREAK_INDENT);
    string.push_str(
        "A dependency is a task that must be completed before this task can be completed.",
    );
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Notes");
    string.push_str(BREAK_INDENT);
    string.push_str("A note is defined with a leading asterisk '*'. It is similar to a taskDescription, but may not define a dependency.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Task Children");
    string.push_str(BREAK_INDENT);
    string.push_str("A task may have zero or more task children. A task child is a task immediatly following the parent task, indented by tabs to the right.");
    string.push_str(BREAK_INDENT);
    string.push_str("The indentation level of a task child is part of the syntax.");
    string.push_str(BREAK_INDENT);
    string.push_str("A parent task is implicitly dependent on all its child subtasks. A parent task cannot be marked completed (`[x]`) until all its child subtasks are completed.");
}
