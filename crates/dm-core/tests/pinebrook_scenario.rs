//! End-to-end Pinebrook Living Icicles combat — seeded replay.
//!
//! Asserts that auto-played Pinebrook combat terminates deterministically
//! under a fixed seed, and that the winner + round count are stable.

use dm_core::combat::Encounter;
use dm_core::effect::Effect;
use dm_core::entity::{EntityId, Side};
use dm_core::intent::Intent;
use dm_core::scenario;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn auto_play(seed: u64) -> (Option<Side>, u32, Vec<Effect>) {
    let creatures = scenario::pinebrook::default_creatures();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut enc = Encounter::new(creatures);
    let mut log = vec![enc.roll_initiative(&mut rng)];

    for _ in 0..5000 {
        if enc.is_ended() {
            break;
        }
        let actor_id = enc.current_actor().expect("actor");
        let actor = enc.find(actor_id).unwrap().clone();
        let target: Option<EntityId> = enc
            .creatures()
            .iter()
            .find(|c| c.side != actor.side && !c.is_down())
            .map(|c| c.id);
        if let Some(t) = target {
            let fx = enc
                .resolve(
                    Intent::Attack {
                        attacker: actor_id,
                        target: t,
                        attack_idx: 0,
                    },
                    &mut rng,
                )
                .expect("attack resolves");
            log.extend(fx);
            if enc.is_ended() {
                break;
            }
        }
        let fx = enc
            .resolve(Intent::EndTurn { actor: actor_id }, &mut rng)
            .expect("end turn");
        log.extend(fx);
    }

    (enc.winner(), enc.round(), log)
}

#[test]
fn pinebrook_terminates_under_seed_42() {
    let (winner, rounds, log) = auto_play(42);
    assert!(winner.is_some(), "encounter must have a winner");
    assert!(rounds >= 1 && rounds <= 50, "bounded round count: got {}", rounds);
    assert!(
        log.iter().any(|e| matches!(e, Effect::EncounterEnded { .. })),
        "log must contain EncounterEnded"
    );
}

#[test]
fn pinebrook_winner_stable_across_seeds() {
    // Not every seed will favour PCs, but across a spread the combat must at
    // least terminate and produce a deterministic result per seed.
    for seed in [1u64, 42, 100, 999, 12345] {
        let (winner, rounds, _) = auto_play(seed);
        assert!(winner.is_some(), "seed {} must terminate", seed);
        assert!(rounds <= 50, "seed {} ran too long: {} rounds", seed, rounds);
    }
}

#[test]
fn pinebrook_seed_42_is_deterministic() {
    let (w1, r1, _) = auto_play(42);
    let (w2, r2, _) = auto_play(42);
    assert_eq!(w1, w2);
    assert_eq!(r1, r2);
}

#[test]
fn pinebrook_emits_dice_visible_narrative() {
    let (_, _, log) = auto_play(42);
    let attacks: Vec<_> = log
        .iter()
        .filter_map(|e| match e {
            Effect::AttackResolved { .. } => Some(e.narrative()),
            _ => None,
        })
        .collect();
    assert!(!attacks.is_empty());
    // every attack line must show both the d20 natural and a vs-AC fragment
    for a in &attacks {
        assert!(a.contains("1d20"), "missing d20 in: {}", a);
        assert!(a.contains("vs AC"), "missing AC in: {}", a);
    }
}
