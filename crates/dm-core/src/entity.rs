//! Creatures — unified PC + monster shape for Prototype 1.
//!
//! No resistances, no full condition set, no positioning. Just the fields the
//! combat loop needs: side, AC, HP, initiative bonus, a list of attacks.

use serde::{Deserialize, Serialize};

use crate::dice::{DiceError, Roll};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct EntityId(pub u32);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Pcs,
    Enemies,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Pcs => Side::Enemies,
            Side::Enemies => Side::Pcs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
    Acid,
    Cold,
    Fire,
    Lightning,
    Thunder,
    Poison,
    Psychic,
    Necrotic,
    Radiant,
    Force,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attack {
    pub name: String,
    pub attack_bonus: i32,
    /// Dice expression, e.g. `"1d6+4"`. Parsed on each use via `Roll::from_str`.
    pub damage_dice: String,
    pub damage_type: DamageType,
}

impl Attack {
    pub fn damage_roll(&self) -> Result<Roll, DiceError> {
        self.damage_dice.parse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Creature {
    pub id: EntityId,
    pub name: String,
    pub side: Side,
    pub ac: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub initiative_bonus: i32,
    pub attacks: Vec<Attack>,
}

impl Creature {
    pub fn is_down(&self) -> bool {
        self.hp <= 0
    }

    /// Subtract damage from HP, saturating at `i32::MIN` to avoid overflow.
    /// Returns the new HP.
    pub fn take_damage(&mut self, amount: i32) -> i32 {
        self.hp = self.hp.saturating_sub(amount);
        self.hp
    }
}
