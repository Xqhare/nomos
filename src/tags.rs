use std::collections::HashMap;

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
    pub fn new() -> Self {
        Self {
            kind_tags: Vec::new(),
            location_tags: Vec::new(),
            metadata_tags: HashMap::new(),
            generic_tags: Vec::new(),
        }
    }
    pub fn add_kind(&mut self, kind: &str) {
        self.kind_tags.push(kind.to_string());
    }
    pub fn add_location(&mut self, location: &str) {
        self.location_tags.push(location.to_string());
    }
    pub fn add_generic_tag(&mut self, tag: &str) {
        self.generic_tags.push(tag.to_string());
    }
    pub fn add_metadata_tag(&mut self, key: &str, value: &str) {
        self.metadata_tags
            .insert(key.to_string(), value.to_string());
    }
    pub fn remove_kinds(&mut self, kinds: &[&str]) {
        self.kind_tags
            .retain(|kind| !kinds.contains(&kind.as_str()));
    }
    pub fn remove_locations(&mut self, locations: &[&str]) {
        self.location_tags
            .retain(|location| !locations.contains(&location.as_str()));
    }
    pub fn remove_generic_tags(&mut self, tags: &[&str]) {
        self.generic_tags
            .retain(|tag| !tags.contains(&tag.as_str()));
    }
    pub fn remove_metadata_tags(&mut self, tags: &[&str]) {
        for tag in tags {
            self.metadata_tags.remove(tag.clone());
        }
    }
}
