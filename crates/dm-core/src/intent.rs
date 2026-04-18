//! Intent — what an actor wants to do.
//!
//! Phase 1 resolves only `Attack` and `EndTurn` in `combat.rs`. The other
//! variants are stubs: the CLI game loop routes them to placeholder handlers
//! today; Phase 2 wires them fully (ability check DC, spell slots, NPC
//! dialogue). Schema uses `serde(tag = "type")` to stay compatible with the
//! Claude tool-use JSON schema that Phase 2 will introduce.

use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intent {
    /// Melee or ranged attack. Needs explicit `target` + `attack_idx` — these
    /// are *not* inferable from a Python intent label alone (see ADR: blind
    /// spot 2 — disambiguation gap).
    Attack {
        attacker: EntityId,
        target: EntityId,
        attack_idx: usize,
    },
    /// d20 ability/skill check vs. a DC.  `skill` and `dc` resolved by the
    /// rules engine in Phase 2; optional here so Phase 1 can stub with a
    /// default DC and the actor's initiative modifier as a proxy.
    AbilityCheck {
        actor: EntityId,
        skill: Option<String>,
        dc: Option<i32>,
    },
    /// Spell cast. `spell_name` drives lookup in the spells table (Phase 2).
    CastSpell {
        actor: EntityId,
        spell_name: Option<String>,
        target: Option<EntityId>,
    },
    /// Narrative action — no mechanical effect in Phase 1. Phase 2 feeds this
    /// to the Narrative Agent for NPC response / disposition shift.
    Roleplay {
        actor: EntityId,
        text: String,
    },
    EndTurn {
        actor: EntityId,
    },
}
