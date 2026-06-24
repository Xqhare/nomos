use std::{collections::BTreeMap, path::PathBuf};

use mawu::read::json;
use nemesis::{NemesisError, NemesisResultExt};

use crate::{
    error::{NomosError, NomosResult},
    nomos::utils::{make_paths_to_crawl, parse_tasks, sort_tasks},
    task::{Task, Tasks},
    utils::{TaskStatus, calc_line_size, read_file_u8, save_file_u8, shift_task_lines_by_offset},
};

mod utils;

/// The main Nomos struct
///
/// Construct via `Nomos::new()`.
///
/// To update a task, use `Nomos::update_task()`.
///
/// # Notes
///
/// This struct automatically updates the file on disk.
///
/// # Example
/// ```
/// use nomos::Nomos;
///
/// let mut nomos = Nomos::new("config.json");
/// assert!(nomos.is_err()); // Config file does not exist
/// ```
pub struct Nomos {
    tasks: Tasks,
}

impl Nomos {
    /// Creates a new Nomos struct
    ///
    /// # Errors
    ///
    /// Returns an error if the config file does not exist or if it is invalid JSON
    pub fn new<P: Into<PathBuf>>(global_config_file: P) -> NomosResult<Self> {
        let global_config_file = global_config_file.into();
        let config = json(&global_config_file).add_ctx(format!(
            "Nomos failed to load global config from {:?}",
            &global_config_file
        ))?;

        let tasks: Tasks = sort_tasks(parse_tasks(make_paths_to_crawl(&config)?)?)?;

        Ok(Nomos { tasks })
    }
    /// Returns a reference to the list of tasks
    pub fn get_tasks(&self) -> &Tasks {
        &self.tasks
    }
    /// Updates a task
    ///
    /// # Arguments
    ///
    /// * `updated_task` - The task to update. Must have the same `file_data` as the task to be updated
    ///
    /// # Returns
    ///
    /// Returns `()` if the task was successfully updated
    ///
    /// # Errors
    ///
    /// Returns an error if the task does not exist.
    /// Also errors if the passed in task does not have the same `file_data` as the task to be updated.
    /// Lastly, errors if the shifting of subsequent tasks fails.
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
    /// Removes a task
    ///
    /// # Arguments
    ///
    /// * `task` - The task to remove. Must have the same `file_data` as the task to be removed
    ///
    /// # Returns
    ///
    /// Returns `()` if the task was successfully removed
    ///
    /// # Errors
    ///
    /// Returns an error if the task does not exist or could not be found.
    /// Also errors if the shifting of subsequent tasks fails.
    ///
    /// # Notes
    ///
    /// Please note, the passed in task _must_ have the same `file_data` as the task to be updated.
    pub fn remove_task(&mut self, task: &Task) -> NomosResult<()> {
        let mut to_remove = None;
        let mut task_ls = 0;
        let mut start_line = 0;
        for (i, t) in self.tasks.iter().enumerate() {
            if t.file_data == task.file_data {
                to_remove = Some(i);
                task_ls = calc_line_size(&t) as i64;
                start_line = t.file_data.line.saturating_add(task_ls as u32);
                break;
            }
        }
        if let Some(i) = to_remove {
            self.tasks.remove(i);
            shift_task_lines_by_offset(&mut self.tasks, -task_ls, start_line)?;
        }
        Ok(())
    }
    /// Adds a task to the end of the list
    ///
    /// # Arguments
    ///
    /// * `task` - The task to add
    ///
    /// # Returns
    ///
    /// Returns `()` if the task was successfully added
    ///
    /// # Errors
    ///
    /// Returns an error if the task already exists.
    ///
    /// # Notes
    ///
    /// New tasks are always added to the end of the list. The associated `file_data` `line` field is not updated.
    /// The `path` field of `file_data` is assumed to be correct.
    pub fn add_task(&mut self, task: Task) -> NomosResult<()> {
        for t in self.tasks.iter() {
            if t.title == task.title && t.project == task.project {
                return Err(NemesisError::new(
                    "nomos::Nomos::add_task",
                    NomosError::Task("Task already exists".to_string()),
                )
                .add_ctx(format!(
                    "Tried to add task: {}:{} in file: {:?}",
                    &task.project, &task.title, &task.file_data.path
                ))
                .add_ctx(
                    "Should you want to update the task instead, please use Nomos::update_task(). Should you want to move the task, please use Nomos::remove_task() first.",
                ));
            }
        }
        let file_path = task.file_data.path.clone();
        let mut file = read_file_u8(&file_path)?;
        file.extend_from_slice(task.to_string().as_bytes());
        save_file_u8(file_path, &file)?;
        self.tasks.push(task);
        Ok(())
    }
    /// Returns all tasks with the given status, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `status` - The status of the tasks to return
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given status, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// This function only allows for a single status at a time.
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
                if task.sub_tasks.is_some() {
                    for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                        if sub_task.status == status {
                            if let Some(tasks) = all_tasks.get_mut(&task.project) {
                                tasks.push(sub_task.clone());
                            } else {
                                all_tasks
                                    .insert(task.project.clone(), vec![sub_task.clone()].into());
                            }
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    /// Returns all tasks with the given priority, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority of the tasks to return
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given priority, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The priority search is case insensitive
    pub fn get_tasks_by_priority(&self, priority: char) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if let Some(task_prio) = task.priority
                && task_prio.to_lowercase().next() == priority.to_lowercase().next()
            {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            } else {
                if task.sub_tasks.is_some() {
                    for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                        if sub_task.priority == Some(priority) {
                            if let Some(tasks) = all_tasks.get_mut(&task.project) {
                                tasks.push(sub_task.clone());
                            } else {
                                all_tasks
                                    .insert(task.project.clone(), vec![sub_task.clone()].into());
                            }
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    /// Returns all tasks with the given title, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the tasks to return
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given title, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The title search is case insensitive but whitespace sensitive
    ///
    /// While task titles must be unique inside one project, they can be shared between projects. This function returns all tasks of any project matching the passed in string.
    pub fn get_tasks_by_title(&self, title: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.title.to_lowercase().as_str() == title.to_lowercase().as_str() {
                if let Some(tasks) = all_tasks.get_mut(&task.project) {
                    tasks.push(task.clone());
                } else {
                    all_tasks.insert(task.project.clone(), vec![task.clone()].into());
                }
            } else {
                if task.sub_tasks.is_some() {
                    for sub_task in task.sub_tasks.as_ref().unwrap().iter() {
                        if sub_task.title.to_lowercase().as_str() == title.to_lowercase().as_str() {
                            if let Some(tasks) = all_tasks.get_mut(&task.project) {
                                tasks.push(sub_task.clone());
                            } else {
                                all_tasks
                                    .insert(task.project.clone(), vec![sub_task.clone()].into());
                            }
                        }
                    }
                }
            }
        }
        sort_tasks(all_tasks)
    }
    /// Returns all tasks with the given project, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `project` - The project name of the tasks to return
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given project, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The project search is case insensitive
    pub fn get_tasks_by_project(&self, project: &str) -> NomosResult<Tasks> {
        let mut all_tasks: BTreeMap<String, Tasks> = BTreeMap::new();
        for task in self.tasks.iter() {
            if task.project.to_lowercase().as_str() == project.to_lowercase().as_str() {
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
    /// Returns all tasks with the given tags, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `tags` - The tags of the tasks to return. Each tag must have the leading `#`, `@`, or
    /// `+` marker
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given tags, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
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
    /// Returns all tasks with the given kind, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the tasks to return. Must not have the leading `+` marker
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given kind, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The kind search is case sensitive, and requires the leading `+` marker
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
    /// Returns all tasks with the given location, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the tasks to return. Must not have the leading `@` marker
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given location, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The location search is case sensitive, and requires the leading `@` marker
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
    /// Returns all tasks with the given generic tag, sorted by Kahn's algorithm
    ///
    /// # Arguments
    ///
    /// * `tag` - The generic tag of the tasks to return. Must not have the leading `#` marker
    ///
    /// # Returns
    ///
    /// Returns a list of tasks with the given generic tag, sorted by Kahn's algorithm
    ///
    /// # Errors
    ///
    /// Returns an error if the tasks could not be sorted
    ///
    /// # Notes
    ///
    /// The generic tag search is case sensitive, and requires the leading `#` marker
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
