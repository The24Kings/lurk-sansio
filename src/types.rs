//! Protocol-level type re-exports and transport-agnostic identifiers.
//!
//! This module re-exports key types from [`lurk_lcsc`] so that consumers of this crate
//! do not need to add a direct dependency on the protocol library.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Bitflags describing character state (alive, started, monster, etc.).
///
/// Re-exported from [`lurk_lcsc::CharacterFlags`].
pub use lurk_lcsc::CharacterFlags;

/// Error codes defined by the Lurk protocol.
///
/// Re-exported from [`lurk_lcsc::LurkError`].
pub use lurk_lcsc::LurkError;

/// Packet type discriminants defined by the Lurk protocol.
///
/// Re-exported from [`lurk_lcsc::PktType`].
pub use lurk_lcsc::PktType;

/// Opaque client identifier assigned by the event loop.
///
/// Maps to whatever transport the event loop uses (TCP, WebSocket, etc.).
/// The inner `u64` has no meaning to the engine — it is simply echoed back in [`crate::Output`]
/// variants so the event loop knows which connection to target.
///
/// ```rust
/// use lurk_sansio::ClientId;
///
/// let id = ClientId(42);
/// assert_eq!(format!("{id}"), "Client(42)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client({})", self.0)
    }
}
