//! Game world model: characters, rooms, monsters, and server configuration.
//!
//! All types in this module are serializable and free of IO concerns. They represent
//! the canonical server-side state that the [`crate::GameEngine`] mutates in response
//! to [`crate::Input`] events.

use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexSet;
use lurk_protocol::CharacterFlags;
use serde::{Deserialize, Serialize};

use crate::types::ClientId;

/// Server-side character representation, free of any IO concerns.
///
/// This is the authoritative view of a player or monster as seen by the engine.
/// The event loop serializes this into Lurk `CHARACTER` packets when the engine emits
/// [`crate::Output::SendCharacter`] or [`crate::Output::AlertRoom`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// The character's unique name (up to 32 bytes in the protocol).
    pub name: Arc<str>,
    /// Bitflags: alive, started, ready, monster, battle, etc.
    pub flags: CharacterFlags,
    /// Offensive stat — contributes to damage dealt.
    pub attack: u16,
    /// Defensive stat — reduces incoming damage.
    pub defense: u16,
    /// Regeneration stat — health recovered after each fight round.
    pub regen: u16,
    /// Current health points. Negative or zero means dead.
    pub health: i16,
    /// Gold carried by this character.
    pub gold: u16,
    /// Room number the character currently occupies.
    pub current_room: u16,
    /// Flavor text description of the character.
    pub description: Box<str>,
}

impl Character {
    /// Create a new character with the given stats and default server-controlled fields.
    ///
    /// Sets health to 100, gold to 0, room to 0, and flags to `ALIVE`.
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use lurk_engine::Character;
    ///
    /// let ch = Character::with_defaults(
    ///     Arc::from("Link"),
    ///     50, 30, 20,
    ///     "Hero of Time".into(),
    /// );
    /// assert_eq!(ch.health, 100);
    /// assert_eq!(ch.current_room, 0);
    /// ```
    pub fn with_defaults(
        name: Arc<str>,
        attack: u16,
        defense: u16,
        regen: u16,
        description: Box<str>,
    ) -> Self {
        Self {
            name,
            flags: CharacterFlags::alive(),
            attack,
            defense,
            regen,
            health: 100,
            gold: 0,
            current_room: 0,
            description,
        }
    }
}

/// A room in the game world.
///
/// Rooms form the game map. Each room has a set of connections (exits) to other rooms,
/// a list of players currently present, and optionally a list of monsters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Unique numeric identifier for this room.
    pub room_number: u16,
    /// Short display title (e.g. "Kokiri Forest").
    pub title: Box<str>,
    /// Map of connected room numbers to their [`Connection`] metadata.
    pub connections: HashMap<u16, Connection>,
    /// Longer prose description of the room.
    pub description: Box<str>,
    /// Names of players currently in this room (insertion-ordered).
    pub players: IndexSet<Arc<str>>,
    /// Monsters inhabiting this room, if any.
    pub monsters: Option<Vec<Monster>>,
}

/// An exit/connection from one room to another.
///
/// Sent to clients so they know which rooms are reachable from their current location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// The destination room number.
    pub room_number: u16,
    /// Short display title for the exit (e.g. "North Door").
    pub title: Box<str>,
    /// Description of the exit shown to the player.
    pub description: Box<str>,
}

/// A monster NPC in the game world.
///
/// Monsters are fought by players and can be looted once dead. They exist within a
/// specific room and do not move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    /// The monster's unique name within its room.
    pub name: Arc<str>,
    /// Room this monster resides in.
    pub current_room: u16,
    /// Maximum health (used for respawn logic, if any).
    pub max_health: i16,
    /// Current health. Zero or below means dead.
    pub health: i16,
    /// Offensive stat.
    pub attack: u16,
    /// Defensive stat.
    pub defense: u16,
    /// Gold dropped on death.
    pub gold: u16,
    /// Flavor text description.
    pub description: Box<str>,
}

/// Configuration for the game engine.
///
/// Controls character creation constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Total stat points a player may allocate across attack, defense, and regen.
    pub initial_points: u16,
    /// Maximum value for any single stat (unused by default, reserved for future use).
    pub stat_limit: u16,
}

/// Internal player state: character data plus the optional client connection.
///
/// When `client` is `None`, the player is disconnected but their character persists
/// in the world (allowing reconnection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    /// The player's character data.
    pub character: Character,
    /// The connected client, or `None` if the player is offline.
    pub client: Option<ClientId>,
}

impl Monster {
    /// Convert this monster into a [`Character`] representation for sending to clients.
    ///
    /// Sets the `MONSTER` and `BATTLE` flags, plus `ALIVE` or dead flags based on health.
    pub fn to_character(&self) -> Character {
        let mut flags = CharacterFlags::MONSTER | CharacterFlags::BATTLE;
        if self.health <= 0 {
            flags |= CharacterFlags::dead();
        } else {
            flags |= CharacterFlags::alive();
        }

        Character {
            name: self.name.clone(),
            flags,
            attack: self.attack,
            defense: self.defense,
            regen: 0,
            health: self.health,
            gold: self.gold,
            current_room: self.current_room,
            description: self.description.clone(),
        }
    }
}
