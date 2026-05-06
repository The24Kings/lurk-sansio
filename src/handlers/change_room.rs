use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::{ClientId, LurkError};

impl GameEngine {
    pub(crate) fn handle_change_room(&mut self, client: ClientId, nxt_room_id: u16) {
        // Find player, validate started
        let (player_name, cur_room_id) = {
            let Some((name, ps)) = self.player_from_client(client) else {
                return;
            };

            let cur_room = ps.character.current_room;
            let pname = name.clone();
            let snapshot = ps.clone();

            if !self.ensure_started(&snapshot, client) {
                return;
            }

            (pname, cur_room)
        };

        // Check not already in target room
        if cur_room_id == nxt_room_id {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADROOM,
                message: "Player is already in the room".into(),
            });
            return;
        }

        // Validate current room exists
        let Some(room) = self.rooms.get(&cur_room_id) else {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADROOM,
                message: "Room not found!".into(),
            });
            return;
        };

        // Validate connection exists
        if !room.connections.contains_key(&nxt_room_id) {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::BADROOM,
                message: "Invalid connection!".into(),
            });
            return;
        }

        // Apply changes: update player room
        if let Some(ps) = self.players.get_mut(&player_name) {
            ps.character.current_room = nxt_room_id;
        }

        // Remove from old room
        if let Some(cur_room) = self.rooms.get_mut(&cur_room_id) {
            cur_room.players.retain(|name| *name != player_name);
        }

        // Add to new room
        if let Some(new_room) = self.rooms.get_mut(&nxt_room_id) {
            new_room.players.insert(player_name.clone());
        }

        // Send new room data
        if let Some(new_room) = self.rooms.get(&nxt_room_id).cloned() {
            self.send_room(client, &new_room);
        }

        // Send updated character to the client
        if let Some(ps) = self.players.get(&player_name) {
            self.emit(Output::SendCharacter {
                client,
                character: ps.character.clone(),
            });
        }

        // Alert old and new rooms
        if let Some(ps) = self.players.get(&player_name) {
            let character = ps.character.clone();
            self.emit(Output::AlertRoom {
                room_number: cur_room_id,
                character: character.clone(),
            });
            self.emit(Output::AlertRoom {
                room_number: nxt_room_id,
                character,
            });
        }

        // Send connections and room contents for new room
        self.send_connections(client, nxt_room_id);

        if let Some(new_room) = self.rooms.get(&nxt_room_id).cloned() {
            self.send_room_contents(client, &new_room);
        }
    }
}
