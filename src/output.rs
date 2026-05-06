use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::state::{Character, Connection, Room};
use crate::types::{ClientId, LurkError, PktType};

/// Side-effects produced by the game engine.
/// The event loop is responsible for executing these (sending bytes over sockets, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    /// Send an error packet to a specific client.
    SendError {
        client: ClientId,
        error_code: LurkError,
        message: Box<str>,
    },

    /// Send an accept packet to a specific client.
    SendAccept {
        client: ClientId,
        accepted_type: PktType,
    },

    /// Send character data to a specific client.
    SendCharacter {
        client: ClientId,
        character: Character,
    },

    /// Send room info to a specific client.
    SendRoom { client: ClientId, room: RoomInfo },

    /// Send a connection/exit to a specific client.
    SendConnection {
        client: ClientId,
        connection: ConnectionInfo,
    },

    /// Send a message to a specific client.
    SendMessage {
        client: ClientId,
        sender_name: Arc<str>,
        recipient_name: Arc<str>,
        message: Box<str>,
        narration: bool,
    },

    /// Broadcast a server message to all connected players.
    Broadcast { message: Box<str> },

    /// Send a narration/server message to all players in a room.
    Narrate {
        room_number: u16,
        message: Box<str>,
        narration: bool,
    },

    /// Alert all players in a room about a character update.
    AlertRoom {
        room_number: u16,
        character: Character,
    },

    /// Signal the event loop to close this client's connection.
    Disconnect { client: ClientId },
}

/// Serializable room information for output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub room_number: u16,
    pub title: Box<str>,
    pub description: Box<str>,
}

impl From<&Room> for RoomInfo {
    fn from(room: &Room) -> Self {
        Self {
            room_number: room.room_number,
            title: room.title.clone(),
            description: room.description.clone(),
        }
    }
}

/// Serializable connection information for output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub room_number: u16,
    pub title: Box<str>,
    pub description: Box<str>,
}

impl From<&Connection> for ConnectionInfo {
    fn from(conn: &Connection) -> Self {
        Self {
            room_number: conn.room_number,
            title: conn.title.clone(),
            description: conn.description.clone(),
        }
    }
}
