use std::sync::Arc;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::state::{Character, PlayerState};
use crate::types::{ClientId, LurkError, PktType};

impl GameEngine {
    pub(crate) fn handle_character(
        &mut self,
        client: ClientId,
        name: Arc<str>,
        attack: u16,
        defense: u16,
        regen: u16,
        description: Box<str>,
    ) {
        // Validate stats
        let total_stats = attack
            .checked_add(defense)
            .and_then(|sum| sum.checked_add(regen))
            .unwrap_or(self.config.initial_points + 1);

        if total_stats > self.config.initial_points {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::STATERROR,
                message: "Invalid stats".into(),
            });
            return;
        }

        // Check if player already exists and is started
        let old_room_number = if let Some(ps) = self.players.get(&name) {
            if ps.character.flags.is_started() {
                self.emit(Output::SendError {
                    client,
                    error_code: LurkError::PLAYEREXISTS,
                    message: "Player is already in the game.".into(),
                });
                return;
            }
            ps.character.current_room
        } else {
            0
        };

        // Insert or update the player
        let character = Character::with_defaults(name.clone(), attack, defense, regen, description);
        self.players.insert(
            name.clone(),
            PlayerState {
                character,
                client: Some(client),
            },
        );

        // Send accept and updated character
        self.emit(Output::SendAccept {
            client,
            accepted_type: PktType::CHARACTER,
        });

        if let Some(ps) = self.players.get(&name) {
            self.emit(Output::SendCharacter {
                client,
                character: ps.character.clone(),
            });
        }

        // Remove from old room if rejoining (old_room != 0 means they were somewhere)
        if old_room_number == 0 {
            return;
        }

        if let Some(room) = self.rooms.get_mut(&old_room_number) {
            room.players.retain(|n| n != &name);
        }

        self.emit(Output::Narrate {
            room_number: old_room_number,
            message: format!("{}'s corpse disappeared into a puff of smoke.", name).into(),
            narration: true,
        });

        if let Some(ps) = self.players.get(&name) {
            self.emit(Output::AlertRoom {
                room_number: old_room_number,
                character: ps.character.clone(),
            });
        }
    }
}
