//! # Lurk Sans-IO
//!
//! A **sans-IO** game engine for the [Lurk protocol](https://github.com/sethlong/lurk)
//! created by S. Seth Long, Ph.D.
//!
//! This crate contains pure game logic with **zero networking or IO** — no sockets, no async
//! runtimes, no threads. Instead it follows the *sans-IO* pattern: you feed [`Input`] events
//! in and poll [`Output`] events out. Your event loop (TCP, WebSocket, QUIC, etc.) is
//! responsible for the actual transport.
//!
//! ## Where to start
//!
//! - [`GameEngine`] — the core struct that holds all game state and processes inputs.
//! - [`Input`] — events the event loop feeds into the engine (client connect, fight, move, etc.).
//! - [`Output`] — side-effects the engine produces for the event loop to execute (send packets,
//!   disconnect clients, broadcast messages).
//! - [`state`] — the game world model: [`Character`], [`Room`], [`Monster`], [`GameConfig`].
//!
//! ## Basic Usage
//!
//! ```rust
//! use std::collections::HashMap;
//! use lurk_sansio::{GameEngine, GameConfig, Input, Output, ClientId};
//!
//! // Build your world (rooms, monsters, connections)
//! let rooms = HashMap::new();
//! let config = GameConfig { initial_points: 100, stat_limit: 65535 };
//! let mut engine = GameEngine::new(rooms, config);
//!
//! // Feed events from your event loop
//! engine.handle_input(Input::ClientConnected { client: ClientId(1) });
//!
//! // Poll outputs and send them over the wire
//! while let Some(output) = engine.poll_output() {
//!     match output {
//!         Output::SendError { client, error_code, message } => { /* send error packet */ }
//!         Output::Disconnect { client } => { /* close connection */ }
//!         _ => { /* handle other outputs */ }
//!     }
//! }
//! ```
//!
//! ## Design Goals
//!
//! - **Testable**: No mocking needed — just construct an engine, push inputs, assert outputs.
//! - **Portable**: Runs anywhere Rust compiles — embed in a server, WASM, or a test harness.
//! - **Deterministic**: Same inputs always produce the same outputs (no internal randomness).

#![doc(html_root_url = "https://docs.rs/lurk-sansio/1.0.3")]
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![cfg_attr(docsrs, allow(internal_features))]
// Ignored clippy and clippy_pedantic lints
#![allow(
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/7768
    clippy::semicolon_if_nothing_returned,
    clippy::type_repetition_in_bounds, // https://github.com/rust-lang/rust-clippy/issues/8772
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    // things are often more readable this way
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::type_complexity,
    clippy::use_self,
    clippy::zero_prefixed_literal,
    // correctly used
    clippy::derive_partial_eq_without_eq,
    clippy::enum_glob_use,
    clippy::explicit_auto_deref,
    clippy::incompatible_msrv,
    clippy::let_underscore_untyped,
    clippy::map_err_ignore,
    clippy::new_without_default,
    clippy::result_unit_err,
    clippy::wildcard_imports,
    // not practical
    clippy::needless_pass_by_value,
    clippy::similar_names,
    clippy::too_many_lines,
    // preference
    clippy::doc_markdown,
    clippy::needless_lifetimes,
    clippy::unseparated_literal_suffix,
    // false positive
    clippy::needless_doctest_main,
    // noisy
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
)]
// Rustc lints.
#![deny(missing_docs, unused_imports)]

/// The core game engine: accepts [`Input`] events, mutates state, produces [`Output`] events.
pub mod engine;
mod handlers;
/// Events fed into the engine by the transport layer.
pub mod input;
/// Side-effects produced by the engine for the transport layer to execute.
pub mod output;
/// Game world model: characters, rooms, monsters, and configuration.
pub mod state;
/// Protocol-level type re-exports from [`lurk_lcsc`].
pub mod types;

pub use engine::GameEngine;
pub use input::Input;
pub use output::{ConnectionInfo, Output, RoomInfo};
pub use state::{Character, Connection, GameConfig, Monster, PlayerState, Room};
pub use types::ClientId;
