use lurk_lcsc::CharacterFlags;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::ClientId;

impl GameEngine {
    pub(crate) fn handle_leave(&mut self, client: ClientId) {
        // Find player, deactivate
        let (player_name, current_room) = {
            let Some((name, ps)) = self.player_from_client(client) else {
                return;
            };

            ps.character.flags = CharacterFlags::empty();
            ps.client = None;

            (name, ps.character.current_room)
        };

        // Broadcast and alert room
        self.emit(Output::Broadcast {
            message: format!("{} has left the game.", player_name).into(),
        });

        if let Some(ps) = self.players.get(&player_name) {
            self.emit(Output::AlertRoom {
                room_number: current_room,
                character: ps.character.clone(),
            });
        }

        self.emit(Output::Disconnect { client });
    }
}
