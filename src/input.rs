use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::types::ClientId;

/// Events fed into the game engine by the event loop.
/// Each variant corresponds to a Lurk protocol message (or a connection lifecycle event).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    /// A new client has connected and been assigned an ID by the event loop.
    ClientConnected { client: ClientId },

    /// Client submitted character stats for registration or update.
    Character {
        client: ClientId,
        name: Arc<str>,
        attack: u16,
        defense: u16,
        regen: u16,
        description: Box<str>,
    },

    /// Client wants to start playing.
    Start { client: ClientId },

    /// Client wants to move to another room.
    ChangeRoom { client: ClientId, room_number: u16 },

    /// Client wants to fight a monster in their current room.
    Fight { client: ClientId },

    /// Client wants to fight another player.
    PvpFight {
        client: ClientId,
        target_name: Arc<str>,
    },

    /// Client wants to loot a dead monster.
    Loot {
        client: ClientId,
        target_name: Arc<str>,
    },

    /// Client wants to send a message to another player.
    Message {
        client: ClientId,
        sender_name: Arc<str>,
        recipient_name: Arc<str>,
        message: Box<str>,
    },

    /// Client is disconnecting.
    Leave { client: ClientId },
}
