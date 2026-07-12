use std::{collections::{BTreeMap, HashMap}, path::PathBuf};


use athena::{Array, sorting::kahns_weighted};
use mawu::{XffValue, read::json};
use nemesis::{NemesisError, NemesisResultExt};

use crate::{
    error::{NomosError, NomosResult},
    parser::parse_file,
    task::{Task, Tasks},
};

/// Expects `paths_to_crawl` to be `Vec<(project_name, Vec<PathBuf>)>`
pub fn parse_tasks(
    paths_to_crawl: Vec<(String, Vec<PathBuf>)>,
) -> NomosResult<BTreeMap<String, Tasks>> {
    let mut all_project_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
    for (project_name, paths) in paths_to_crawl {
        for path in paths {
            if let Some(tasks) = all_project_tasks.get_mut(&project_name) {
                tasks.extend(parse_file(&path, Some(project_name.clone()))?);
            } else {
                all_project_tasks.insert(
                    project_name.clone(),
                    parse_file(&path, Some(project_name.clone()))?,
                );
            }
        }
    }
    Ok(all_project_tasks)
}

fn collect_recursive(
    task: &Task,
    project_name: &str,
    parent_titles: &mut Vec<String>,
    flat_tasks: &mut HashMap<String, Task>,
    child_ids_map: &mut HashMap<String, Vec<String>>,
) {
    let current_id = if parent_titles.is_empty() {
        format!("{project_name}:{}", task.title)
    } else {
        format!("{project_name}:{}:{}", parent_titles.join(":"), task.title)
    };

    flat_tasks.insert(current_id.clone(), task.clone());

    if let Some(sub_tasks) = &task.sub_tasks {
        let mut child_ids = Vec::new();
        for sub_task in sub_tasks.iter() {
            parent_titles.push(task.title.clone());
            let sub_id = format!("{project_name}:{}:{}", parent_titles.join(":"), sub_task.title);
            child_ids.push(sub_id);
            collect_recursive(sub_task, project_name, parent_titles, flat_tasks, child_ids_map);
            parent_titles.pop();
        }
        child_ids_map.insert(current_id, child_ids);
    }
}

/// Expects `all_project_tasks` to be `String = project_name, Vec<Task> = tasks`
pub fn sort_tasks(all_project_tasks: BTreeMap<String, Tasks>) -> NomosResult<Tasks> {
    let mut flat_tasks = HashMap::new();
    let mut child_ids_map = HashMap::new();

    for (project_name, tasks) in all_project_tasks.iter() {
        for task in tasks.iter() {
            let mut parent_titles = Vec::new();
            collect_recursive(
                task,
                project_name,
                &mut parent_titles,
                &mut flat_tasks,
                &mut child_ids_map,
            );
        }
    }

    // Kahn's algorithm, modified for weighted tasks. The lower the weight, the higher the
    // priority. 0 is the highest, 255 is the lowest
    // Why: As prio is a valid 7bit ASCII char and A (highest) - Z (lowest) ordering is desired
    let kahn_input = {
        let mut kahn_input: Vec<(String, u8, Vec<String>)> = Vec::with_capacity(flat_tasks.len());
        for (current_id, task) in flat_tasks.iter() {
            let dep_iter = task.dependencies.iter();
            let mut dependencies: Vec<String> = Vec::with_capacity(dep_iter.size_hint().0);

            for dependency in dep_iter {
                let dep_project = dependency.project.as_ref().unwrap_or(&task.project);
                let matched_id = flat_tasks.keys().find(|key| {
                    if let Some((first, _)) = key.split_once(':') {
                        if first == dep_project {
                            if let Some(last) = key.rsplit(':').next() {
                                if last == dependency.title {
                                    return true;
                                }
                            }
                        }
                    }
                    false
                });
                let dep_id = if let Some(matched) = matched_id {
                    matched.clone()
                } else {
                    format!("{dep_project}:{}", dependency.title)
                };
                dependencies.push(dep_id);
            }

            if let Some(child_ids) = child_ids_map.get(current_id) {
                for child_id in child_ids {
                    dependencies.push(child_id.clone());
                }
            }

            let prio: u8 = if let Some(prio) = task.priority {
                prio as u8
            } else {
                255
            };
            kahn_input.push((current_id.clone(), prio, dependencies));
        }
        kahn_input
    };

    let kahn_in: Vec<(&str, u8, Vec<&str>)> = kahn_input
        .iter()
        .map(|(a, u, b)| (a.as_str(), *u, b.iter().map(|s| s.as_str()).collect()))
        .collect();

    let sorted = match kahns_weighted(&kahn_in) {
        Ok(v) => v,
        Err(e) => {
            return Err(NemesisError::new("nomos::Nomos::new", e)
                .add_ctx("Sorting using Kahn's algorythm failed"));
        }
    };

    let mut sorted_tasks: Vec<Task> = Vec::with_capacity(sorted.len());
    for task_name in sorted {
        if let Some(task) = flat_tasks.get(task_name) {
            sorted_tasks.push(task.clone());
        }
    }
    Ok(sorted_tasks.into())
}


/// Expects `config` to be `XffValue::Object`
pub fn make_paths_to_crawl(config: &XffValue) -> NomosResult<Vec<(String, Vec<PathBuf>)>> {
    let search_base = make_search_base(config)?;
    let mut out: Vec<(String, Vec<PathBuf>)> =
        Vec::with_capacity(search_base.len().saturating_mul(2)); // Should overallocate by quite a margin
    for base in search_base.iter() {
        let base_path: PathBuf = base.to_string().into();
        if !base_path.exists() {
            return Err(NemesisError::new(
                "nomos::make_paths_to_crawl",
                NomosError::Config(format!(
                    "Invalid global config: Search base {base} does not exist"
                )),
            )
            .add_ctx(format!("Got global config: {config}")));
        }
        for dir in base_path
            .read_dir()
            .map_err(|e| NemesisError::new("nomos::make_paths_to_crawl", e))?
        {
            let dir: PathBuf = dir
                .map_err(|e| NemesisError::new("nomos::make_paths_to_crawl", e))?
                .path();
            if dir.is_dir() {
                make_project_paths(&dir, &mut out)?;
            }
        }
    }
    make_file_paths(config, &mut out)?;
    out.shrink_to_fit();
    Ok(out)
}

fn make_file_paths(config: &XffValue, out: &mut Vec<(String, Vec<PathBuf>)>) -> NomosResult<()> {
    if let Some(obj) = config.as_object()
        && let Some(files) = obj.get("files")
        && let Some(inner_files) = files.as_object()
    {
        if !inner_files.is_empty() {
            'file_loop: for (project_name, file_path) in inner_files.iter() {
                let file_path: PathBuf = file_path.to_string().into();
                if let Some(entry) = out.iter_mut().find(|(s, _)| s == project_name) {
                    for path in &entry.1 {
                        if path == &file_path {
                            continue 'file_loop;
                        }
                    }
                    if file_path.exists() {
                        entry.1.push(file_path);
                    } else {
                        return Err(NemesisError::new(
                            "nomos::make_paths_to_crawl::make_file_paths",
                            NomosError::Config(format!(
                                "Invalid global config files path: {:?} does not exist.",
                                file_path
                            )),
                        ));
                    }
                } else {
                    if file_path.exists() {
                        out.push((project_name.clone(), vec![file_path]));
                    } else {
                        return Err(NemesisError::new(
                            "nomos::make_paths_to_crawl::make_file_paths",
                            NomosError::Config(format!(
                                "Invalid global config files path: {:?} does not exist.",
                                file_path
                            )),
                        ));
                    }
                }
            }
        }
        Ok(())
    } else {
        return Err(NemesisError::new(
            "nomos::make_paths_to_crawl::make_file_paths",
            NomosError::Config(format!(
                "Invalid global config. config not an object, or doesnt contain the key `files` holding another object value.",
            )),
        ));
    }
}

fn make_project_paths(
    base_path: &PathBuf,
    out: &mut Vec<(String, Vec<PathBuf>)>,
) -> NomosResult<()> {
    let mut tmp: Vec<PathBuf> = Vec::new();
    let project_name = if let Some(name) = base_path.file_name() {
        name.to_string_lossy().to_string()
    } else {
        return Err(NemesisError::new(
            "nomos::make_paths_to_crawl::make_project_paths",
            NomosError::Config(format!(
                "Invalid global config: Search base path {:?} does not contain a project name",
                base_path
            )),
        ));
    };
    if let Ok(file) = json(base_path.join("nomos.json")) {
        if let Some(obj) = file.as_object()
            && let Some(task_files) = obj.get("task_files")
            && let Some(file_arr) = task_files.as_array()
        {
            if file_arr.iter().find(|v| !v.is_string()).is_some() {
                return Err(NemesisError::new(
                    "nomos::make_paths_to_crawl::make_project_paths",
                    NomosError::Config(format!(
                        "Invalid global config: Search base {:?} contains invalid task files",
                        base_path
                    )),
                ));
            }
            for file in file_arr.iter() {
                let potential_path = base_path.join(file.to_string());
                if potential_path.exists() {
                    tmp.push(potential_path);
                } else {
                    return Err(NemesisError::new(
                        "nomos::make_paths_to_crawl::make_project_paths",
                        NomosError::Config(format!(
                            "Invalid global config: Search base {:?} contains invalid task file inside project: {project_name}. File does not exist: {:?}",
                            base_path, potential_path
                        )),
                    ));
                }
            }
        }
    } else {
        // TODO: add to doc: Only dirs with a standard project marker are considered
        if base_path.join(".git").exists() || base_path.join("README.md").exists() {
            for file in ["nomos", "todo", "tasks", "roadmap"] {
                for extension in ["txt", "md"] {
                    let path = PathBuf::from(base_path.join(format!("{file}.{extension}")));
                    match base_path.read_dir() {
                        Err(err) => {
                            return Err(NemesisError::new(
                                "nomos::make_paths_to_crawl::make_project_paths",
                                err,
                            )
                            .add_ctx(format!("Cannot read project directory: {base_path:?}")));
                        }
                        Ok(inner_files) => {
                            for inner_file in inner_files {
                                match inner_file {
                                    Err(_) => (),
                                    Ok(inner_file) => {
                                        if inner_file
                                            .path()
                                            .to_string_lossy()
                                            .to_lowercase()
                                            .eq(&path.to_string_lossy().to_lowercase())
                                        {
                                            tmp.push(inner_file.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Dont do shit if tmp is empty; it's not a project using nomos
        }
    }
    out.push((project_name, tmp));
    Ok(())
}

/// Validates the global config and returns the search bases
fn make_search_base(config: &XffValue) -> NomosResult<Array> {
    if let Some(obj) = config.as_object()
        && let Some(bases) = obj.get("search_bases")
    {
        if let Some(arr) = bases.as_array() {
            if arr.iter().find(|v| !v.is_string()).is_some() {
                return Err(NemesisError::new(
                        "nomos::make_paths_to_crawl",
                        NomosError::Config(
                            "Invalid global config: Key 'search_bases' exists, but the value is not an array of strings"
                            .to_string(),
                        ),
                    )
                    .add_ctx(format!("Got global config: {config}")));
            }
            Ok(arr.clone())
        } else {
            Err(NemesisError::new(
                    "nomos::make_paths_to_crawl",
                    NomosError::Config(
                        "Invalid global config: Key 'search_bases' exists, but the value is not an array".to_string(),
                    ),
                )
                .add_ctx(format!("Got global config: {config}")))
        }
    } else {
        return Err(NemesisError::new(
            "nomos::make_paths_to_crawl",
            NomosError::Config("Nomos failed to find search_bases in global config".to_string()),
        )
        .add_ctx(
            "Valid global config file was found, but it is missing the key: 'search_bases'"
                .to_string(),
        ))
        .add_ctx(format!("Global config: {config}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, Tasks};
    use crate::utils::{Dependencies, Dependency, FileData, TaskStatus};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn test_cross_project_dependency_sorting() {
        let mut tasks_mawu = Tasks::new();
        let mut dep_mawu = Dependencies::new();
        dep_mawu.add(Dependency {
            title: "Add emoji support".to_string(),
            project: Some("thoth".to_string()),
        });

        tasks_mawu.push(Task {
            parents_amount: 0,
            status: TaskStatus::Open,
            priority: None,
            title: "Toml Emoji support".to_string(),
            inception_date: None,
            completion_date: None,
            tags: crate::tags::Tags::new(),
            dependencies: dep_mawu,
            description: None,
            notes: None,
            sub_tasks: None,
            file_data: FileData {
                line: 1,
                path: PathBuf::from("mawu.md"),
            },
            project: "mawu".to_string(),
        });

        let mut tasks_thoth = Tasks::new();
        tasks_thoth.push(Task {
            parents_amount: 0,
            status: TaskStatus::Open,
            priority: None,
            title: "Add emoji support".to_string(),
            inception_date: None,
            completion_date: None,
            tags: crate::tags::Tags::new(),
            dependencies: Dependencies::new(),
            description: None,
            notes: None,
            sub_tasks: None,
            file_data: FileData {
                line: 1,
                path: PathBuf::from("thoth.md"),
            },
            project: "thoth".to_string(),
        });

        let mut all_project_tasks = BTreeMap::new();
        all_project_tasks.insert("mawu".to_string(), tasks_mawu);
        all_project_tasks.insert("thoth".to_string(), tasks_thoth);

        let sorted = sort_tasks(all_project_tasks).unwrap();
        let sorted_list: Vec<&Task> = sorted.iter().collect();

        assert_eq!(sorted_list.len(), 2);
        // thoth task must be scheduled before mawu task
        assert_eq!(sorted_list[0].project, "thoth");
        assert_eq!(sorted_list[0].title, "Add emoji support");
        assert_eq!(sorted_list[1].project, "mawu");
        assert_eq!(sorted_list[1].title, "Toml Emoji support");
    }

    #[test]
    fn test_subtask_sorting_and_dependencies() {
        let mut tasks = Tasks::new();
        let mut subtasks = Tasks::new();
        subtasks.push(Task {
            parents_amount: 1,
            status: TaskStatus::Open,
            priority: None,
            title: "Subtask A".to_string(),
            inception_date: None,
            completion_date: None,
            tags: crate::tags::Tags::new(),
            dependencies: Dependencies::new(),
            description: None,
            notes: None,
            sub_tasks: None,
            file_data: FileData { line: 2, path: PathBuf::from("test.nomos") },
            project: "proj".to_string(),
        });
        tasks.push(Task {
            parents_amount: 0,
            status: TaskStatus::Open,
            priority: None,
            title: "Parent P".to_string(),
            inception_date: None,
            completion_date: None,
            tags: crate::tags::Tags::new(),
            dependencies: Dependencies::new(),
            description: None,
            notes: None,
            sub_tasks: Some(subtasks),
            file_data: FileData { line: 1, path: PathBuf::from("test.nomos") },
            project: "proj".to_string(),
        });

        let mut all_project_tasks = BTreeMap::new();
        all_project_tasks.insert("proj".to_string(), tasks);

        let sorted = sort_tasks(all_project_tasks).unwrap();
        let sorted_list: Vec<&Task> = sorted.iter().collect();

        // Subtask A must sort before Parent P because the parent depends on the subtask
        assert_eq!(sorted_list[0].title, "Subtask A");
        assert_eq!(sorted_list[1].title, "Parent P");
    }
}
