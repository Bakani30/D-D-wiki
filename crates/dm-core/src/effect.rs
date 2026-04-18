//! Effect — the deterministic outcome of resolving an Intent.
//!
//! Every variant carries enough data for the narrator (or CLI) to render a
//! dice-visible line. Names are embedded so `narrative()` needs no lookup
//! table — this keeps the Effect stream self-describing for logs/replay.

use serde::{Deserialize, Serialize};

use crate::dice::RollResult;
use crate::entity::{DamageType, EntityId, Side};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageApplied {
    pub roll: RollResult,
    pub damage_type: DamageType,
    pub hp_before: i32,
    pub hp_after: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    InitiativeRolled {
        order: Vec<InitiativeEntry>,
    },
    AttackResolved {
        attacker: EntityId,
        attacker_name: String,
        target: EntityId,
        target_name: String,
        attack_name: String,
        attack_roll: RollResult,
        target_ac: i32,
        natural_d20: u32,
        crit: bool,
        hit: bool,
        damage: Option<DamageApplied>,
    },
    CreatureDowned {
        entity: EntityId,
        name: String,
    },
    TurnEnded {
        actor: EntityId,
        actor_name: String,
    },
    RoundStarted {
        round: u32,
    },
    EncounterEnded {
        winner: Side,
        rounds: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeEntry {
    pub entity: EntityId,
    pub name: String,
    pub total: i32,
    pub natural_d20: u32,
}

impl Effect {
    pub fn narrative(&self) -> String {
        match self {
            Effect::InitiativeRolled { order } => {
                let lines: Vec<String> = order
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        format!(
                            "  {}. {} (nat {}, total {})",
                            i + 1,
                            e.name,
                            e.natural_d20,
                            e.total
                        )
                    })
                    .collect();
                format!("Initiative order:\n{}", lines.join("\n"))
            }
            Effect::AttackResolved {
                attacker_name,
                target_name,
                attack_name,
                attack_roll,
                target_ac,
                crit,
                hit,
                damage,
                ..
            } => {
                let verdict = match (hit, crit) {
                    (true, true) => "CRIT",
                    (true, false) => "hit",
                    (false, _) => "miss",
                };
                let mut s = format!(
                    "{} uses {} on {} — {} vs AC {} → {}",
                    attacker_name,
                    attack_name,
                    target_name,
                    attack_roll.narrative(),
                    target_ac,
                    verdict
                );
                if let Some(d) = damage {
                    s.push_str(&format!(
                        " — damage {} {} (HP {} → {})",
                        d.roll.narrative(),
                        damage_type_label(d.damage_type),
                        d.hp_before,
                        d.hp_after
                    ));
                }
                s
            }
            Effect::CreatureDowned { name, .. } => format!("{} is down.", name),
            Effect::TurnEnded { actor_name, .. } => format!("{} ends their turn.", actor_name),
            Effect::RoundStarted { round } => format!("— Round {} —", round),
            Effect::EncounterEnded { winner, rounds } => format!(
                "Encounter ends after {} round(s). Winner: {:?}.",
                rounds, winner
            ),
        }
    }
}

fn damage_type_label(t: DamageType) -> &'static str {
    match t {
        DamageType::Bludgeoning => "bludgeoning",
        DamageType::Piercing => "piercing",
        DamageType::Slashing => "slashing",
        DamageType::Acid => "acid",
        DamageType::Cold => "cold",
        DamageType::Fire => "fire",
        DamageType::Lightning => "lightning",
        DamageType::Thunder => "thunder",
        DamageType::Poison => "poison",
        DamageType::Psychic => "psychic",
        DamageType::Necrotic => "necrotic",
        DamageType::Radiant => "radiant",
        DamageType::Force => "force",
    }
}
