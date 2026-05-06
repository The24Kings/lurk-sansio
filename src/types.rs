use std::fmt;

use serde::{Deserialize, Serialize};

// Re-export protocol types from lurk_lcsc so consumers don't need to depend on it directly.
pub use lurk_lcsc::CharacterFlags;
pub use lurk_lcsc::LurkError;
pub use lurk_lcsc::PktType;

/// Opaque client identifier assigned by the event loop.
/// Maps to whatever transport the event loop uses (TCP, WebSocket, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client({})", self.0)
    }
}
