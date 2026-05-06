//! The core sans-IO game engine.
//!
//! [`GameEngine`] is the central struct of this crate. It holds all game state in memory
//! and exposes a simple input/output interface:
//!
//! 1. Call [`GameEngine::handle_input`] with an [`Input`] event.
//! 2. Call [`GameEngine::poll_output`] in a loop to drain the resulting [`Output`] events.
//!
//! The engine never performs IO itself — that responsibility belongs to your event loop.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::input::Input;
use crate::output::{ConnectionInfo, Output, RoomInfo};
use crate::state::{GameConfig, PlayerState, Room};
use crate::types::{ClientId, LurkError};

/// The core sans-IO game engine.
///
/// Accepts [`Input`] events, mutates internal state, and produces [`Output`] events
/// that the event loop must execute.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use lurk_sansio::{GameEngine, GameConfig, Input, Output, ClientId};
///
/// let mut engine = GameEngine::new(
///     HashMap::new(),
///     GameConfig { initial_points: 100, stat_limit: 65535 },
/// );
///
/// engine.handle_input(Input::ClientConnected { client: ClientId(1) });
/// // No outputs for a bare connect — the event loop sends Version/Game on its own.
/// assert!(engine.poll_output().is_none());
/// ```
pub struct GameEngine {
    pub(crate) players: HashMap<Arc<str>, PlayerState>,
    pub(crate) rooms: HashMap<u16, Room>,
    pub(crate) config: GameConfig,
    pub(crate) outputs: VecDeque<Output>,
}

impl GameEngine {
    /// Create a new game engine with the given rooms and configuration.
    ///
    /// The `rooms` map defines the game world (including monsters and connections).
    /// The `config` controls character creation constraints.
    pub fn new(rooms: HashMap<u16, Room>, config: GameConfig) -> Self {
        Self {
            players: HashMap::new(),
            rooms,
            config,
            outputs: VecDeque::new(),
        }
    }

    /// Get an immutable reference to the players map.
    pub fn players(&self) -> &HashMap<Arc<str>, PlayerState> {
        &self.players
    }

    /// Get a mutable reference to the players map.
    pub fn players_mut(&mut self) -> &mut HashMap<Arc<str>, PlayerState> {
        &mut self.players
    }

    /// Get an immutable reference to the rooms map.
    pub fn rooms(&self) -> &HashMap<u16, Room> {
        &self.rooms
    }

    /// Get a mutable reference to the rooms map.
    pub fn rooms_mut(&mut self) -> &mut HashMap<u16, Room> {
        &mut self.rooms
    }

    /// Feed an input event into the engine.
    ///
    /// Dispatches to the appropriate handler based on the [`Input`] variant.
    /// Any resulting side-effects are queued internally and can be retrieved
    /// via [`GameEngine::poll_output`].
    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::ClientConnected { .. } => {
                // No-op: the event loop handles sending Version/Game packets.
                // The engine doesn't need to track connections until Character is sent.
            }
            Input::Character {
                client,
                name,
                attack,
                defense,
                regen,
                description,
            } => self.handle_character(client, name, attack, defense, regen, description),
            Input::Start { client } => self.handle_start(client),
            Input::ChangeRoom {
                client,
                room_number,
            } => self.handle_change_room(client, room_number),
            Input::Fight { client } => self.handle_fight(client),
            Input::PvpFight {
                client,
                target_name,
            } => self.handle_pvp_fight(client, target_name),
            Input::Loot {
                client,
                target_name,
            } => self.handle_loot(client, target_name),
            Input::Message {
                client,
                sender_name,
                recipient_name,
                message,
            } => self.handle_message(client, sender_name, recipient_name, message),
            Input::Leave { client } => self.handle_leave(client),
        }
    }

    /// Drain the next output event. Returns `None` when the queue is empty.
    ///
    /// Call this in a loop after each [`GameEngine::handle_input`] to collect all
    /// side-effects that the event loop must execute.
    pub fn poll_output(&mut self) -> Option<Output> {
        self.outputs.pop_front()
    }

    // ==================== Internal helpers ====================

    /// Push an output event onto the queue.
    pub(crate) fn emit(&mut self, output: Output) {
        self.outputs.push_back(output);
    }

    /// Find a player by their client ID. Returns the player name and a mutable reference.
    pub(crate) fn player_from_client(
        &mut self,
        client: ClientId,
    ) -> Option<(Arc<str>, &mut PlayerState)> {
        self.players
            .iter_mut()
            .find(|(_, ps)| ps.client == Some(client))
            .map(|(name, ps)| (name.clone(), ps))
    }

    /// Check that a player is started and ready. Emits an error to the client if not.
    /// Returns `true` if the player is started and ready.
    pub(crate) fn ensure_started(&mut self, player: &PlayerState, client: ClientId) -> bool {
        if !player.character.flags.is_started() && !player.character.flags.is_ready() {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::NOTREADY,
                message: "Start the game first!".into(),
            });
            return false;
        }
        true
    }

    /// Emit a room info packet to a client for the given room.
    pub(crate) fn send_room(&mut self, client: ClientId, room: &Room) {
        self.outputs.push_back(Output::SendRoom {
            client,
            room: RoomInfo::from(room),
        });
    }

    /// Emit character data for all players and monsters in a room to a specific client.
    pub(crate) fn send_room_contents(&mut self, client: ClientId, room: &Room) {
        // Send all players in the room
        for name in &room.players {
            if let Some(ps) = self.players.get(name) {
                self.outputs.push_back(Output::SendCharacter {
                    client,
                    character: ps.character.clone(),
                });
            }
        }

        // Send all monsters in the room
        if let Some(monsters) = &room.monsters {
            for monster in monsters {
                self.outputs.push_back(Output::SendCharacter {
                    client,
                    character: monster.to_character(),
                });
            }
        }
    }

    /// Emit all connection exits for a room to a specific client.
    pub(crate) fn send_connections(&mut self, client: ClientId, room_id: u16) {
        let Some(room) = self.rooms.get(&room_id) else {
            return;
        };

        // Clone connections to avoid borrow conflict
        let connections: Vec<ConnectionInfo> = room
            .connections
            .values()
            .map(ConnectionInfo::from)
            .collect();

        for conn in connections {
            self.outputs.push_back(Output::SendConnection {
                client,
                connection: conn,
            });
        }
    }
}
