//! Input events fed into the [`crate::GameEngine`] by the transport layer.
//!
//! Each [`Input`] variant maps to either a Lurk protocol packet received from a client
//! or a connection lifecycle event detected by the event loop.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::types::ClientId;

/// Events fed into the game engine by the event loop.
///
/// Each variant corresponds to a Lurk protocol message or a connection lifecycle event.
/// The event loop is responsible for parsing raw bytes into these variants before calling
/// [`crate::GameEngine::handle_input`].
///
/// # Lifecycle
///
/// A typical client session produces inputs in this order:
///
/// 1. [`Input::ClientConnected`] — transport accepted a new connection.
/// 2. [`Input::Character`] — client submits stats.
/// 3. [`Input::Start`] — client enters the game world.
/// 4. [`Input::ChangeRoom`] / [`Input::Fight`] / [`Input::Message`] — gameplay.
/// 5. [`Input::Leave`] — client disconnects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    /// A new client has connected and been assigned an ID by the event loop.
    ///
    /// Currently a no-op inside the engine — the event loop is expected to send
    /// `VERSION` and `GAME` packets on its own.
    ClientConnected {
        /// The transport-assigned client identifier.
        client: ClientId,
    },

    /// Client submitted character stats for registration or update.
    ///
    /// The engine validates stat totals against [`crate::GameConfig::initial_points`]
    /// and either accepts or rejects the character.
    Character {
        /// The client submitting the character.
        client: ClientId,
        /// Desired character name (up to 32 bytes).
        name: Arc<str>,
        /// Attack stat allocation.
        attack: u16,
        /// Defense stat allocation.
        defense: u16,
        /// Regen stat allocation.
        regen: u16,
        /// Character description / flavor text.
        description: Box<str>,
    },

    /// Client wants to start playing (enter the game world).
    ///
    /// The character must already be registered and in the `READY` state.
    Start {
        /// The client requesting to start.
        client: ClientId,
    },

    /// Client wants to move to another room.
    ///
    /// The target room must be connected to the player's current room.
    ChangeRoom {
        /// The client requesting the move.
        client: ClientId,
        /// The destination room number.
        room_number: u16,
    },

    /// Client wants to fight a monster in their current room.
    ///
    /// Targets the alive monster with the lowest health. All players with the `BATTLE`
    /// flag in the same room join the attack.
    Fight {
        /// The client initiating the fight.
        client: ClientId,
    },

    /// Client wants to fight another player (PvP).
    ///
    /// Currently rejected by the engine — PvP is not enabled.
    PvpFight {
        /// The client initiating PvP.
        client: ClientId,
        /// Name of the target player.
        target_name: Arc<str>,
    },

    /// Client wants to loot a dead monster.
    ///
    /// The target monster must be dead and still have gold remaining.
    Loot {
        /// The client requesting loot.
        client: ClientId,
        /// Name of the monster to loot.
        target_name: Arc<str>,
    },

    /// Client wants to send a message to another player.
    Message {
        /// The sending client.
        client: ClientId,
        /// Display name of the sender.
        sender_name: Arc<str>,
        /// Display name of the recipient.
        recipient_name: Arc<str>,
        /// The message body.
        message: Box<str>,
    },

    /// Client is disconnecting (graceful leave).
    ///
    /// The engine marks the player as offline, broadcasts a leave message,
    /// and emits a [`crate::Output::Disconnect`] to finalize the connection.
    Leave {
        /// The departing client.
        client: ClientId,
    },
}
