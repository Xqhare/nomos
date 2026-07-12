use std::{path::{Path, PathBuf}, process::exit};

use eshu::{CliCommand, CliFlag};
use nomos::{prelude::*, version::FormatVersion};

use crate::write_err_and_exit;

/// CLI command to update Nomos task files and configuration from v0 to v1 format.
pub struct Update {
    global_config_file: PathBuf,
    flags: Vec<CliFlag>,
}

impl Update {
    /// Creates a new Update command instance.
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> Self {
        Update {
            global_config_file: global_config_file.into(),
            flags: vec![],
        }
    }
}

impl<'c> CliCommand<'c> for Update {
    fn name(&self) -> String {
        "update".to_string()
    }
    fn short_about(&self) -> String {
        "Migrate and update tasks to v1 format".to_string()
    }
    fn long_about(&self) -> String {
        "Migrate project task files from v0 format to v1 format, updating priorities and file extensions.".to_string()
    }
    fn flags(&self) -> &Vec<CliFlag> {
        &self.flags
    }
    fn subcommands(&self) -> Vec<std::rc::Rc<dyn CliCommand<'c>>> {
        vec![]
    }
    fn execute(&self, _cli: &eshu::Cli<'c>) {
        match run_update(&self.global_config_file) {
            Ok(_) => {
                println!("Nomos update pass successful!");
                exit(0);
            }
            Err(err) => write_err_and_exit(&format!("{err}")),
        }
    }
}

fn convert_priority(prio: char) -> char {
    match prio {
        'A' | 'B' | 'C' | 'a' | 'b' | 'c' => '1',
        'D' | 'E' | 'F' | 'd' | 'e' | 'f' => '2',
        'G' | 'H' | 'I' | 'g' | 'h' | 'i' => '3',
        'J' | 'K' | 'L' | 'j' | 'k' | 'l' => '4',
        'M' | 'N' | 'm' | 'n' => '5',
        'O' | 'P' | 'Q' | 'o' | 'p' | 'q' => '6',
        'R' | 'S' | 'T' | 'r' | 's' | 't' => '7',
        'U' | 'V' | 'W' | 'u' | 'v' | 'w' => '8',
        'X' | 'Y' | 'Z' | 'x' | 'y' | 'z' => '9',
        other => other,
    }
}

fn convert_tasks_priorities(tasks: &mut Tasks) {
    for task in tasks.iter_mut() {
        if let Some(prio) = task.priority {
            task.priority = Some(convert_priority(prio));
        }
        if let Some(sub_tasks) = &mut task.sub_tasks {
            convert_tasks_priorities(sub_tasks);
        }
    }
}

fn detect_version(file_path: &Path, global_config: &mawu::XffValue) -> FormatVersion {
    // 1. In-File Metadata Override (Optional)
    if let Ok(content) = std::fs::read_to_string(file_path) {
        if let Some(v) = FormatVersion::detect_from_file_content(&content) {
            return v;
        }
    }

    // 2. Project Configuration (nomos.json)
    if let Some(parent) = file_path.parent() {
        let nomos_json_path = parent.join("nomos.json");
        if nomos_json_path.exists() {
            if let Ok(file_val) = mawu::read::json(&nomos_json_path) {
                if let Some(obj) = file_val.as_object() {
                    if let Some(ver_val) = obj.get("version") {
                        if let Some(num) = ver_val.as_number() {
                            let val = num.into_usize().unwrap_or(0);
                            if val == 0 {
                                return FormatVersion::V0;
                            } else if val == 1 {
                                return FormatVersion::V1;
                            }
                        } else if let Some(s) = ver_val.as_string() {
                            if s == "0" {
                                return FormatVersion::V0;
                            } else if s == "1" {
                                return FormatVersion::V1;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Global Configuration (config.json)
    if let Some(obj) = global_config.as_object() {
        if let Some(ver_val) = obj.get("version") {
            if let Some(num) = ver_val.as_number() {
                let val = num.into_usize().unwrap_or(0);
                if val == 0 {
                    return FormatVersion::V0;
                } else if val == 1 {
                    return FormatVersion::V1;
                }
            } else if let Some(s) = ver_val.as_string() {
                if s == "0" {
                    return FormatVersion::V0;
                } else if s == "1" {
                    return FormatVersion::V1;
                }
            }
        }
    }

    // 4. Extension Inference
    if file_path.extension().map_or(false, |ext| ext == "md") {
        FormatVersion::V0
    } else {
        FormatVersion::V1
    }
}

fn run_update(global_config_file: &Path) -> Result<(), nemesis::NemesisError> {
    let mut global_config = mawu::read::json(global_config_file)?;
    let paths = nomos::nomos::utils::make_paths_to_crawl(&global_config)?;

    let mut all_migrated_files = Vec::new();
    let mut project_dirs = std::collections::HashSet::new();

    for (project_name, files) in &paths {
        for file_path in files {
            let version = detect_version(file_path, &global_config);
            if version == FormatVersion::V0 {
                // Parse file as v0 tasks
                let mut tasks = nomos::parser::parse_file(file_path, Some(project_name.clone()))?;

                // Convert priorities recursively
                convert_tasks_priorities(&mut tasks);

                // Write back tasks to .nomos file using Display trait
                let new_file_path = file_path.with_extension("nomos");
                let mut content = String::new();
                for task in tasks.iter() {
                    content.push_str(&task.to_string());
                    content.push('\n');
                }
                std::fs::write(&new_file_path, content)
                    .map_err(|e| nemesis::NemesisError::new("nomos::cli::update::run_update", e))?;

                // Delete old file
                std::fs::remove_file(file_path)
                    .map_err(|e| nemesis::NemesisError::new("nomos::cli::update::run_update", e))?;

                println!("Migrated task file: {:?} -> {:?}", file_path, new_file_path);
                all_migrated_files.push((file_path.clone(), new_file_path));

                if let Some(parent) = file_path.parent() {
                    project_dirs.insert(parent.to_path_buf());
                }
            }
        }
    }

    // Collect more project dirs from search bases
    if let Some(obj) = global_config.as_object() {
        if let Some(bases_val) = obj.get("search_bases") {
            if let Some(arr) = bases_val.as_array() {
                for v in arr.iter() {
                    if let Some(s) = v.as_string() {
                        let base_path = PathBuf::from(s);
                        if base_path.exists() {
                            if let Ok(entries) = base_path.read_dir() {
                                for entry in entries {
                                    if let Ok(entry) = entry {
                                        let dir = entry.path();
                                        if dir.is_dir() {
                                            project_dirs.insert(dir);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Update nomos.json files
    for dir in project_dirs {
        let nomos_json_path = dir.join("nomos.json");
        if nomos_json_path.exists() {
            let mut nomos_json_val = mawu::read::json(&nomos_json_path)?;
            if let Some(obj) = nomos_json_val.as_object_mut() {
                obj.insert("version".to_string(), athena::XffValue::Number(athena::Number::from(1)));

                if let Some(task_files) = obj.get_mut("task_files") {
                    if let Some(arr) = task_files.as_array_mut() {
                        for i in 0..arr.len() {
                            if let Some(file_str_val) = arr.get_mut(i) {
                                if let Some(file_str) = file_str_val.as_string() {
                                    let abs_path = dir.join(file_str);
                                    for (old_p, new_p) in &all_migrated_files {
                                        if &abs_path == old_p {
                                            if let Ok(rel_path) = new_p.strip_prefix(&dir) {
                                                *file_str_val = athena::XffValue::from(rel_path.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                mawu::write(&nomos_json_path, nomos_json_val)?;
                println!("Updated project configuration version in {:?}", nomos_json_path);
            }
        }
    }

    // Update global config config.json files list if it lists any migrated files
    let mut global_config_modified = false;
    if let Some(obj) = global_config.as_object_mut() {
        if let Some(files_val) = obj.get_mut("files") {
            if let Some(files_obj) = files_val.as_object_mut() {
                for (_proj_name, file_val) in files_obj.iter_mut() {
                    if let Some(file_str) = file_val.as_string() {
                        let old_path = PathBuf::from(file_str);
                        for (old_p, new_p) in &all_migrated_files {
                            if &old_path == old_p {
                                *file_val = athena::XffValue::from(new_p.to_string_lossy().to_string());
                                global_config_modified = true;
                            }
                        }
                    }
                }
            }
        }
    }
    if global_config_modified {
        mawu::write(global_config_file, global_config)?;
        println!("Updated global config files list.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_conversion() {
        assert_eq!(convert_priority('A'), '1');
        assert_eq!(convert_priority('B'), '1');
        assert_eq!(convert_priority('C'), '1');
        assert_eq!(convert_priority('D'), '2');
        assert_eq!(convert_priority('E'), '2');
        assert_eq!(convert_priority('F'), '2');
        assert_eq!(convert_priority('G'), '3');
        assert_eq!(convert_priority('H'), '3');
        assert_eq!(convert_priority('I'), '3');
        assert_eq!(convert_priority('J'), '4');
        assert_eq!(convert_priority('K'), '4');
        assert_eq!(convert_priority('L'), '4');
        assert_eq!(convert_priority('M'), '5');
        assert_eq!(convert_priority('N'), '5');
        assert_eq!(convert_priority('O'), '6');
        assert_eq!(convert_priority('P'), '6');
        assert_eq!(convert_priority('Q'), '6');
        assert_eq!(convert_priority('R'), '7');
        assert_eq!(convert_priority('S'), '7');
        assert_eq!(convert_priority('T'), '7');
        assert_eq!(convert_priority('U'), '8');
        assert_eq!(convert_priority('V'), '8');
        assert_eq!(convert_priority('W'), '8');
        assert_eq!(convert_priority('X'), '9');
        assert_eq!(convert_priority('Y'), '9');
        assert_eq!(convert_priority('Z'), '9');
        
        // Lowercase
        assert_eq!(convert_priority('a'), '1');
        assert_eq!(convert_priority('z'), '9');

        // Number/other character
        assert_eq!(convert_priority('1'), '1');
        assert_eq!(convert_priority('0'), '0');
        assert_eq!(convert_priority('?'), '?');
    }
}

