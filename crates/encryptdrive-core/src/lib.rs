#![forbid(unsafe_code)]
//! GUI-independent product identity and readiness boundary.
//!
//! Phase 1A does not implement vault operations, cryptography, storage, authentication,
//! networking, or telemetry. The API makes the engineering-preview state explicit.

/// Stable product identity for shell chrome and diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationIdentity {
    /// User-visible product name.
    pub product_name: &'static str,
    /// Version from this crate's package metadata.
    pub version: &'static str,
}

/// Product features whose availability must be reported truthfully.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Creating an encrypted vault.
    VaultCreation,
    /// Opening an existing vault.
    VaultOpening,
    /// Importing files into a vault.
    FileImport,
    /// Exporting files from a vault.
    FileExport,
    /// Moving, restoring, or permanently deleting trashed entries.
    TrashOperations,
    /// Recovering a damaged or interrupted vault operation.
    Recovery,
    /// Authenticating an account or user.
    Authentication,
    /// Connecting to network services.
    NetworkServices,
    /// Sending product telemetry.
    Telemetry,
}

impl Capability {
    /// Every user-facing capability tracked by Phase 1A readiness.
    pub const ALL: [Self; 9] = [
        Self::VaultCreation,
        Self::VaultOpening,
        Self::FileImport,
        Self::FileExport,
        Self::TrashOperations,
        Self::Recovery,
        Self::Authentication,
        Self::NetworkServices,
        Self::Telemetry,
    ];
}

/// High-level application readiness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessState {
    /// The native shell is available, but product data operations are not implemented.
    EngineeringPreview,
}

/// Truthful summary of the current application's supported behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductReadiness {
    /// Current product state.
    pub state: ReadinessState,
    /// Short status suitable for the application header.
    pub message: &'static str,
    /// Capabilities verified as available in this build.
    pub available_capabilities: &'static [Capability],
}

impl ProductReadiness {
    /// Returns whether the build implements the requested product capability.
    pub fn is_available(self, capability: Capability) -> bool {
        self.available_capabilities.contains(&capability)
    }
}

/// Returns the stable EncryptDrive product identity.
pub const fn application_identity() -> ApplicationIdentity {
    ApplicationIdentity {
        product_name: "EncryptDrive",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Returns the capabilities verified as available in this engineering preview.
pub const fn readiness() -> ProductReadiness {
    ProductReadiness {
        state: ReadinessState::EngineeringPreview,
        message: "Engineering preview — vault operations unavailable",
        available_capabilities: &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability, ReadinessState, application_identity, readiness};

    #[test]
    fn product_identity_is_explicit_and_versioned() {
        let identity = application_identity();

        assert_eq!(identity.product_name, "EncryptDrive");
        assert_eq!(identity.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn preview_readiness_exposes_no_available_capabilities() {
        let status = readiness();

        assert_eq!(status.state, ReadinessState::EngineeringPreview);
        assert_eq!(
            status.message,
            "Engineering preview — vault operations unavailable"
        );
        assert!(status.available_capabilities.is_empty());
    }

    #[test]
    fn every_known_product_capability_is_unavailable() {
        let status = readiness();

        for capability in Capability::ALL {
            assert!(!status.is_available(capability));
        }
    }
}
