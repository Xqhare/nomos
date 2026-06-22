use eshu::{CliFlag, StoreSyntax, StoreType};

pub fn make_standard_flags() -> Vec<CliFlag> {
    vec![project_flag(), status_flag(), number_flag()]
}

fn status_flag() -> CliFlag {
    CliFlag::new("status")
        .with_required_store(StoreType::Value, StoreSyntax::Attached)
        .with_flag_char('s')
        .with_short_about("Shows all tasks with the passed in status.")
        .with_long_about(make_status_long_about())
        .build()
        .expect("Failed to build status flag - Can only happen during development while the flag was never built.")
}

fn make_status_long_about<'a>() -> &'a str {
    const OUT: &str = "Status requires an attached, single value. It may be one of:\n  - \"open\"\n  - \"in_progress\"\n  - \"done\"\n  - \"blocked\"\n  - \"deferred\"\n  - \"cut\"\n\nPlease note that the value is case insensitive, 's=oPeN' is valid for example.";
    OUT
}

fn number_flag() -> CliFlag {
    CliFlag::new("number").with_required_store(StoreType::Value, StoreSyntax::Detached).with_flag_char('n').with_short_about("Show a variable number of tasks").with_long_about(make_number_long_about()).build().expect("Failed to build number flag - Can only happen during development while the flag was never built.")
}

fn make_number_long_about<'a>() -> &'a str {
    const OUT: &str = "Number requires an attached, single value. It must be a positive integer.";
    OUT
}

fn project_flag() -> CliFlag {
    CliFlag::new("project")
        .with_required_store(StoreType::Value, StoreSyntax::Attached)
        .with_flag_char('p')
        .with_about(
            "Show tasks only belonging to the provided project.",
            make_project_long_about(),
        ).build().expect("Failed to build project flag - Can only happen during development while the flag was never built.")
}

fn make_project_long_about<'a>() -> &'a str {
    const OUT: &str = "Project requires an attached, single value. It must be a valid project name, case insensitive.";
    OUT
}
