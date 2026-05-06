use std::sync::Arc;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::{ClientId, LurkError};

impl GameEngine {
    pub(crate) fn handle_message(
        &mut self,
        client: ClientId,
        sender_name: Arc<str>,
        recipient_name: Arc<str>,
        message: Box<str>,
    ) {
        // Find recipient player, extract snapshot and client
        let (recipient_snapshot, recipient_client) = {
            let Some(recipient_ps) = self.players.get(recipient_name.as_ref()) else {
                self.emit(Output::SendError {
                    client,
                    error_code: LurkError::OTHER,
                    message: "Player not found".into(),
                });
                return;
            };

            (recipient_ps.clone(), recipient_ps.client)
        };

        // Validate recipient is started
        if !self.ensure_started(&recipient_snapshot, client) {
            return;
        }

        // Validate recipient is connected
        let Some(recipient_client) = recipient_client else {
            self.emit(Output::SendError {
                client,
                error_code: LurkError::OTHER,
                message: "Not connected".into(),
            });
            return;
        };

        // Forward message to recipient
        self.emit(Output::SendMessage {
            client: recipient_client,
            sender_name,
            recipient_name,
            message,
            narration: false,
        });
    }
}
