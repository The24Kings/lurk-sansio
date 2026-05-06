use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::{CharacterFlags, ClientId, LurkError};

impl GameEngine {
    pub(crate) fn handle_start(&mut self, client: ClientId) {
        // Find player, validate, activate
        let player_name = {
            let Some((name, ps)) = self.player_from_client(client) else {
                return;
            };

            if !ps.character.flags.is_ready() {
                self.emit(Output::SendError {
                    client,
                    error_code: LurkError::NOTREADY,
                    message: "Supply a valid player first!".into(),
                });
                return;
            }

            ps.character.flags |= CharacterFlags::STARTED;
            name
        };

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
