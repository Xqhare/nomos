use std::{collections::BTreeMap, path::PathBuf};

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

/// Expects `all_project_tasks` to be `String = project_name, Vec<Task> = tasks`
pub fn sort_tasks(all_project_tasks: BTreeMap<String, Tasks>) -> NomosResult<Tasks> {
    // Kahn's algorithm, modified for weighted tasks. The lower the weight, the higher the
    // priority. 0 is the highest, 255 is the lowest
    // Why: As prio is a valid 7bit ASCII char and A (highest) - Z (lowest) ordering is desired
    let kahn_input = {
        // Only assumes 4 tasks per project on average; should be fine but will lead
        // to some allocations
        let mut kahn_input: Vec<(String, u8, Vec<String>)> =
            Vec::with_capacity(all_project_tasks.len().saturating_mul(4));
        for (project_name, tasks) in all_project_tasks.iter() {
            for task in tasks.iter() {
                let dep_iter = task.dependencies.iter();
                let mut dependencies: Vec<String> = Vec::with_capacity(dep_iter.size_hint().0);
                for dependency in dep_iter {
                    let complete_name = format!("{project_name}:{}", dependency.title);
                    dependencies.push(complete_name);
                }
                let task_name = format!("{project_name}:{}", task.title);
                let prio: u8 = if let Some(prio) = task.priority {
                    prio as u8
                } else {
                    255
                };
                kahn_input.push((task_name, prio, dependencies));
            }
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
        let (project_name, task_name) = task_name.split_once(':').unwrap();
        if let Some(tasks) = all_project_tasks.get(project_name) {
            for task in tasks.iter() {
                if task.title == task_name {
                    sorted_tasks.push(task.clone());
                }
            }
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
        make_project_paths(&base_path, &mut out)?;
    }
    out.shrink_to_fit();
    Ok(out)
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
                "Invalid global config: Search base {:?} does not contain a project name",
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
                                            .eq(&path)
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
            if tmp.is_empty() {
                return Err(NemesisError::new(
                    "nomos::make_paths_to_crawl::make_project_paths",
                    NomosError::Config(format!(
                        "Invalid global config: Search base {:?} does not contain a nomos.json file",
                        base_path
                    )),
                ));
            }
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
