use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use crate::types::{CharacterFlags, ClientId};

/// Server-side character representation, free of any IO concerns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: Arc<str>,
    pub flags: CharacterFlags,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description: Box<str>,
}

impl Character {
    /// Create a new character with the given stats and default server-controlled fields.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_number: u16,
    pub title: Box<str>,
    pub connections: HashMap<u16, Connection>,
    pub description: Box<str>,
    pub players: IndexSet<Arc<str>>,
    pub monsters: Option<Vec<Monster>>,
}

/// An exit/connection from one room to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub room_number: u16,
    pub title: Box<str>,
    pub description: Box<str>,
}

/// A monster NPC in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub name: Arc<str>,
    pub current_room: u16,
    pub max_health: i16,
    pub health: i16,
    pub attack: u16,
    pub defense: u16,
    pub gold: u16,
    pub description: Box<str>,
}

/// Configuration for the game engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub initial_points: u16,
    pub stat_limit: u16,
}

/// Internal player state: character data plus the optional client connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub character: Character,
    pub client: Option<ClientId>,
}

/// Convert a Monster to a Character representation (for sending to clients).
impl Monster {
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
