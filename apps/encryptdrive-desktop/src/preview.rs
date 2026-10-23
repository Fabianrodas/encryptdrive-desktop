//! Development-only synthetic application state for design review.
//!
//! This module contains no file, vault, network, persistence, or cryptographic APIs.

mod model;
mod state;

pub use crate::Destination;
pub use model::{DatasetSize, Entry, EntryId, EntryKind, NameValidationError, PreviewModel};
pub use state::{ExplorerState, Modifier, SortDirection, SortKey};

/// Named screens and states in the design preview.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewScenario {
    /// First-run create/open vault choice.
    VaultSelection,
    /// Representative create vault form; submitting is not a real operation.
    CreateVault,
    /// Representative vault unlock form; submitting does not authenticate.
    UnlockVault,
    /// Account choices after the conceptual vault unlock step.
    AccountSelection,
    /// Representative account login form; submitting does not authenticate.
    AccountLogin,
    /// Representative account registration form.
    RegisterAccount,
    /// Populated synthetic file table.
    Explorer,
    /// Empty folder contents.
    EmptyExplorer,
    /// Nested folder navigation.
    FolderNavigation,
    /// Synthetic name-search results.
    SearchResults,
    /// Transitional in-memory loading state.
    Loading,
    /// Multi-selected synthetic entries and contextual actions.
    MultiSelection,
    /// Import progress; no input is read from disk.
    ImportProgress,
    /// Plaintext export warning; no output file is written.
    ExportWarning,
    /// Trash listing and restore action.
    Trash,
    /// Permanent-delete confirmation.
    PermanentDelete,
    /// Account settings page.
    AccountSettings,
    /// Vault settings page.
    VaultSettings,
    /// Synthetic metadata-recovery notice.
    RecoveryNotice,
    /// Busy shell with simulated progress.
    Busy,
    /// Persistent in-window synthetic error.
    Error,
    /// Disabled action explanations.
    Disabled,
}

impl PreviewScenario {
    /// All states available from the development preview picker.
    pub const ALL: [Self; 22] = [
        Self::VaultSelection,
        Self::CreateVault,
        Self::UnlockVault,
        Self::AccountSelection,
        Self::AccountLogin,
        Self::RegisterAccount,
        Self::Explorer,
        Self::EmptyExplorer,
        Self::FolderNavigation,
        Self::SearchResults,
        Self::Loading,
        Self::MultiSelection,
        Self::ImportProgress,
        Self::ExportWarning,
        Self::Trash,
        Self::PermanentDelete,
        Self::AccountSettings,
        Self::VaultSettings,
        Self::RecoveryNotice,
        Self::Busy,
        Self::Error,
        Self::Disabled,
    ];

    /// Human-readable development-only scenario name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::VaultSelection => "Vault selection",
            Self::CreateVault => "Create vault",
            Self::UnlockVault => "Unlock vault",
            Self::AccountSelection => "Account selection",
            Self::AccountLogin => "Account login",
            Self::RegisterAccount => "Register account",
            Self::Explorer => "Populated Files",
            Self::EmptyExplorer => "Empty folder",
            Self::FolderNavigation => "Folder navigation",
            Self::SearchResults => "Search results",
            Self::Loading => "Loading",
            Self::MultiSelection => "Multi-selection",
            Self::ImportProgress => "Import progress",
            Self::ExportWarning => "Export warning",
            Self::Trash => "Trash and restore",
            Self::PermanentDelete => "Permanent deletion",
            Self::AccountSettings => "Account settings",
            Self::VaultSettings => "Vault settings",
            Self::RecoveryNotice => "Recovery notice",
            Self::Busy => "Busy state",
            Self::Error => "Error state",
            Self::Disabled => "Disabled actions",
        }
    }
}

/// Current synthetic operation state. No worker or external side effect is created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationState {
    /// No operation is being previewed.
    Idle,
    /// A synthetic operation can be advanced by repaint ticks.
    Running { completed: u8, total: u8 },
    /// The user canceled the preview operation.
    Canceled,
    /// The preview operation completed successfully without writing data.
    Succeeded,
    /// A deterministic preview failure was selected.
    Failed,
}

impl OperationState {
    /// Starts a synthetic operation with a fixed number of repaint-driven steps.
    pub const fn running(total: u8) -> Self {
        Self::Running {
            completed: 0,
            total,
        }
    }

    /// Returns normalized progress while running.
    pub const fn progress(self) -> Option<f32> {
        match self {
            Self::Running { completed, total } if total > 0 => {
                Some(completed as f32 / total as f32)
            }
            Self::Running { .. } | Self::Idle | Self::Canceled | Self::Succeeded | Self::Failed => {
                None
            }
        }
    }

    /// Advances one synthetic step and reports success at the declared total.
    pub const fn advance(self) -> Self {
        match self {
            Self::Running { completed, total } if total > 0 => {
                let next = completed.saturating_add(1);
                if next >= total {
                    Self::Succeeded
                } else {
                    Self::Running {
                        completed: next,
                        total,
                    }
                }
            }
            Self::Running { .. } => Self::Succeeded,
            state => state,
        }
    }

    /// Cancels only a running synthetic operation.
    pub const fn cancel(self) -> Self {
        match self {
            Self::Running { .. } => Self::Canceled,
            state => state,
        }
    }

    /// Switches a running synthetic operation to its deterministic error state.
    pub const fn fail(self) -> Self {
        match self {
            Self::Running { .. } => Self::Failed,
            state => state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationState, PreviewScenario};

    #[test]
    fn preview_scenario_picker_covers_required_screen_states() {
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::VaultSelection));
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::CreateVault));
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::RegisterAccount));
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::Explorer));
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::PermanentDelete));
        assert!(PreviewScenario::ALL.contains(&PreviewScenario::RecoveryNotice));
        assert_eq!(PreviewScenario::Explorer.label(), "Populated Files");
    }

    #[test]
    fn operation_progress_is_bounded_to_the_declared_total() {
        assert_eq!(
            OperationState::Running {
                completed: 3,
                total: 8
            }
            .progress(),
            Some(0.375)
        );
        assert_eq!(OperationState::Idle.progress(), None);
    }

    #[test]
    fn operation_advances_cancels_and_fails_without_external_work() {
        let running = OperationState::running(4);
        assert_eq!(running.advance().progress(), Some(0.25));
        assert_eq!(running.cancel(), OperationState::Canceled);
        assert_eq!(running.fail(), OperationState::Failed);
        assert_eq!(
            running.advance().advance().advance().advance(),
            OperationState::Succeeded
        );
    }
}
