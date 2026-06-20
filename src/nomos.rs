use std::{collections::BTreeMap, path::PathBuf};

use athena::{Array, sorting::kahns_weighted};
use mawu::{XffValue, read::json};
use nemesis::{NemesisError, NemesisResultExt};

use crate::{
    error::{NomosError, NomosResult},
    parser::parse_file,
    task::{Task, Tasks},
    utils::{TaskStatus, calc_line_size, shift_task_lines_by_offset},
};

pub struct Nomos {
    pub tasks: Tasks,
}

impl Nomos {
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> NomosResult<Self> {
        let global_config_file = global_config_file.into();
        let config = json(&global_config_file).add_ctx(format!(
            "Nomos failed to load global config from {:?}",
            &global_config_file
        ))?;

        let tasks: Tasks = sort_tasks(parse_tasks(make_paths_to_crawl(&config)?)?)?;

        Ok(Nomos { tasks })
    }
    ///
    /// # Notes
    ///
    /// Assumes that the passed in task has the correct lines for itself, its notes and subtasks.
    ///
    /// Please note, the passed in task _must_ have the same `file_data` as the task to be updated.
    /// Should you want to update the `line_number` held by `file_data`, please remove the
    /// task and add it again.
    pub fn update_task(&mut self, updated_task: Task) -> NomosResult<()> {
        let mut offset = None;
        let starting_line = updated_task.file_data.line;
        for task in self.tasks.iter_mut() {
            if task.file_data == updated_task.file_data {
                let task_ls = calc_line_size(&task) as i64;
                let updated_task_ls = calc_line_size(&updated_task) as i64;
                if task_ls != updated_task_ls {
                    // Negative offset: list shrunk, positive offset: list grew
                    // # Note: Cast of i64 only to ensure all u32 values can be represented
                    offset = Some(updated_task_ls.saturating_sub(task_ls));
                } else {
                    offset = Some(0);
                }
                *task = updated_task.clone();
                break;
            }
        }
        if let Some(offset) = offset {
            shift_task_lines_by_offset(&mut self.tasks, offset, starting_line)
        } else {
            Err(NemesisError::new(
                "nomos::Nomos::update_task",
                NomosError::Task(
                    "Task to be updated must have the same `file_data` as the updated task"
                        .to_string(),
                ),
            )
            .add_ctx(format!(
                "Tried to update task: {}:{} in file: {:?}",
                &updated_task.project, &updated_task.title, &updated_task.file_data.path
            )))
        }
    }
    /// OPEN TODOs! does not update the file on disk atm
    /// # Notes
    ///
    /// Please note, the passed in task _must_ have the same `file_data` as the task to be updated.
    pub fn remove_task(&mut self, task: &Task) -> NomosResult<()> {
        for t in self.tasks.iter() {
            if t.file_data == task.file_data {
                todo!(())
            }
        }
        Ok(())
    }
    /// OPEN TODOs! does not update the file on disk atm
    /// # Notes
    ///
    /// Assumes that the passed in task has the correct lines for itself, its notes and subtasks.
    pub fn add_task(&mut self, task: Task) -> NomosResult<()> {
        self.tasks.push(task);
        // TODO: Check if task title is unique, and if so, add it to the list.
        // Check if line number is in use, if so, insert there and move all other tasks
        Ok(())
    }
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.status == status {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            } else {
                for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                    if sub_task.status == status {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(sub_task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![sub_task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    pub fn get_tasks_by_priority(&self, priority: char) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.priority == Some(priority) {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            } else {
                for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                    if sub_task.priority == Some(priority) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(sub_task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![sub_task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    /// While task titles must be unique inside one project, they can be shared between projects. This function returns all tasks of any project matching the passed in string.
    pub fn get_tasks_by_title(&self, title: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.title == title {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            } else {
                for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                    if sub_task.title == title {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(sub_task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![sub_task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    pub fn get_tasks_by_project(&self, project: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.project == project {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
            // All subtasks will be of the same project, so we don't need to check them
        }
        sort_tasks(all_tasks)
    }
    /// Expects a `Vec<String>` with each element representing a tag. Each element must have
    /// the leading `#`, `@`, or `+`
    pub fn get_tasks_by_tags(&self, tags: Vec<String>) -> NomosResult<Tasks> {
        let (kind_tags, location_tags, generic_tags) = {
            let mut kind_tags: Vec<String> = Vec::new();
            let mut location_tags: Vec<String> = Vec::new();
            let mut generic_tags: Vec<String> = Vec::new();
            for tag in tags {
                if tag.starts_with('#') {
                    generic_tags.push(tag);
                } else if tag.starts_with('+') {
                    kind_tags.push(tag);
                } else if tag.starts_with('@') {
                    location_tags.push(tag);
                }
            }
            (kind_tags, location_tags, generic_tags)
        };
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for tag in kind_tags {
            let tasks = self.get_tasks_by_kind(&tag[1..])?;
            for task in tasks.iter() {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
        }
        for tag in location_tags {
            let tasks = self.get_tasks_by_location(&tag[1..])?;
            for task in tasks.iter() {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
        }
        for tag in generic_tags {
            let tasks = self.get_tasks_by_generic_tag(&tag[1..])?;
            for task in tasks.iter() {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
        }
        sort_tasks(all_tasks)
    }
    pub fn get_tasks_by_kind(&self, kind: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.tags.contains_kind(kind) {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
            if let Some(sub_tasks) = task.sub_tasks.as_ref() {
                for sub_task in sub_tasks.iter() {
                    if sub_task.tags.contains_kind(kind) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
            if let Some(notes) = task.notes.as_ref() {
                for note in &notes.notes {
                    if note.tags.contains_kind(kind) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    pub fn get_tasks_by_location(&self, location: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.tags.contains_location(location) {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
            if let Some(sub_tasks) = task.sub_tasks.as_ref() {
                for sub_task in sub_tasks.iter() {
                    if sub_task.tags.contains_location(location) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
            if let Some(notes) = task.notes.as_ref() {
                for note in &notes.notes {
                    if note.tags.contains_location(location) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    pub fn get_tasks_by_generic_tag(&self, tag: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.tags.contains_generic_tag(tag) {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            }
            if let Some(sub_tasks) = task.sub_tasks.as_ref() {
                for sub_task in sub_tasks.iter() {
                    if sub_task.tags.contains_generic_tag(tag) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
            if let Some(notes) = task.notes.as_ref() {
                for note in &notes.notes {
                    if note.tags.contains_generic_tag(tag) {
                        if let Some(tasks) = all_tasks.get_mut(&task.project) {
                            tasks.push(task.clone());
                        } else {
                            all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
}

fn parse_tasks(
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
fn sort_tasks(all_project_tasks: BTreeMap<String, Tasks>) -> NomosResult<Tasks> {
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

fn make_paths_to_crawl(config: &XffValue) -> NomosResult<Vec<(String, Vec<PathBuf>)>> {
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
