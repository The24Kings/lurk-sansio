//! Protocol-level type re-exports and transport-agnostic identifiers.

use std::fmt;

use serde::{Deserialize, Serialize};

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
