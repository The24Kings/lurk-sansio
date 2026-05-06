//! Side-effects produced by the [`crate::GameEngine`] for the transport layer to execute.
//!
//! After calling [`crate::GameEngine::handle_input`], poll [`Output`] events with
//! [`crate::GameEngine::poll_output`] and translate each variant into the appropriate
//! network action (sending packets, closing sockets, broadcasting, etc.).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::state::{Character, Connection, Room};
use crate::types::{ClientId, LurkError, PktType};

/// Side-effects produced by the game engine.
///
/// The event loop is responsible for executing these — the engine itself never performs IO.
/// Each variant describes *what* should happen; the event loop decides *how*.
///
/// # Variant Categories
///
/// | Category | Variants |
/// |----------|----------|
/// | Unicast (one client) | `SendError`, `SendAccept`, `SendCharacter`, `SendRoom`, `SendConnection`, `SendMessage` |
/// | Room-wide | `Narrate`, `AlertRoom` |
/// | Global | `Broadcast` |
/// | Lifecycle | `Disconnect` |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    /// Send an error packet to a specific client.
    SendError {
        /// Target client.
        client: ClientId,
        /// The Lurk error code.
        error_code: LurkError,
        /// Human-readable error description.
        message: Box<str>,
    },

    /// Send an accept packet to a specific client, acknowledging a valid request.
    SendAccept {
        /// Target client.
        client: ClientId,
        /// The packet type that was accepted (e.g. `CHARACTER`).
        accepted_type: PktType,
    },

    /// Send character data to a specific client.
    SendCharacter {
        /// Target client.
        client: ClientId,
        /// The character to serialize and send.
        character: Character,
    },

    /// Send room info to a specific client.
    SendRoom {
        /// Target client.
        client: ClientId,
        /// Room metadata to send.
        room: RoomInfo,
    },

    /// Send a connection/exit description to a specific client.
    SendConnection {
        /// Target client.
        client: ClientId,
        /// The exit/connection info.
        connection: ConnectionInfo,
    },

    /// Send a message to a specific client.
    SendMessage {
        /// Target client (the recipient).
        client: ClientId,
        /// Display name of the sender.
        sender_name: Arc<str>,
        /// Display name of the recipient.
        recipient_name: Arc<str>,
        /// The message body.
        message: Box<str>,
        /// Whether this is a server narration vs. a player message.
        narration: bool,
    },

    /// Broadcast a server message to **all** connected players.
    Broadcast {
        /// The message to broadcast.
        message: Box<str>,
    },

    /// Send a narration/server message to all players in a specific room.
    Narrate {
        /// The room to narrate to.
        room_number: u16,
        /// The narration text.
        message: Box<str>,
        /// Whether to flag this as a narration packet.
        narration: bool,
    },

    /// Alert all players in a room about a character update (player or monster).
    AlertRoom {
        /// The room whose occupants should receive the update.
        room_number: u16,
        /// Updated character data.
        character: Character,
    },

    /// Signal the event loop to close this client's connection.
    Disconnect {
        /// The client to disconnect.
        client: ClientId,
    },
}

/// Serializable room information included in [`Output::SendRoom`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    /// The room's numeric identifier.
    pub room_number: u16,
    /// Short display title.
    pub title: Box<str>,
    /// Longer prose description.
    pub description: Box<str>,
}

impl From<&Room> for RoomInfo {
    /// Convert a full [`Room`] into the lightweight [`RoomInfo`] sent to clients.
    fn from(room: &Room) -> Self {
        Self {
            room_number: room.room_number,
            title: room.title.clone(),
            description: room.description.clone(),
        }
    }
}

/// Serializable connection/exit information included in [`Output::SendConnection`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Destination room number.
    pub room_number: u16,
    /// Short display title for the exit.
    pub title: Box<str>,
    /// Description of the exit.
    pub description: Box<str>,
}

impl From<&Connection> for ConnectionInfo {
    /// Convert a full [`Connection`] into the lightweight [`ConnectionInfo`] sent to clients.
    fn from(conn: &Connection) -> Self {
        Self {
            room_number: conn.room_number,
            title: conn.title.clone(),
            description: conn.description.clone(),
        }
    }
}
