#![forbid(unsafe_code)]
//! Reserved boundary for future EncryptDrive cryptographic implementation.
//!
//! Phase 2A-4 authorizes pure VF1 AAD byte formatting only. No cryptographic operation, key type,
//! placeholder algorithm, or security claim is exposed here. Cryptographic dependencies and
//! operations require their own separately approved phase.

/// Pure VF1 associated-data byte construction. These functions do not perform cryptography.
pub mod aad;
