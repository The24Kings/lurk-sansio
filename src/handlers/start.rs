use lurk_lcsc::CharacterFlags;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::ClientId;

impl GameEngine {
    pub(crate) fn handle_start(&mut self, client: ClientId) {
        // Find player, validate, activate
        let player_name = {
            let Some((name, ps)) = self.player_from_client(client) else {
                return;
            };

            let snapshot = ps.clone();

            if !self.ensure_ready(&snapshot, client) {
                return;
            }

            name
        };

        // Set the STARTED flag.
        if let Some(ps) = self.players.get_mut(&player_name) {
            ps.character.flags |= CharacterFlags::STARTED;
        }

        // Send updated character to the client
        if let Some(ps) = self.players.get(&player_name) {
            self.emit(Output::SendCharacter {
                client,
                character: ps.character.clone(),
            });
        }

        // Alert room 0 about the new player and broadcast
        if let Some(ps) = self.players.get(&player_name) {
            self.emit(Output::AlertRoom {
                room_number: 0,
                character: ps.character.clone(),
            });
        }
        self.emit(Output::Broadcast {
            message: format!("{} has started the game!", player_name).into(),
        });

        // Add player to starting room
        if let Some(room) = self.rooms.get_mut(&0) {
            room.players.insert(player_name.clone());
        }

        // Send room data, connections, and contents
        if let Some(room) = self.rooms.get(&0).cloned() {
            self.send_room(client, &room);
        }

        self.send_connections(client, 0);

        if let Some(room) = self.rooms.get(&0).cloned() {
            self.send_room_contents(client, &room);
        }
    }
}
