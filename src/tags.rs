use std::collections::HashMap;

/// A collection of tags for a task or note
#[derive(Debug, Clone)]
pub struct Tags {
    /// Starting with `+`
    pub kind_tags: Vec<String>,
    /// Starting with `@`
    pub location_tags: Vec<String>,
    /// containing an `=`
    pub metadata_tags: HashMap<String, String>,
    /// Starting with `#`
    pub generic_tags: Vec<String>,
}

impl Tags {
    /// Creates a new, empty set of tags
    pub fn new() -> Self {
        Self {
            kind_tags: Vec::new(),
            location_tags: Vec::new(),
            metadata_tags: HashMap::new(),
            generic_tags: Vec::new(),
        }
    }
    /// Adds a tag of the kind `kind`
    pub fn add_kind(&mut self, kind: &str) {
        self.kind_tags.push(kind.to_string());
    }
    /// Adds a tag of the location `location`
    pub fn add_location(&mut self, location: &str) {
        self.location_tags.push(location.to_string());
    }
    /// Adds a tag of the generic `tag`
    pub fn add_generic_tag(&mut self, tag: &str) {
        self.generic_tags.push(tag.to_string());
    }
    /// Adds a tag of the metadata `key=value`
    pub fn add_metadata_tag(&mut self, key: &str, value: &str) {
        self.metadata_tags
            .insert(key.to_string(), value.to_string());
    }
    /// Removes all kind tags in provided `kinds`
    pub fn remove_kinds(&mut self, kinds: &[&str]) {
        self.kind_tags
            .retain(|kind| !kinds.contains(&kind.as_str()));
    }
    /// Removes all location tags in provided `locations`
    pub fn remove_locations(&mut self, locations: &[&str]) {
        self.location_tags
            .retain(|location| !locations.contains(&location.as_str()));
    }
    /// Removes all generic tags in provided `tags`
    pub fn remove_generic_tags(&mut self, tags: &[&str]) {
        self.generic_tags
            .retain(|tag| !tags.contains(&tag.as_str()));
    }
    /// Removes all metadata tags in provided `tags`
    ///
    /// # Note
    ///
    /// Only the key of the pair is needed for removal
    pub fn remove_metadata_tags(&mut self, tags: &[&str]) {
        for tag in tags {
            self.metadata_tags.remove(*tag);
        }
    }
    /// Checks if a tag of the kind `kind` exists
    pub fn contains_kind(&self, kind: &str) -> bool {
        self.kind_tags.contains(&kind.to_string())
    }
    /// Checks if a tag of the location `location` exists
    pub fn contains_location(&self, location: &str) -> bool {
        self.location_tags.contains(&location.to_string())
    }
    /// Checks if a tag of the generic `tag` exists
    pub fn contains_generic_tag(&self, tag: &str) -> bool {
        self.generic_tags.contains(&tag.to_string())
    }
}
