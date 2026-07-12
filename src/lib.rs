#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::all)]
#![warn(clippy::restriction)]
#![expect(
    clippy::missing_docs_in_private_items,
    clippy::print_stdout,
    clippy::implicit_return,
    clippy::single_call_fn,
    clippy::str_to_string,
    clippy::question_mark_used,
    clippy::indexing_slicing,
    clippy::pattern_type_mismatch,
    clippy::arbitrary_source_item_ordering,
    clippy::doc_paragraphs_missing_punctuation,
    clippy::exhaustive_enums,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::impl_trait_in_params,
    clippy::as_conversions,
    clippy::cast_lossless,
    clippy::shadow_reuse,
    clippy::blanket_clippy_restriction_lints,
    clippy::doc_include_without_cfg,
    reason = "Ignored warnings"
)]

mod error;
/// LSP integration
pub mod lsp;
/// Nomos main module
pub mod nomos;
mod notes;
/// Parser module
pub mod parser;
mod tags;
mod task;
mod utils;
/// Version module
pub mod version;

pub use error::{NomosError, NomosResult};
pub use nomos::Nomos;
pub use version::FormatVersion;
/// Prelude
///
/// Contains common types used in Nomos
pub mod prelude {
    pub use crate::notes::{Note, Notes};
    pub use crate::tags::Tags;
    pub use crate::task::{Task, Tasks};
    pub use crate::utils::{FileData, TaskStatus};
}
