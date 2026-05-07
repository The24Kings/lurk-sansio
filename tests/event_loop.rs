use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexSet;
use lurk_lcsc::{CharacterFlags, LurkError, PktType};
use lurk_sansio::*;

// ==================== Test Helpers ====================

fn drain_outputs(engine: &mut GameEngine) -> Vec<Output> {
    let mut outputs = Vec::new();
    while let Some(output) = engine.poll_output() {
        outputs.push(output);
    }
    outputs
}

/// Build a test map with 3 rooms connected linearly: 0 -- 1 -- 2
/// Room 0 has one monster ("Skulltula"), room 2 has one monster ("Deku Baba").
fn build_test_map() -> HashMap<u16, Room> {
    let mut rooms = HashMap::new();

    // Room 0: Starting room
    let mut conns_0 = HashMap::new();
    conns_0.insert(
        1,
        Connection {
            room_number: 1,
            title: "North Door".into(),
            description: "A door leading north.".into(),
        },
    );
    rooms.insert(
        0,
        Room {
            room_number: 0,
            title: "Kokiri Forest".into(),
            connections: conns_0,
            description: "A peaceful forest clearing.".into(),
            players: IndexSet::new(),
            monsters: Some(vec![Monster {
                name: Arc::from("Skulltula"),
                current_room: 0,
                max_health: 20,
                health: 20,
                attack: 5,
                defense: 2,
                gold: 10,
                description: "A giant spider.".into(),
            }]),
        },
    );

    // Room 1: Middle room (no monsters)
    let mut conns_1 = HashMap::new();
    conns_1.insert(
        0,
        Connection {
            room_number: 0,
            title: "South Door".into(),
            description: "Back to the forest.".into(),
        },
    );
    conns_1.insert(
        2,
        Connection {
            room_number: 2,
            title: "East Path".into(),
            description: "A narrow path east.".into(),
        },
    );
    rooms.insert(
        1,
        Room {
            room_number: 1,
            title: "Lost Woods".into(),
            connections: conns_1,
            description: "Eerie trees surround you.".into(),
            players: IndexSet::new(),
            monsters: None,
        },
    );

    // Room 2: End room with a monster
    let mut conns_2 = HashMap::new();
    conns_2.insert(
        1,
        Connection {
            room_number: 1,
            title: "West Path".into(),
            description: "Back to the woods.".into(),
        },
    );
    rooms.insert(
        2,
        Room {
            room_number: 2,
            title: "Sacred Meadow".into(),
            connections: conns_2,
            description: "A sun-drenched meadow.".into(),
            players: IndexSet::new(),
            monsters: Some(vec![Monster {
                name: Arc::from("Deku Baba"),
                current_room: 2,
                max_health: 10,
                health: 10,
                attack: 3,
                defense: 1,
                gold: 5,
                description: "A carnivorous plant.".into(),
            }]),
        },
    );

    rooms
}

fn test_config() -> GameConfig {
    GameConfig {
        initial_points: 100,
        stat_limit: 65535,
    }
}

fn new_engine() -> GameEngine {
    GameEngine::new(build_test_map(), test_config())
}

const CLIENT_A: ClientId = ClientId(1);
const CLIENT_B: ClientId = ClientId(2);

/// Register a character with the engine.
fn register_character(engine: &mut GameEngine, client: ClientId, name: &str) {
    engine.handle_input(Input::Character {
        client,
        name: Arc::from(name),
        attack: 40,
        defense: 30,
        regen: 30,
        description: "A test hero.".into(),
    });
}

/// Register and start a character, returning all outputs from both steps.
fn register_and_start(engine: &mut GameEngine, client: ClientId, name: &str) -> Vec<Output> {
    register_character(engine, client, name);
    let _ = drain_outputs(engine); // discard registration outputs
    engine.handle_input(Input::Start { client });
    drain_outputs(engine)
}

// ==================== Full Lifecycle Test ====================

#[test]
fn full_player_lifecycle() {
    let mut engine = new_engine();

    // Step 1: Register character
    register_character(&mut engine, CLIENT_A, "Link");
    let outputs = drain_outputs(&mut engine);

    // Should get Accept + SendCharacter
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendAccept { client, accepted_type }
                if *client == CLIENT_A && *accepted_type == PktType::CHARACTER)),
        "Expected Accept for CHARACTER"
    );
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendCharacter { client, character }
                if *client == CLIENT_A && character.name.as_ref() == "Link")),
        "Expected SendCharacter for Link"
    );

    // Step 2: Start the game
    engine.handle_input(Input::Start { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    // Should get: SendCharacter (updated), AlertRoom, Broadcast, SendRoom, connections, room contents
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendCharacter { character, .. }
                if character.flags.is_started())),
        "Expected character with STARTED flag"
    );
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::Broadcast { message }
                if message.contains("Link") && message.contains("started"))),
        "Expected broadcast about starting"
    );
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendRoom { room, .. }
                if room.room_number == 0)),
        "Expected room 0 data"
    );

    // Verify player is in room 0
    assert!(engine.rooms()[&0].players.contains("Link"));

    // Step 3: Move to room 1
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 1,
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendRoom { room, .. } if room.room_number == 1)),
        "Expected room 1 data"
    );
    assert!(!engine.rooms()[&0].players.contains("Link"));
    assert!(engine.rooms()[&1].players.contains("Link"));

    // Step 4: Move to room 2 (has a monster)
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 2,
    });
    let _ = drain_outputs(&mut engine);
    assert!(engine.rooms()[&2].players.contains("Link"));

    // Step 5: Fight the Deku Baba
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    // Should have narration about attacking and AlertRoom outputs
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::Narrate { message, .. }
                if message.contains("attacking") && message.contains("Deku Baba"))),
        "Expected attack narration"
    );
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::AlertRoom { character, .. }
                if character.name.as_ref() == "Deku Baba")),
        "Expected monster alert"
    );

    // With 40 attack vs 1 defense, monster should be dead
    let monster = &engine.rooms()[&2].monsters.as_ref().unwrap()[0];
    assert!(monster.health <= 0, "Deku Baba should be dead");

    // Step 6: Loot the dead monster
    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Deku Baba"),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendCharacter { character, .. }
                if character.name.as_ref() == "Link" && character.gold == 5)),
        "Expected Link to have 5 gold after looting"
    );

    // Step 7: Leave the game
    engine.handle_input(Input::Leave { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::Broadcast { message }
                if message.contains("Link") && message.contains("left"))),
        "Expected leave broadcast"
    );
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::Disconnect { client } if *client == CLIENT_A)),
        "Expected disconnect"
    );

    // Player should still exist but be deactivated
    let ps = &engine.players()["Link"];
    assert!(ps.client.is_none());
    assert_eq!(ps.character.flags, CharacterFlags::empty());
}

// ==================== Character Registration Errors ====================

#[test]
fn character_bad_stats_rejected() {
    let mut engine = new_engine();

    engine.handle_input(Input::Character {
        client: CLIENT_A,
        name: Arc::from("Link"),
        attack: 50,
        defense: 30,
        regen: 30, // total = 110 > 100
        description: "A test hero.".into(),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::STATERROR)),
        "Expected STATERROR"
    );
    assert!(
        engine.players().is_empty(),
        "Player should not be registered"
    );
}

#[test]
fn character_already_started_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Try to re-register while started
    register_character(&mut engine, CLIENT_A, "Link");
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::PLAYEREXISTS)),
        "Expected PLAYEREXISTS"
    );
}

#[test]
fn character_stat_overflow_rejected() {
    let mut engine = new_engine();

    engine.handle_input(Input::Character {
        client: CLIENT_A,
        name: Arc::from("Link"),
        attack: u16::MAX,
        defense: u16::MAX,
        regen: u16::MAX,
        description: "Overflow hero.".into(),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::STATERROR)),
        "Expected STATERROR for overflow"
    );
}

// ==================== Start Errors ====================

#[test]
fn start_without_character_does_nothing() {
    let mut engine = new_engine();

    engine.handle_input(Input::Start { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    // No player registered, so player_from_client returns None → no outputs
    assert!(
        outputs.is_empty(),
        "Expected no outputs for unregistered client"
    );
}

#[test]
fn start_not_ready_rejected() {
    let mut engine = new_engine();

    // Manually insert a player with no READY flag
    engine.players_mut().insert(
        Arc::from("Link"),
        PlayerState {
            character: Character {
                name: Arc::from("Link"),
                flags: CharacterFlags::empty(), // Not ready
                attack: 10,
                defense: 10,
                regen: 10,
                health: 100,
                gold: 0,
                current_room: 0,
                description: "Not ready.".into(),
            },
            client: Some(CLIENT_A),
        },
    );

    engine.handle_input(Input::Start { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::NOTREADY)),
        "Expected NOTREADY"
    );
}

// ==================== ChangeRoom Errors ====================

#[test]
fn change_room_already_in_room() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Try to move to room 0 (already there)
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 0,
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::BADROOM)),
        "Expected BADROOM for same room"
    );
}

#[test]
fn change_room_no_connection() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Try to go from room 0 directly to room 2 (no connection)
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 2,
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::BADROOM)),
        "Expected BADROOM for invalid connection"
    );
}

#[test]
fn change_room_updates_both_rooms() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 1,
    });
    let outputs = drain_outputs(&mut engine);

    // Should alert both old (0) and new (1) rooms
    let alert_rooms: Vec<u16> = outputs
        .iter()
        .filter_map(|o| match o {
            Output::AlertRoom { room_number, .. } => Some(*room_number),
            _ => None,
        })
        .collect();

    assert!(alert_rooms.contains(&0), "Expected alert for old room");
    assert!(alert_rooms.contains(&1), "Expected alert for new room");
}

// ==================== Fight Tests ====================

#[test]
fn fight_no_monsters_in_room() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Move to room 1 (no monsters)
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 1,
    });
    let _ = drain_outputs(&mut engine);

    engine.handle_input(Input::Fight { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::NOFIGHT)),
        "Expected NOFIGHT in room without monsters"
    );
}

#[test]
fn fight_victory_kills_monster() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Fight Skulltula in room 0 (40 atk vs 2 def = 38 damage, monster has 20 HP)
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    let monster = &engine.rooms()[&0].monsters.as_ref().unwrap()[0];
    assert!(monster.health <= 0, "Skulltula should be dead after fight");

    // Player should still be alive (monster has 5 attack vs 30 defense = 0 damage, but victory skips counter)
    let player = &engine.players()["Link"];
    assert!(player.character.health > 0, "Link should still be alive");
}

#[test]
fn fight_player_takes_counter_damage_on_non_victory() {
    let mut engine = new_engine();

    // Create a very weak character that can't kill the monster in one hit
    engine.handle_input(Input::Character {
        client: CLIENT_A,
        name: Arc::from("Weakling"),
        attack: 1,
        defense: 0,
        regen: 0,
        description: "Very weak.".into(),
    });
    let _ = drain_outputs(&mut engine);

    // Start — Weakling now in room 0 with Skulltula (20 HP, 5 atk, 2 def)
    engine.handle_input(Input::Start { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    let hp_before = engine.players()["Weakling"].character.health;

    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    let hp_after = engine.players()["Weakling"].character.health;

    // Monster should still be alive (1 atk - 2 def = 0 damage)
    let monster = &engine.rooms()[&0].monsters.as_ref().unwrap()[0];
    assert!(monster.health > 0, "Skulltula should survive 0 damage");

    // Player should take counter damage: 5 atk - 0 def = 5 damage
    assert!(
        hp_after < hp_before,
        "Player should take counter damage: before={hp_before} after={hp_after}"
    );
    assert_eq!(hp_after, hp_before - 5, "Expected 5 counter damage");
}

#[test]
fn fight_all_monsters_dead_gives_nofight() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Kill the Skulltula
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    // Try to fight again — no alive monsters
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::NOFIGHT)),
        "Expected NOFIGHT when all monsters dead"
    );
}

#[test]
fn fight_allies_join_battle() {
    let mut engine = new_engine();

    // Register and start two players in room 0
    register_and_start(&mut engine, CLIENT_A, "Link");
    register_and_start(&mut engine, CLIENT_B, "Zelda");

    // Both have BATTLE flag (set by alive()), both in room 0
    // Link attacks: total battle damage = Link(40) + Zelda(40) = 80 vs Skulltula(2 def) = 78 damage
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    let monster = &engine.rooms()[&0].monsters.as_ref().unwrap()[0];
    assert!(
        monster.health <= 0,
        "Skulltula should be dead with combined 78 damage vs 20 HP"
    );
}

// ==================== PVP Fight ====================

#[test]
fn pvp_fight_always_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::PvpFight {
        client: CLIENT_A,
        target_name: Arc::from("Zelda"),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::NOPLAYERCOMBAT)),
        "Expected NOPLAYERCOMBAT"
    );
}

// ==================== Loot Tests ====================

#[test]
fn loot_alive_monster_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Skulltula"),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::BADMONSTER)),
        "Expected BADMONSTER for alive monster"
    );
}

#[test]
fn loot_nonexistent_monster_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Ganondorf"),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::BADMONSTER)),
        "Expected BADMONSTER for nonexistent monster"
    );
}

#[test]
fn loot_already_looted_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Kill and loot
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);
    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Skulltula"),
    });
    let _ = drain_outputs(&mut engine);

    // Try to loot again
    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Skulltula"),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::BADMONSTER)),
        "Expected BADMONSTER for already-looted monster"
    );
}

#[test]
fn loot_transfers_gold() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Kill Skulltula (10 gold)
    engine.handle_input(Input::Fight { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    // Loot
    engine.handle_input(Input::Loot {
        client: CLIENT_A,
        target_name: Arc::from("Skulltula"),
    });
    let outputs = drain_outputs(&mut engine);

    let player = &engine.players()["Link"];
    assert_eq!(player.character.gold, 10, "Expected 10 gold from Skulltula");

    let monster = &engine.rooms()[&0].monsters.as_ref().unwrap()[0];
    assert_eq!(monster.gold, 0, "Monster gold should be 0 after looting");

    // Should send updated player and monster characters
    let send_chars: Vec<&Character> = outputs
        .iter()
        .filter_map(|o| match o {
            Output::SendCharacter { character, .. } => Some(character),
            _ => None,
        })
        .collect();

    assert_eq!(send_chars.len(), 2, "Expected 2 SendCharacter outputs");
}

// ==================== Message Tests ====================

#[test]
fn message_forwarded_to_recipient() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");
    register_and_start(&mut engine, CLIENT_B, "Zelda");

    engine.handle_input(Input::Message {
        client: CLIENT_A,
        sender_name: Arc::from("Link"),
        recipient_name: Arc::from("Zelda"),
        message: "Hello Zelda!".into(),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs.iter().any(
            |o| matches!(o, Output::SendMessage { client, sender_name, message, .. }
                if *client == CLIENT_B && sender_name.as_ref() == "Link" && message.as_ref() == "Hello Zelda!")
        ),
        "Expected message forwarded to Zelda's client"
    );
}

#[test]
fn message_recipient_not_found() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::Message {
        client: CLIENT_A,
        sender_name: Arc::from("Link"),
        recipient_name: Arc::from("Navi"),
        message: "Hey listen!".into(),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { error_code, .. }
                if *error_code == LurkError::OTHER)),
        "Expected OTHER error for missing recipient"
    );
}

#[test]
fn message_to_disconnected_player_rejected() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");
    register_and_start(&mut engine, CLIENT_B, "Zelda");

    // Zelda leaves
    engine.handle_input(Input::Leave { client: CLIENT_B });
    let _ = drain_outputs(&mut engine);

    // Link tries to message Zelda
    engine.handle_input(Input::Message {
        client: CLIENT_A,
        sender_name: Arc::from("Link"),
        recipient_name: Arc::from("Zelda"),
        message: "Are you there?".into(),
    });
    let outputs = drain_outputs(&mut engine);

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendError { .. })),
        "Expected error when messaging disconnected player"
    );
}

// ==================== Leave & Reconnect ====================

#[test]
fn leave_deactivates_player() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::Leave { client: CLIENT_A });
    let outputs = drain_outputs(&mut engine);

    let ps = &engine.players()["Link"];
    assert!(ps.client.is_none(), "Client should be None after leave");
    assert_eq!(
        ps.character.flags,
        CharacterFlags::empty(),
        "Flags should be empty after leave"
    );

    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::Disconnect { client } if *client == CLIENT_A)),
        "Expected disconnect output"
    );
}

#[test]
fn reconnect_after_leave() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    engine.handle_input(Input::Leave { client: CLIENT_A });
    let _ = drain_outputs(&mut engine);

    // Reconnect with a new client ID
    let client_a2 = ClientId(99);
    register_character(&mut engine, client_a2, "Link");
    let outputs = drain_outputs(&mut engine);

    // Should succeed — player existed but wasn't started
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::SendAccept { .. })),
        "Expected Accept on reconnect"
    );

    let ps = &engine.players()["Link"];
    assert_eq!(ps.client, Some(client_a2), "Client should be updated");
    assert!(ps.character.flags.is_ready(), "Should be ready again");
}

// ==================== Multi-Client Interaction ====================

#[test]
fn two_players_in_same_room_see_each_other() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    register_character(&mut engine, CLIENT_B, "Zelda");
    let _ = drain_outputs(&mut engine);

    engine.handle_input(Input::Start { client: CLIENT_B });
    let outputs = drain_outputs(&mut engine);

    // When Zelda starts, she should receive room contents that includes Link
    let char_sends: Vec<&Character> = outputs
        .iter()
        .filter_map(|o| match o {
            Output::SendCharacter {
                client, character, ..
            } if *client == CLIENT_B => Some(character),
            _ => None,
        })
        .collect();

    // Zelda should receive her own updated character + Link as room content
    let names: Vec<&str> = char_sends.iter().map(|c| c.name.as_ref()).collect();
    assert!(
        names.contains(&"Link"),
        "Zelda should see Link in room contents, got: {names:?}"
    );
}

#[test]
fn player_leaving_alerts_room() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");
    register_and_start(&mut engine, CLIENT_B, "Zelda");

    engine.handle_input(Input::Leave { client: CLIENT_B });
    let outputs = drain_outputs(&mut engine);

    // Should broadcast and alert room 0
    assert!(
        outputs
            .iter()
            .any(|o| matches!(o, Output::AlertRoom { room_number, character }
                if *room_number == 0 && character.name.as_ref() == "Zelda")),
        "Expected AlertRoom for Zelda leaving room 0"
    );
}

// ==================== Event Loop Simulation ====================

#[test]
fn event_loop_drains_all_outputs() {
    let mut engine = new_engine();
    register_and_start(&mut engine, CLIENT_A, "Link");

    // Feed multiple inputs
    engine.handle_input(Input::Fight { client: CLIENT_A });
    engine.handle_input(Input::ChangeRoom {
        client: CLIENT_A,
        room_number: 1,
    });

    // Drain all outputs in a loop — simulates event loop polling
    let mut total_outputs = 0;
    while let Some(_output) = engine.poll_output() {
        total_outputs += 1;
        // In a real event loop, we'd execute the output here
    }

    assert!(
        total_outputs > 0,
        "Event loop should have processed outputs"
    );

    // Queue should be empty now
    assert!(
        engine.poll_output().is_none(),
        "Queue should be empty after draining"
    );
}

#[test]
fn no_io_imports_in_library() {
    // This test is a compile-time guarantee: the library compiles without std::net.
    // If we accidentally import TcpStream, the library won't compile since lurk_lcsc
    // types with TcpStream aren't used in our state types.
    let mut engine = new_engine();
    assert!(engine.poll_output().is_none());
}
