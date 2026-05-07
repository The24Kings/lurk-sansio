use std::sync::Arc;

use lurk_lcsc::LurkError;

use crate::engine::GameEngine;
use crate::output::Output;
use crate::types::ClientId;

impl GameEngine {
    pub(crate) fn handle_fight(&mut self, client: ClientId) {
        // Find the player and extract needed data
        let (attacker_name, current_room) = {
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

        // Collect all players that will join in battle (those with BATTLE flag in same room)
        let in_battle: Vec<Arc<str>> = self
            .players
            .iter()
            .filter(|(_, ps)| {
                ps.character.flags.is_battle() && ps.character.current_room == current_room
            })
            .map(|(name, _)| name.clone())
            .collect();

        // Find the target monster (alive, lowest health)
        let target_idx = {
            let Some(monsters) = self
                .rooms
                .get(&current_room)
                .and_then(|r| r.monsters.as_ref())
            else {
                self.emit(Output::SendError {
                    client,
                    error_code: LurkError::NOFIGHT,
                    message: "The room is eerily quiet...".into(),
                });
                return;
            };

            let Some((idx, _)) = monsters
                .iter()
                .enumerate()
                .filter(|(_, m)| m.health > 0)
                .min_by_key(|(_, m)| (m.health, m.name.clone()))
            else {
                self.emit(Output::SendError {
                    client,
                    error_code: LurkError::NOFIGHT,
                    message: "No monsters alive. Let them rest.".into(),
                });
                return;
            };

            idx
        };

        let target_name = self.rooms[&current_room]
            .monsters
            .as_ref()
            .expect("monsters confirmed present above")[target_idx]
            .name
            .clone();

        // Narrate the attack
        self.emit(Output::Narrate {
            room_number: current_room,
            message: format!("{} is attacking {}", attacker_name, target_name).into(),
            narration: false,
        });

        self.emit(Output::Narrate {
            room_number: current_room,
            message: format!("{} player(s) joining the fight", in_battle.len() - 1).into(),
            narration: false,
        });

        // OFFENSE PHASE: Calculate total battle damage
        let battle_damage: u16 = in_battle
            .iter()
            .filter_map(|name| self.players.get(name))
            .map(|ps| ps.character.attack)
            .sum();

        // Apply damage to monster
        let monster = &mut self
            .rooms
            .get_mut(&current_room)
            .expect("room confirmed")
            .monsters
            .as_mut()
            .expect("monsters confirmed")[target_idx];

        let damage = battle_damage.saturating_sub(monster.defense);
        let damage: i16 = damage.try_into().unwrap_or(i16::MAX);
        monster.health = monster.health.saturating_sub(damage);

        let victory = monster.health <= 0;

        // DEFENSE PHASE: Monster counterattack (if not victory)
        let monster_attack = monster.attack;

        if !victory && let Some(ps) = self.players.get_mut(&attacker_name) {
            let counter_damage = monster_attack.saturating_sub(ps.character.defense);
            let counter_damage: i16 = counter_damage.try_into().unwrap_or(i16::MAX);
            ps.character.health = ps.character.health.saturating_sub(counter_damage);
        }

        // REGEN PHASE: Attacker regenerates (if alive)
        if let Some(ps) = self.players.get_mut(&attacker_name)
            && ps.character.flags.is_alive()
        {
            let regen: i16 = ps.character.regen.try_into().unwrap_or(i16::MAX);
            ps.character.health = ps.character.health.saturating_add(regen);
        }

        // RESOLUTION: Alert room with updated characters
        // Alert all battle participants
        for name in &in_battle {
            if let Some(ps) = self.players.get(name) {
                self.outputs.push_back(Output::AlertRoom {
                    room_number: current_room,
                    character: ps.character.clone(),
                });
            }
        }

        // Alert room with updated monster
        let monster_character = self.rooms[&current_room]
            .monsters
            .as_ref()
            .expect("monsters confirmed")[target_idx]
            .to_character();

        self.emit(Output::AlertRoom {
            room_number: current_room,
            character: monster_character,
        });
    }
}
