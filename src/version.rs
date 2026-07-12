#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use crate::error::NomosResult;

/// Format version of a Nomos file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersion {
    /// Legacy format version (v0).
    V0,
    /// Current format version (v1).
    V1,
}

impl FormatVersion {
    /// In-File Metadata Override detection (first non-empty line HTML comment)
    pub fn detect_from_file_content(content: &str) -> Option<Self> {
        let first_line = content.lines().find(|l| !l.trim().is_empty())?;
        let trimmed = first_line.trim();
        if trimmed == "<!-- nomos: 0 -->" {
            Some(FormatVersion::V0)
        } else if trimmed == "<!-- nomos: 1 -->" {
            Some(FormatVersion::V1)
        } else {
            None
        }
    }
}
