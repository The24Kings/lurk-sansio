//! Input event handlers for the [`crate::GameEngine`].
//!
//! Each sub-module implements one handler method on [`crate::GameEngine`] corresponding to
//! an [`crate::Input`] variant. Handlers validate the request, mutate game state, and
//! emit [`crate::Output`] events.

mod change_room;
mod character;
mod fight;
mod leave;
mod loot;
mod message;
mod pvp_fight;
mod start;
