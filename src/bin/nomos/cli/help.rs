const LINEBREAK: &str = "\n";
const DOUBLE_LINEBREAK: &str = "\n\n";
const PARAGRAPH: &str = "\n\n\n";

pub fn make_about() -> String {
    let mut string = String::with_capacity(512); // Half a kb should be more than enough

    make_welcome(&mut string);

    make_usage(&mut string);

    string.push_str(
        "Important: This software expects files following the syntax defined for Nomos files.",
    );
    string.push_str(LINEBREAK);
    string.push_str("Find out more here: ");
    string.push_str(env!("CARGO_PKG_REPOSITORY"));
    string.push_str(DOUBLE_LINEBREAK);

    string.push_str("Usage:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("On fist execution Nomos creates a `config.json` file inside `~/.config/nomos`, or as a fallback inside `~/.nomos`.");
    string.push_str(LINEBREAK);
    string.push_str("Update the `search_bases` key in the `config.json` file with paths pointing to the root directories containing the projects you want to track.");
    string.push_str(LINEBREAK);
    string.push_str("Example: If the path `~/projects/rust` is present, Nomos will crawl each subdirectory of it.");
    string.push_str(LINEBREAK);
    string.push_str("Nomos will look for a `nomos.json` inside the directory. If found it will read each file it finds held by the key `task_files`.");
    string.push_str(LINEBREAK);
    string.push_str("Should Nomos be unable to find `nomos.json`, and the directory contains either a `.git` directory or a `README.md` file, Nomos falls back to looking for [nomos, todo, tasks, roadmap] files with either a `.txt` or `.md` extension inside that directory.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str(
        "Inside the global `config.json` file, you can also define specific files to be tracked.",
    );
    string.push_str(LINEBREAK);
    string.push_str("Please note that each entry requires a project name and a single, valid, path pointing to the file.");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Using Nomos inside a project:");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("As described above, you can either provide a project specific configuration file with the name `nomos.json` in the project root, or rely on the fallback behaviour of the crawler.");
    string.push_str(LINEBREAK);
    string.push_str("Using the project configuarion file is heavily recommended. Example files are provided in the Nomos repository (found at: ");
    string.push_str(env!("CARGO_PKG_REPOSITORY"));
    string.push_str(" ).");
    string.push_str(LINEBREAK);
    string.push_str("Project configuration files must contain the key `task_files` that holds an array of strings.");
    string.push_str(LINEBREAK);
    string.push_str("Example:");
    string.push_str(DOUBLE_LINEBREAK);
    // Don't change the line below, the whitespace is important. Sure there are better ways,
    // but KISS and such
    string.push_str("{\n  \"task_files\": [\n    \"TODO.md\",\n    \"docs/roadmap.md\"\n  ]\n}");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("Nomos, the CLI program you are using, also supports a range of subcommands as well as two flags:");
    // rest is done by eshu
    string
}

fn make_welcome(string: &mut String) {
    string.push_str(
        "Nomos is a decentralized, text-based project management system built in Rust. It utilizes a human-readable markdown-compatible task protocol of the same name."
    );
    string.push_str(LINEBREAK);
    string.push_str("The goal of Nomos is to provide a simple, yet powerful tool for tracking tasks across projects.");
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
    make_usage_file(string);
    string.push_str(DOUBLE_LINEBREAK);
    make_usage_cli(string);
    string.push_str(PARAGRAPH);
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
    string.push_str(LINEBREAK);
    string.push_str("- [Status] (Priority) Title :: [InceptionDate] [CompletionDate] Description +kindTag @locationTag keyTag=valueTag dep=\"Dependency Title\"");
    string.push_str(DOUBLE_LINEBREAK);
    string.push_str("A note follows this syntax:");
    string.push_str(LINEBREAK);
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
}
