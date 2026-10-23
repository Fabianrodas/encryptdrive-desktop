//! Shared, GUI-independent state for the two Phase 0.7 framework prototypes.
//! All data is synthetic and remains in memory.

pub mod fixture;
pub mod model;
pub mod theme;

pub use fixture::{Fixture, make_fixture};
pub use model::{
    Dialog, DialogKind, Entry, EntryId, EntryKind, ExplorerModel, OperationStatus, Page,
    SortColumn, SyntheticAction,
};
pub use theme::{Appearance, SemanticTokens, ThemeMode};
