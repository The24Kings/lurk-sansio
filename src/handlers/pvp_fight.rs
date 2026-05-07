use lurk_lcsc::LurkError;
use std::sync::Arc;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::ClientId;

impl GameEngine {
    pub(crate) fn handle_pvp_fight(&mut self, client: ClientId, _target_name: Arc<str>) {
        self.emit(Output::SendError {
            client,
            error_code: LurkError::NOPLAYERCOMBAT,
            message: "No player combat allowed".into(),
        });
    }
}
