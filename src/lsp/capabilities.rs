//! When extracting this code from nomos, implement this as a Trait and update.
//! Should be easiest and quickest way to reuse code (`uri_to_path`) and maintain the custom logic (like `get_diagnostics`).
use std::collections::HashSet;
use std::path::Path;

use athena::Object;
use mawu::XffValue;

use crate::nomos::Nomos;
use crate::parser::parse_file;
use crate::task::Task;

/// Parse an LSP URI to a local file path string
pub fn uri_to_path(uri: &str) -> String {
    if let Some(path) = uri.strip_prefix("file://") {
        path.to_string()
    } else {
        uri.to_string()
    }
}

/// Generate LSP diagnostics for a Nomos file
pub fn get_diagnostics(uri: &str, content: &str) -> XffValue {
    let path_str = uri_to_path(uri);
    let path = Path::new(&path_str);

    // Deduce project name from the parent directory of the file URI
    let project = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    let mut diagnostics = Vec::new();

    // Parse the file content.
    // Since parse_file reads from disk, we can write the current buffer content to a temp file, or read it.
    // To be fast and accurate to the unsaved buffer, we can write the content to a temp file first.
    static DIAG_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let counter = DIAG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("nomos");
    let temp_path =
        std::env::temp_dir().join(format!("nomos_diag_{}_{}.{}", std::process::id(), counter, extension));
    if std::fs::write(&temp_path, content).is_ok() {
        if let Err(err) = parse_file(&temp_path, project) {
            // Extract line number
            let mut line_num = 0;
            for ctx in err.contexts() {
                if ctx.starts_with("Line: ") {
                    if let Some(pos) = ctx.find(" in file:") {
                        if let Ok(l) = ctx[6..pos].parse::<u32>() {
                            line_num = l.saturating_sub(1); // LSP lines are 0-indexed
                            break;
                        }
                    }
                }
            }

            let mut diag = Object::new();
            let mut range = Object::new();
            let mut start = Object::new();
            start.insert("line", XffValue::from(line_num as i64));
            start.insert("character", XffValue::from(0));
            let mut end = Object::new();
            end.insert("line", XffValue::from(line_num as i64));
            end.insert("character", XffValue::from(200)); // highlight entire line

            range.insert("start", XffValue::from(start));
            range.insert("end", XffValue::from(end));

            diag.insert("range", XffValue::from(range));
            diag.insert("severity", XffValue::from(1)); // Error
            diag.insert("message", XffValue::from(format!("{}", err.leaf_error())));
            diag.insert("source", XffValue::from("nomos-lsp".to_string()));
            diagnostics.push(XffValue::from(diag));
        }
        let _ = std::fs::remove_file(&temp_path);
    }

    let mut result = Object::new();
    result.insert("uri", XffValue::from(uri.to_string()));
    result.insert("diagnostics", XffValue::from(diagnostics));
    XffValue::from(result)
}

/// Generate LSP completions
pub fn get_completions(
    nomos: &Option<Nomos>,
    current_line: &str,
    character_pos: usize,
    current_project: &str,
) -> XffValue {
    let mut items = Vec::new();

    let before_cursor = if character_pos <= current_line.len() {
        &current_line[..character_pos]
    } else {
        current_line
    };

    // Split the before_cursor string by whitespace to get the last word
    let last_word = before_cursor
        .split(|c: char| c.is_whitespace())
        .last()
        .unwrap_or("");

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
    let mut generic_tags = HashSet::new();

    if let Some(n) = nomos {
        fn collect_tags_recursively(
            task: &Task,
            kind_tags: &mut HashSet<String>,
            location_tags: &mut HashSet<String>,
            metadata_keys: &mut HashSet<String>,
            generic_tags: &mut HashSet<String>,
        ) {
            for kind in &task.tags.kind_tags {
                kind_tags.insert(kind.clone());
            }
            for loc in &task.tags.location_tags {
                location_tags.insert(loc.clone());
            }
            for key in task.tags.metadata_tags.keys() {
                metadata_keys.insert(key.clone());
            }
            for tag in &task.tags.generic_tags {
                generic_tags.insert(tag.clone());
            }

            if let Some(notes) = &task.notes {
                for note in notes.iter() {
                    for kind in &note.tags.kind_tags {
                        kind_tags.insert(kind.clone());
                    }
                    for loc in &note.tags.location_tags {
                        location_tags.insert(loc.clone());
                    }
                    for key in note.tags.metadata_tags.keys() {
                        metadata_keys.insert(key.clone());
                    }
                    for tag in &note.tags.generic_tags {
                        generic_tags.insert(tag.clone());
                    }
                }
            }

            if let Some(sub_tasks) = &task.sub_tasks {
                for sub_task in sub_tasks.iter() {
                    collect_tags_recursively(sub_task, kind_tags, location_tags, metadata_keys, generic_tags);
                }
            }
        }

        for task in n.get_tasks().iter() {
            collect_tags_recursively(task, &mut kind_tags, &mut location_tags, &mut metadata_keys, &mut generic_tags);
        }
    }

    if last_word.starts_with('@') {
        // Location Tag completion
        for loc in location_tags {
            let mut item = Object::new();
            item.insert("label", XffValue::from(loc));
            item.insert("kind", XffValue::from(14)); // Keyword/Tag
            items.push(XffValue::from(item));
        }
    } else if last_word.starts_with('+') {
        // Kind Tag completion
        for kind in kind_tags {
            let mut item = Object::new();
            item.insert("label", XffValue::from(kind));
            item.insert("kind", XffValue::from(14)); // Keyword/Tag
            items.push(XffValue::from(item));
        }
    } else if last_word.starts_with('#') {
        // Generic Tag completion
        for tag in generic_tags {
            let mut item = Object::new();
            item.insert("label", XffValue::from(tag));
            item.insert("kind", XffValue::from(14)); // Keyword/Tag
            items.push(XffValue::from(item));
        }
    } else if last_word.starts_with("dep=") {
        // Dependency completion
        if let Some(n) = nomos {
            let dep_str = &last_word[4..];
            if dep_str.contains(':') {
                let parts: Vec<&str> = dep_str.split(':').collect();
                let target_project = parts[0].trim_matches('"').trim();
                for task in n.get_tasks().iter() {
                    if task.project == target_project {
                        let mut item = Object::new();
                        item.insert("label", XffValue::from(format!("\"{}\"", task.title)));
                        item.insert("kind", XffValue::from(18)); // Reference/Task
                        items.push(XffValue::from(item));
                    }
                }
            } else {
                let mut projects = HashSet::new();
                for task in n.get_tasks().iter() {
                    projects.insert(task.project.clone());
                    if task.project == current_project {
                        let mut item = Object::new();
                        item.insert("label", XffValue::from(format!("\"{}\"", task.title)));
                        item.insert("kind", XffValue::from(18)); // Reference/Task
                        items.push(XffValue::from(item));
                    }
                }
                for proj in projects {
                    if proj != current_project {
                        let mut item = Object::new();
                        item.insert("label", XffValue::from(format!("{}:", proj)));
                        item.insert("kind", XffValue::from(9)); // Module/Project
                        items.push(XffValue::from(item));
                    }
                }
            }
        }
    } else if last_word.contains('=') {
        // Metadata Value completion (e.g. key=value)
        let (key, _val) = last_word.split_once('=').unwrap();
        let mut values = HashSet::new();
        if let Some(n) = nomos {
            fn collect_metadata_values_recursively(
                task: &Task,
                target_key: &str,
                values: &mut HashSet<String>,
            ) {
                if let Some(v) = task.tags.metadata_tags.get(target_key) {
                    values.insert(v.clone());
                }
                if let Some(notes) = &task.notes {
                    for note in notes.iter() {
                        if let Some(v) = note.tags.metadata_tags.get(target_key) {
                            values.insert(v.clone());
                        }
                    }
                }
                if let Some(sub_tasks) = &task.sub_tasks {
                    for sub_task in sub_tasks.iter() {
                        collect_metadata_values_recursively(sub_task, target_key, values);
                    }
                }
            }
            for task in n.get_tasks().iter() {
                collect_metadata_values_recursively(task, key, &mut values);
            }
        }
        for v in values {
            let mut item = Object::new();
            item.insert("label", XffValue::from(v));
            item.insert("kind", XffValue::from(12)); // Value completion
            items.push(XffValue::from(item));
        }
    } else {
        // Metadata Key completion (suggesting key=)
        for key in metadata_keys {
            let mut item = Object::new();
            item.insert("label", XffValue::from(format!("{}=", key)));
            item.insert("kind", XffValue::from(14)); // Keyword/Tag
            items.push(XffValue::from(item));
        }
    }

    XffValue::from(items)
}

/// Generate Hover tooltips
pub fn get_hover(nomos: &Option<Nomos>, current_line: &str, character_pos: usize) -> XffValue {
    if let Some(n) = nomos {
        // Parse the line to see if we are hovering over a dependency
        let mut hovered_task: Option<&Task> = None;

        // Check for dep= format
        if let Some(pos) = current_line.find("dep=") {
            let dep_part = &current_line[pos..];
            // Find boundaries of the dependency value
            if let Some(val_pos) = dep_part.find('=') {
                let val_str = &dep_part[val_pos + 1..];
                let trimmed = val_str.trim().trim_matches('"');

                let (target_project, target_title) = if trimmed.contains(':') {
                    if let Some((p, t)) = trimmed.split_once(':') {
                        (Some(p.trim_matches('"').to_string()), t.trim_matches('"'))
                    } else {
                        (None, trimmed)
                    }
                } else {
                    (None, trimmed)
                };

                // Check if the cursor is near the dependency part
                let dep_start_char = pos;
                let dep_end_char = pos + dep_part.len();
                if character_pos >= dep_start_char && character_pos <= dep_end_char {
                    for task in n.get_tasks().iter() {
                        let matches_project = match &target_project {
                            Some(proj) => task.project == *proj,
                            None => true, // default to current project or match first
                        };
                        if matches_project && task.title == target_title {
                            hovered_task = Some(task);
                            break;
                        }
                    }
                }
            }
        }

        if let Some(task) = hovered_task {
            let mut contents = Object::new();
            contents.insert("kind", XffValue::from("markdown"));
            let md = format!(
                "### Task: {}\n- **Project**: {}\n- **Status**: {:?}\n- **Priority**: {:?}\n\n{}",
                task.title,
                task.project,
                task.status,
                task.priority,
                task.description.as_deref().unwrap_or("")
            );
            contents.insert("value", XffValue::from(md));

            let mut result = Object::new();
            result.insert("contents", XffValue::from(contents));
            return XffValue::from(result);
        }
    }

    XffValue::Null
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_uri_to_path() {
        assert_eq!(uri_to_path("file:///foo/bar.md"), "/foo/bar.md");
        assert_eq!(uri_to_path("/foo/bar.md"), "/foo/bar.md");
    }

    #[test]
    fn test_diagnostics_no_error() {
        let content = "- [ ] Clean room :: \n";
        let diag_val = get_diagnostics("file:///tmp/test.md", content);
        let obj = diag_val.as_object().unwrap();
        assert_eq!(
            obj.get("uri").unwrap().as_string().unwrap(),
            "file:///tmp/test.md"
        );
        let diags = obj.get("diagnostics").unwrap().as_array().unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_with_error() {
        let content = "- invalid\n";
        let diag_val = get_diagnostics("file:///tmp/test.md", content);
        let obj = diag_val.as_object().unwrap();
        let diags = obj.get("diagnostics").unwrap().as_array().unwrap();
        assert_eq!(diags.len(), 1);
        let diag = diags[0].as_object().unwrap();
        let range = diag.get("range").unwrap().as_object().unwrap();
        let start = range.get("start").unwrap().as_object().unwrap();

        let line_val = start.get("line");
        let line_num_opt = line_val.unwrap().as_number();
        let line_isize = line_num_opt.unwrap().into_isize();
        assert_eq!(line_isize.unwrap(), 0);

        let severity_val = diag.get("severity");
        let severity_num = severity_val.unwrap().as_number();
        let severity_isize = severity_num.unwrap().into_isize();
        assert_eq!(severity_isize.unwrap(), 1);
    }

    #[test]
    fn test_completions_and_hover() {
        // Create a temporary task file and config file to load Nomos
        let temp_dir = env::temp_dir().join(format!("nomos_test_workspace_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let task_file = temp_dir.join("tasks.md");
        fs::write(
            &task_file,
            "- [ ] Buy milk :: +shopping @store dep=\"my_proj:Clean kitchen\"\n- [ ] Clean kitchen :: \n"
        ).unwrap();

        let config_file = temp_dir.join("config.json");
        let config_content = format!(
            r#"{{"search_bases":[], "files":{{"my_proj":"{}"}}}}"#,
            task_file.to_str().unwrap().replace('\\', "/")
        );
        fs::write(&config_file, config_content).unwrap();

        let nomos_res = Nomos::new(&config_file);
        if let Err(ref e) = nomos_res {
            println!("DEBUG Nomos::new failed: {:?}", e);
        }
        let nomos = nomos_res.ok();
        assert!(nomos.is_some());

        // Test tag completions
        let comps_loc = get_completions(&nomos, "Buy milk @", 10, "my_proj");
        let items_loc = comps_loc.as_array().unwrap();
        assert_eq!(items_loc.len(), 1);
        assert_eq!(
            items_loc[0]
                .as_object()
                .unwrap()
                .get("label")
                .unwrap()
                .as_string()
                .unwrap(),
            "store"
        );

        let comps_kind = get_completions(&nomos, "Buy milk +", 10, "my_proj");
        let items_kind = comps_kind.as_array().unwrap();
        assert_eq!(items_kind.len(), 9);
        let labels: Vec<&str> = items_kind
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(labels.contains(&"shopping"));

        // Test dependency completions
        let comps_dep = get_completions(&nomos, "dep=", 4, "my_proj");
        let items_dep = comps_dep.as_array().unwrap();
        // Since project matches and other project is not set, we should get "Clean kitchen" and "Buy milk"
        assert!(items_dep.iter().any(|item| {
            item.as_object()
                .unwrap()
                .get("label")
                .unwrap()
                .as_string()
                .unwrap()
                == "\"Clean kitchen\""
        }));

        // Test hover
        let hover_val = get_hover(&nomos, "dep=\"my_proj:Clean kitchen\"", 12);
        assert!(hover_val.as_object().is_some());
        let hover_obj = hover_val.as_object().unwrap();
        let contents = hover_obj.get("contents").unwrap().as_object().unwrap();
        assert_eq!(
            contents.get("kind").unwrap().as_string().unwrap(),
            "markdown"
        );
        assert!(
            contents
                .get("value")
                .unwrap()
                .as_string()
                .unwrap()
                .contains("Task: Clean kitchen")
        );

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_recursive_completions() {
        let temp_dir = env::temp_dir().join(format!("nomos_test_recursive_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let task_file = temp_dir.join("tasks.md");
        fs::write(
            &task_file,
            "- [ ] Parent Task :: +parent_kind @parent_loc\n    - [ ] Subtask :: +sub_kind @sub_loc est=3d owner=xqhare\n"
        ).unwrap();

        let config_file = temp_dir.join("config.json");
        let config_content = format!(
            r#"{{"search_bases":[], "files":{{"my_proj":"{}"}}}}"#,
            task_file.to_str().unwrap().replace('\\', "/")
        );
        fs::write(&config_file, config_content).unwrap();

        let nomos = Nomos::new(&config_file).ok();
        assert!(nomos.is_some());

        // Test recursive kind tags (including subtask kind tag)
        let comps_kind = get_completions(&nomos, "+", 1, "my_proj");
        let items_kind = comps_kind.as_array().unwrap();
        let kind_labels: Vec<&str> = items_kind
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(kind_labels.contains(&"parent_kind"));
        assert!(kind_labels.contains(&"sub_kind"));
        assert!(kind_labels.contains(&"bug"));
        assert!(kind_labels.contains(&"refactor"));

        // Test recursive location tags (including subtask location tag)
        let comps_loc = get_completions(&nomos, "Buy @", 5, "my_proj");
        let items_loc = comps_loc.as_array().unwrap();
        let loc_labels: Vec<&str> = items_loc
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(loc_labels.contains(&"parent_loc"));
        assert!(loc_labels.contains(&"sub_loc"));

        // Test metadata key suggestions
        let comps_keys = get_completions(&nomos, "es", 2, "my_proj");
        let items_keys = comps_keys.as_array().unwrap();
        let key_labels: Vec<&str> = items_keys
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(key_labels.contains(&"est="));
        assert!(key_labels.contains(&"owner="));

        // Test metadata value suggestion when '=' is typed
        let comps_vals = get_completions(&nomos, "est=", 4, "my_proj");
        let items_vals = comps_vals.as_array().unwrap();
        let val_labels: Vec<&str> = items_vals
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(val_labels.contains(&"3d"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_diagnostics_preserves_extension() {
        let content = "- [ ] Optional Delimiter task\n";
        let diag_val = get_diagnostics("file:///tmp/test.nomos", content);
        let obj = diag_val.as_object().unwrap();
        let diags = obj.get("diagnostics").unwrap().as_array().unwrap();
        assert!(diags.is_empty(), "Expected no diagnostics, but got: {:?}", diags);
    }

    #[test]
    fn test_generic_tag_completions() {
        let temp_dir = env::temp_dir().join(format!("nomos_test_generic_comp_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let task_file = temp_dir.join("tasks.nomos");
        fs::write(
            &task_file,
            "- [ ] Tagged Task :: #important #todo\n    * Note with #refactor\n"
        ).unwrap();

        let config_file = temp_dir.join("config.json");
        let config_content = format!(
            r#"{{"search_bases":[], "files":{{"my_proj":"{}"}}}}"#,
            task_file.to_str().unwrap().replace('\\', "/")
        );
        fs::write(&config_file, config_content).unwrap();

        let nomos = Nomos::new(&config_file).ok();
        assert!(nomos.is_some());

        let comps = get_completions(&nomos, "Task #", 6, "my_proj");
        let items = comps.as_array().unwrap();
        let labels: Vec<&str> = items
            .iter()
            .map(|item| {
                item.as_object()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            })
            .collect();
        assert!(labels.contains(&"important"));
        assert!(labels.contains(&"todo"));
        assert!(labels.contains(&"refactor"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
