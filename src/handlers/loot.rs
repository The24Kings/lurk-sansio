use std::sync::Arc;

use lurk_lcsc::LurkError;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::ClientId;

impl GameEngine {
    pub(crate) fn handle_loot(&mut self, client: ClientId, target_name: Arc<str>) {
        // Find player, validate started
        let (player_name, current_room) = {
            let Some((name, ps)) = self.player_from_client(client) else {
                return;
            };

            let room = ps.character.current_room;
            let snapshot = ps.clone();

            if !self.ensure_started(&snapshot, client) {
                return;
            }

            (name, room)
        };

        // Get room and find target monster
        let Some(room) = self.rooms.get_mut(&current_room) else {
            return;
        };

        let Some(monsters) = &mut room.monsters else {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::OTHER,
                message: "No monsters to loot!".into(),
            });
            return;
        };

        let Some(to_loot) = monsters
            .iter_mut()
            .find(|m| m.name.as_ref() == target_name.as_ref())
        else {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADMONSTER,
                message: "Monster doesn't exist!".into(),
            });
            return;
        };

        if to_loot.health > 0 {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADMONSTER,
                message: "Monster is still alive!".into(),
            });
            return;
        }

        if to_loot.gold == 0 {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADMONSTER,
                message: "Monster already looted!".into(),
            });
            return;
        }

        // Transfer gold
        let gold = to_loot.gold;
        to_loot.gold = 0;
        let monster_character = to_loot.to_character();

        // Update player gold
        let Some(ps) = self.players.get_mut(&player_name) else {
            return;
        };
        ps.character.gold += gold;
        let updated_player = ps.character.clone();

        // Send updated player and monster to the client
        if let Some(_) = self.players.get(&player_name) {
            self.emit(Output::AlertRoom {
                room_number: current_room,
                character: updated_player,
            });

            self.emit(Output::AlertRoom {
                room_number: current_room,
                character: monster_character,
            });
        }
    }
}
