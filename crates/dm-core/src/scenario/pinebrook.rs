//! Peril in Pinebrook — Encounter 2: Living Icicles.
//!
//! 4 pregen PCs (Shalefire/Noorah/Gallantine/Evandon) vs 5 Living Icicles.
//! Stat blocks come straight from the adventure text — see
//! `design/adventure-pinebrook.md`.

use crate::entity::{Attack, Creature, DamageType, EntityId, Side};

fn pc_shalefire() -> Creature {
    Creature {
        id: EntityId(1),
        name: "Shalefire Stoutheart".into(),
        side: Side::Pcs,
        ac: 16,
        hp: 13,
        max_hp: 13,
        initiative_bonus: 1,
        attacks: vec![Attack {
            name: "Handaxe".into(),
            attack_bonus: 6,
            damage_dice: "1d6+4".into(),
            damage_type: DamageType::Slashing,
        }],
    }
}

fn pc_noorah() -> Creature {
    Creature {
        id: EntityId(2),
        name: "Noorah Eldenfield".into(),
        side: Side::Pcs,
        ac: 14,
        hp: 11,
        max_hp: 11,
        initiative_bonus: 3,
        attacks: vec![Attack {
            name: "Shortsword".into(),
            attack_bonus: 5,
            damage_dice: "1d6+3".into(),
            damage_type: DamageType::Piercing,
        }],
    }
}

fn pc_gallantine() -> Creature {
    Creature {
        id: EntityId(3),
        name: "Gallantine Birchenbough".into(),
        side: Side::Pcs,
        ac: 12,
        hp: 9,
        max_hp: 9,
        initiative_bonus: 2,
        attacks: vec![Attack {
            name: "Fire Bolt".into(),
            attack_bonus: 5,
            // Adventure pre-rolls Fire Bolt at a flat 7 fire damage.
            damage_dice: "7".into(),
            damage_type: DamageType::Fire,
        }],
    }
}

fn pc_evandon() -> Creature {
    Creature {
        id: EntityId(4),
        name: "Evandon Haart".into(),
        side: Side::Pcs,
        ac: 14,
        hp: 11,
        max_hp: 11,
        initiative_bonus: 1,
        attacks: vec![Attack {
            name: "Mace".into(),
            attack_bonus: 5,
            damage_dice: "1d6+3".into(),
            damage_type: DamageType::Bludgeoning,
        }],
    }
}

fn living_icicle(id: u32, n: u32) -> Creature {
    Creature {
        id: EntityId(id),
        name: format!("Living Icicle {}", n),
        side: Side::Enemies,
        ac: 10,
        hp: 7,
        max_hp: 7,
        initiative_bonus: 0,
        attacks: vec![Attack {
            name: "Claws".into(),
            attack_bonus: 2,
            damage_dice: "1d6".into(),
            damage_type: DamageType::Cold,
        }],
    }
}

pub fn default_creatures() -> Vec<Creature> {
    vec![
        pc_shalefire(),
        pc_noorah(),
        pc_gallantine(),
        pc_evandon(),
        living_icicle(101, 1),
        living_icicle(102, 2),
        living_icicle(103, 3),
        living_icicle(104, 4),
        living_icicle(105, 5),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_4_pcs_and_5_enemies() {
        let cs = default_creatures();
        assert_eq!(cs.iter().filter(|c| c.side == Side::Pcs).count(), 4);
        assert_eq!(cs.iter().filter(|c| c.side == Side::Enemies).count(), 5);
    }

    #[test]
    fn all_ids_unique() {
        let cs = default_creatures();
        let mut ids: Vec<_> = cs.iter().map(|c| c.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 9);
    }

    #[test]
    fn all_damage_expressions_parse() {
        for c in default_creatures() {
            for a in &c.attacks {
                a.damage_roll()
                    .unwrap_or_else(|e| panic!("{} bad damage dice: {:?}", a.name, e));
            }
        }
    }
}
