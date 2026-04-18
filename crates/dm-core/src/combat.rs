//! Combat — encounter state, initiative, and Intent → Effect resolution.
//!
//! Prototype 1 scope: attacks and turn ending. No movement, no saves, no
//! conditions beyond "down at 0 HP". Initiative is rolled once; the turn
//! order skips downed creatures.
//!
//! 5e 2024 attack crit rules: nat-20 on the d20 is an automatic hit AND a
//! crit (damage dice doubled, modifier unchanged); nat-1 is an automatic
//! miss. Checks (non-attack d20 tests) have no such rule — see `check.rs`.

use rand::Rng;
use thiserror::Error;

use crate::check::D20Test;
use crate::dice::{DiceTerm, Roll};
use crate::effect::{DamageApplied, Effect, InitiativeEntry};
use crate::entity::{Creature, EntityId, Side};
use crate::intent::Intent;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CombatError {
    #[error("unknown entity {0}")]
    UnknownEntity(EntityId),
    #[error("entity {0} has no attack at index {1}")]
    NoSuchAttack(EntityId, usize),
    #[error("damage dice parse error in attack: {0}")]
    BadDamageDice(String),
    #[error("attacker {0} is down")]
    AttackerDown(EntityId),
    #[error("initiative not yet rolled")]
    NoInitiative,
    #[error("intent actor {0} is not the current turn actor {1:?}")]
    NotYourTurn(EntityId, Option<EntityId>),
}

#[derive(Debug, Clone)]
pub struct Encounter {
    creatures: Vec<Creature>,
    initiative: Vec<EntityId>,
    turn_idx: usize,
    round: u32,
    ended: bool,
}

impl Encounter {
    pub fn new(creatures: Vec<Creature>) -> Self {
        Self {
            creatures,
            initiative: Vec::new(),
            turn_idx: 0,
            round: 0,
            ended: false,
        }
    }

    pub fn creatures(&self) -> &[Creature] {
        &self.creatures
    }

    pub fn initiative(&self) -> &[EntityId] {
        &self.initiative
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn is_ended(&self) -> bool {
        self.ended
    }

    pub fn find(&self, id: EntityId) -> Option<&Creature> {
        self.creatures.iter().find(|c| c.id == id)
    }

    fn find_mut(&mut self, id: EntityId) -> Option<&mut Creature> {
        self.creatures.iter_mut().find(|c| c.id == id)
    }

    pub fn roll_initiative<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Effect {
        let mut entries: Vec<InitiativeEntry> = self
            .creatures
            .iter()
            .map(|c| {
                let r = D20Test::new(c.initiative_bonus).execute(rng);
                InitiativeEntry {
                    entity: c.id,
                    name: c.name.clone(),
                    total: r.total,
                    natural_d20: r.natural_d20,
                }
            })
            .collect();
        entries.sort_by(|a, b| {
            b.total
                .cmp(&a.total)
                .then(b.natural_d20.cmp(&a.natural_d20))
                .then(a.entity.cmp(&b.entity))
        });
        self.initiative = entries.iter().map(|e| e.entity).collect();
        self.turn_idx = 0;
        self.round = 1;
        Effect::InitiativeRolled { order: entries }
    }

    /// Force a specific initiative order. Useful for tests and for encounters
    /// where surprise/ambush dictates order instead of a d20.
    pub fn force_initiative(&mut self, order: Vec<EntityId>) {
        self.initiative = order;
        self.turn_idx = 0;
        self.round = 1;
    }

    pub fn current_actor(&self) -> Option<EntityId> {
        if self.initiative.is_empty() {
            return None;
        }
        self.initiative.get(self.turn_idx).copied()
    }

    pub fn winner(&self) -> Option<Side> {
        let pcs_alive = self
            .creatures
            .iter()
            .any(|c| c.side == Side::Pcs && !c.is_down());
        let enemies_alive = self
            .creatures
            .iter()
            .any(|c| c.side == Side::Enemies && !c.is_down());
        match (pcs_alive, enemies_alive) {
            (true, false) => Some(Side::Pcs),
            (false, true) => Some(Side::Enemies),
            _ => None,
        }
    }

    pub fn resolve<R: Rng + ?Sized>(
        &mut self,
        intent: Intent,
        rng: &mut R,
    ) -> Result<Vec<Effect>, CombatError> {
        if self.ended {
            return Ok(Vec::new());
        }
        if self.initiative.is_empty() {
            return Err(CombatError::NoInitiative);
        }
        match intent {
            Intent::Attack { attacker, target, attack_idx } => {
                self.resolve_attack(attacker, target, attack_idx, rng)
            }
            Intent::EndTurn { actor } => self.resolve_end_turn(actor),
            // Non-combat intents (Phase 2): the combat resolver is not
            // responsible for ability checks, spells, or roleplay.  Return
            // an empty effect list; the orchestrator handles routing.
            _ => Ok(Vec::new()),
        }
    }

    fn resolve_attack<R: Rng + ?Sized>(
        &mut self,
        attacker_id: EntityId,
        target_id: EntityId,
        attack_idx: usize,
        rng: &mut R,
    ) -> Result<Vec<Effect>, CombatError> {
        let current = self.current_actor();
        if current != Some(attacker_id) {
            return Err(CombatError::NotYourTurn(attacker_id, current));
        }

        let attacker = self
            .find(attacker_id)
            .ok_or(CombatError::UnknownEntity(attacker_id))?;
        if attacker.is_down() {
            return Err(CombatError::AttackerDown(attacker_id));
        }
        let attacker_name = attacker.name.clone();
        let attack = attacker
            .attacks
            .get(attack_idx)
            .ok_or(CombatError::NoSuchAttack(attacker_id, attack_idx))?
            .clone();

        let target = self
            .find(target_id)
            .ok_or(CombatError::UnknownEntity(target_id))?;
        let target_name = target.name.clone();
        let target_ac = target.ac;
        let hp_before = target.hp;

        let d20 = D20Test::new(attack.attack_bonus).execute(rng);
        let natural_d20 = d20.natural_d20;
        let crit = natural_d20 == 20;
        let auto_miss = natural_d20 == 1;
        let hit = !auto_miss && (crit || d20.total >= target_ac);

        let mut effects = Vec::new();

        let damage_effect = if hit {
            let base = attack
                .damage_roll()
                .map_err(|_| CombatError::BadDamageDice(attack.damage_dice.clone()))?;
            let damage_roll = if crit { double_dice_on_crit(&base) } else { base };
            let dmg_result = damage_roll.execute(rng);
            let dmg = dmg_result.total.max(0);
            let target_mut = self
                .find_mut(target_id)
                .expect("target existed a moment ago");
            target_mut.take_damage(dmg);
            let hp_after = target_mut.hp;
            Some(DamageApplied {
                roll: dmg_result,
                damage_type: attack.damage_type,
                hp_before,
                hp_after,
            })
        } else {
            None
        };

        let hp_after = damage_effect.as_ref().map(|d| d.hp_after).unwrap_or(hp_before);

        effects.push(Effect::AttackResolved {
            attacker: attacker_id,
            attacker_name,
            target: target_id,
            target_name: target_name.clone(),
            attack_name: attack.name.clone(),
            attack_roll: d20.roll,
            target_ac,
            natural_d20,
            crit,
            hit,
            damage: damage_effect,
        });

        if hp_after <= 0 && hp_before > 0 {
            effects.push(Effect::CreatureDowned {
                entity: target_id,
                name: target_name,
            });
        }

        if let Some(winner) = self.winner() {
            self.ended = true;
            effects.push(Effect::EncounterEnded { winner, rounds: self.round });
        }

        Ok(effects)
    }

    fn resolve_end_turn(&mut self, actor: EntityId) -> Result<Vec<Effect>, CombatError> {
        let current = self.current_actor();
        if current != Some(actor) {
            return Err(CombatError::NotYourTurn(actor, current));
        }
        let actor_name = self
            .find(actor)
            .ok_or(CombatError::UnknownEntity(actor))?
            .name
            .clone();
        let mut effects = vec![Effect::TurnEnded { actor, actor_name }];
        if let Some(round_effect) = self.advance_turn() {
            effects.push(round_effect);
        }
        if !self.ended {
            if let Some(winner) = self.winner() {
                self.ended = true;
                effects.push(Effect::EncounterEnded { winner, rounds: self.round });
            }
        }
        Ok(effects)
    }

    /// Advance `turn_idx` past downed creatures. Returns a `RoundStarted`
    /// effect when the index wraps to a new round.
    fn advance_turn(&mut self) -> Option<Effect> {
        if self.initiative.is_empty() {
            return None;
        }
        let max_steps = self.initiative.len() * 2 + 2;
        let mut round_effect = None;
        for _ in 0..max_steps {
            self.turn_idx += 1;
            if self.turn_idx >= self.initiative.len() {
                self.turn_idx = 0;
                self.round = self.round.saturating_add(1);
                round_effect = Some(Effect::RoundStarted { round: self.round });
            }
            if let Some(id) = self.current_actor() {
                if let Some(c) = self.find(id) {
                    if !c.is_down() {
                        return round_effect;
                    }
                }
            }
        }
        round_effect
    }
}

fn double_dice_on_crit(roll: &Roll) -> Roll {
    let terms: Vec<DiceTerm> = roll
        .terms()
        .iter()
        .map(|t| match *t {
            DiceTerm::Dice { count, sides, sign } => DiceTerm::Dice {
                count: count.saturating_mul(2),
                sides,
                sign,
            },
            DiceTerm::Modifier { value } => DiceTerm::Modifier { value },
        })
        .collect();
    Roll::new(terms).expect("doubling dice preserves validity for realistic damage")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Attack, DamageType};
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn pc(id: u32, hp: i32, ac: i32, atk: i32, dice: &str) -> Creature {
        Creature {
            id: EntityId(id),
            name: format!("PC{}", id),
            side: Side::Pcs,
            ac,
            hp,
            max_hp: hp,
            initiative_bonus: 0,
            attacks: vec![Attack {
                name: "Hit".into(),
                attack_bonus: atk,
                damage_dice: dice.into(),
                damage_type: DamageType::Slashing,
            }],
        }
    }

    fn enemy(id: u32, hp: i32, ac: i32, atk: i32, dice: &str) -> Creature {
        Creature {
            side: Side::Enemies,
            ..pc(id, hp, ac, atk, dice)
        }
    }

    fn two_sided(hp_a: i32, ac_b: i32, atk: i32, dice: &str) -> Encounter {
        let mut e = Encounter::new(vec![
            pc(1, hp_a, 10, atk, dice),
            enemy(2, i32::MAX / 4, ac_b, 0, "1d4"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2)]);
        e
    }

    fn attack_one(
        e: &mut Encounter,
        rng: &mut StdRng,
    ) -> (bool, bool, u32, Option<DamageApplied>) {
        let fx = e
            .resolve(
                Intent::Attack {
                    attacker: EntityId(1),
                    target: EntityId(2),
                    attack_idx: 0,
                },
                rng,
            )
            .unwrap();
        match fx.into_iter().next().unwrap() {
            Effect::AttackResolved { hit, crit, natural_d20, damage, .. } => {
                (hit, crit, natural_d20, damage)
            }
            other => panic!("expected AttackResolved, got {:?}", other),
        }
    }

    #[test]
    fn huge_bonus_always_hits_except_nat_1() {
        let mut e = two_sided(20, 10, 100, "1d4");
        let mut rng = StdRng::seed_from_u64(1);
        let mut saw_non_nat_1 = false;
        for _ in 0..200 {
            let (hit, _, nat, _) = attack_one(&mut e, &mut rng);
            if nat != 1 {
                saw_non_nat_1 = true;
                assert!(hit, "bonus +100 should hit when nat != 1");
            }
        }
        assert!(saw_non_nat_1);
    }

    #[test]
    fn nat_1_auto_misses_even_with_huge_bonus() {
        let mut e = two_sided(20, 10, 100, "1d4");
        let mut rng = StdRng::seed_from_u64(7);
        let mut saw_nat_1 = false;
        for _ in 0..4000 {
            let (hit, crit, nat, _) = attack_one(&mut e, &mut rng);
            if nat == 1 {
                saw_nat_1 = true;
                assert!(!hit, "nat 1 should auto-miss");
                assert!(!crit);
            }
        }
        assert!(saw_nat_1);
    }

    #[test]
    fn nat_20_auto_hits_even_with_negative_bonus() {
        // bonus -30 vs AC 10: total always <= -10 < 10, so only nat-20 can hit.
        let mut e = two_sided(20, 10, -30, "1d4");
        let mut rng = StdRng::seed_from_u64(3);
        let mut saw_nat_20 = false;
        for _ in 0..4000 {
            let (hit, crit, nat, _) = attack_one(&mut e, &mut rng);
            if nat == 20 {
                saw_nat_20 = true;
                assert!(hit);
                assert!(crit);
            } else {
                assert!(!hit, "non-20 total below AC should miss");
            }
        }
        assert!(saw_nat_20);
    }

    #[test]
    fn crit_doubles_damage_dice_floor() {
        // 1d4+0 damage; on crit becomes 2d4+0, so damage in [2, 8] instead of [1, 4].
        // Force crit path: nat-30 bonus is irrelevant — we only care about seeds
        // where nat == 20. Check the damage is >= 2 on every crit.
        let mut e = two_sided(20, 10, -30, "1d4");
        let mut rng = StdRng::seed_from_u64(5);
        let mut saw_crit_damage = false;
        for _ in 0..4000 {
            let (hit, crit, _, damage) = attack_one(&mut e, &mut rng);
            if crit {
                assert!(hit);
                let d = damage.expect("crit hits deal damage");
                saw_crit_damage = true;
                assert!(
                    d.roll.total >= 2 && d.roll.total <= 8,
                    "crit 2d4 out of bounds: {}",
                    d.roll.total
                );
            }
        }
        assert!(saw_crit_damage);
    }

    #[test]
    fn damage_reduces_hp() {
        let mut e = Encounter::new(vec![
            pc(1, 20, 10, 100, "1d4"),
            enemy(2, 5, 10, 0, "1d1"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2)]);
        let mut rng = StdRng::seed_from_u64(42);
        let hp_before = e.find(EntityId(2)).unwrap().hp;
        let _ = attack_one(&mut e, &mut rng);
        let hp_after = e.find(EntityId(2)).unwrap().hp;
        assert!(hp_after < hp_before);
    }

    #[test]
    fn creature_downed_and_encounter_ends() {
        // One attacker one defender; big damage ensures one-shot kill.
        let mut e = Encounter::new(vec![
            pc(1, 20, 10, 100, "20d10"),
            enemy(2, 5, 10, 0, "1d1"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2)]);
        let mut rng = StdRng::seed_from_u64(0);
        let fx = e
            .resolve(
                Intent::Attack {
                    attacker: EntityId(1),
                    target: EntityId(2),
                    attack_idx: 0,
                },
                &mut rng,
            )
            .unwrap();
        let has_downed = fx.iter().any(|f| matches!(f, Effect::CreatureDowned { .. }));
        let has_end = fx.iter().any(|f| matches!(f, Effect::EncounterEnded { .. }));
        assert!(has_downed);
        assert!(has_end);
        assert!(e.is_ended());
        assert_eq!(e.winner(), Some(Side::Pcs));
    }

    #[test]
    fn not_your_turn_rejected() {
        let mut e = two_sided(20, 10, 100, "1d4");
        let mut rng = StdRng::seed_from_u64(0);
        let err = e
            .resolve(
                Intent::Attack {
                    attacker: EntityId(2),
                    target: EntityId(1),
                    attack_idx: 0,
                },
                &mut rng,
            )
            .unwrap_err();
        assert!(matches!(err, CombatError::NotYourTurn(_, _)));
    }

    #[test]
    fn initiative_sorted_descending() {
        let creatures = vec![
            Creature { initiative_bonus: -5, ..pc(1, 10, 10, 0, "1d4") },
            Creature { initiative_bonus: 10, ..pc(2, 10, 10, 0, "1d4") },
            Creature { initiative_bonus: 0, ..enemy(3, 10, 10, 0, "1d4") },
        ];
        let mut e = Encounter::new(creatures);
        let mut rng = StdRng::seed_from_u64(100);
        let fx = e.roll_initiative(&mut rng);
        if let Effect::InitiativeRolled { order } = fx {
            let totals: Vec<i32> = order.iter().map(|o| o.total).collect();
            let mut sorted = totals.clone();
            sorted.sort_by(|a, b| b.cmp(a));
            assert_eq!(totals, sorted);
        } else {
            panic!();
        }
    }

    #[test]
    fn end_turn_advances_to_next_actor() {
        let mut e = Encounter::new(vec![
            pc(1, 10, 10, 0, "1d4"),
            enemy(2, 10, 10, 0, "1d4"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2)]);
        assert_eq!(e.current_actor(), Some(EntityId(1)));
        let _ = e
            .resolve(Intent::EndTurn { actor: EntityId(1) }, &mut StdRng::seed_from_u64(0))
            .unwrap();
        assert_eq!(e.current_actor(), Some(EntityId(2)));
    }

    #[test]
    fn end_turn_wraps_round() {
        let mut e = Encounter::new(vec![
            pc(1, 10, 10, 0, "1d4"),
            enemy(2, 10, 10, 0, "1d4"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2)]);
        assert_eq!(e.round(), 1);
        let _ = e
            .resolve(Intent::EndTurn { actor: EntityId(1) }, &mut StdRng::seed_from_u64(0))
            .unwrap();
        let fx = e
            .resolve(Intent::EndTurn { actor: EntityId(2) }, &mut StdRng::seed_from_u64(0))
            .unwrap();
        assert_eq!(e.round(), 2);
        assert!(fx.iter().any(|f| matches!(f, Effect::RoundStarted { round: 2 })));
    }

    #[test]
    fn advance_skips_downed() {
        let mut e = Encounter::new(vec![
            pc(1, 10, 10, 0, "1d4"),
            pc(2, 0, 10, 0, "1d4"), // already down
            enemy(3, 10, 10, 0, "1d4"),
        ]);
        e.force_initiative(vec![EntityId(1), EntityId(2), EntityId(3)]);
        let _ = e
            .resolve(Intent::EndTurn { actor: EntityId(1) }, &mut StdRng::seed_from_u64(0))
            .unwrap();
        assert_eq!(e.current_actor(), Some(EntityId(3)));
    }

    proptest! {
        #[test]
        fn hp_monotonic_under_attacks(seed in any::<u64>()) {
            let mut e = Encounter::new(vec![
                pc(1, 1000, 10, 10, "1d6"),
                enemy(2, 1000, 10, 0, "1d4"),
            ]);
            e.force_initiative(vec![EntityId(1), EntityId(2)]);
            let mut rng = StdRng::seed_from_u64(seed);
            let mut last_hp = e.find(EntityId(2)).unwrap().hp;
            for _ in 0..20 {
                let _ = e.resolve(
                    Intent::Attack { attacker: EntityId(1), target: EntityId(2), attack_idx: 0 },
                    &mut rng,
                ).unwrap();
                let hp = e.find(EntityId(2)).unwrap().hp;
                prop_assert!(hp <= last_hp);
                last_hp = hp;
            }
        }

        #[test]
        fn combat_terminates(seed in any::<u64>()) {
            // Two-sided auto-play: each actor attacks the first living opponent.
            let creatures = vec![
                pc(1, 13, 16, 6, "1d6+4"),
                pc(2, 11, 14, 5, "1d6+3"),
                enemy(101, 7, 10, 2, "1d6"),
                enemy(102, 7, 10, 2, "1d6"),
                enemy(103, 7, 10, 2, "1d6"),
            ];
            let mut e = Encounter::new(creatures);
            let mut rng = StdRng::seed_from_u64(seed);
            e.roll_initiative(&mut rng);
            for _ in 0..2000 {
                if e.is_ended() { break; }
                let actor_id = e.current_actor().expect("active actor");
                let actor = e.find(actor_id).unwrap().clone();
                let target_opt = e.creatures().iter()
                    .find(|c| c.side != actor.side && !c.is_down())
                    .map(|c| c.id);
                match target_opt {
                    Some(tgt) => {
                        let _ = e.resolve(
                            Intent::Attack { attacker: actor_id, target: tgt, attack_idx: 0 },
                            &mut rng,
                        ).unwrap();
                    }
                    None => { /* no targets — end turn (should already be ended) */ }
                }
                if e.is_ended() { break; }
                let _ = e.resolve(Intent::EndTurn { actor: actor_id }, &mut rng).unwrap();
            }
            prop_assert!(e.is_ended(), "combat did not terminate in 2000 steps (seed {})", seed);
            prop_assert!(e.winner().is_some());
        }
    }
}
