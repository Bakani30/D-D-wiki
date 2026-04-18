---
type: design
name: Adventure Pinebrook — Reference Scenario
description: Structural synthesis of Peril in Pinebrook as our first end-to-end engine test case — pregen PCs, encounter shapes, DC calibration.
status: draft
sources: ["[[Peril_in_Pinebrook_COMPLETE1-1]]"]
tags: [design, adventure, test-case]
---

# Adventure Pinebrook — Reference Scenario

Synthesis of *Peril in Pinebrook* (WotC 2023 intro adventure, 22 pages) as a **reference test scenario** for engine validation. Not the adventure text — the engine uses this to check that Intent/Effect resolution matches the published DCs and stat block shapes.

## Why this adventure

- Small — 4 encounters, linear, ~2 hour play.
- Pregen PCs — fixed stat block shape we can hard-code for tests.
- Mix of resolution modes — skill challenge, combat, social, multi-path. Exercises most of the d20-test surface.
- Explicit DCs in the text — ground truth for DC calibration.

## Pregen PC schema (4 characters)

All four pregens share the same shape — this defines our minimum PC record:

```
name, species, class, AC, HP/max_HP,
primary_attack: { name, bonus: +X, damage_dice: "1d6+Y", damage_type },
signature_skill: { name, ability, bonus: +X }
```

| Name | Species | Class | AC | HP | Attack | Signature Skill |
|---|---|---|---|---|---|---|
| Shalefire Stoutheart | Dwarf | Fighter | 16 | 13 | Handaxe +6 (1d6+4 slashing) | Athletics +6 |
| Noorah Eldenfield | Halfling | Rogue | 14 | 11 | Shortsword +5 (1d6+3 piercing) | Stealth +5 |
| Gallantine Birchenbough | Elf | Wizard | 12 | 9 | Fire Bolt +5 (7 fire, auto-rolled) | Arcana +5 |
| Evandon Haart | Human | Cleric | 14 | 11 | Mace +5 (1d6+3 bludgeoning) | Religion +5 |

**Observation:** adventure uses a *flat* attack/skill modifier per PC rather than decomposed ability+prof. Our engine's PC record should support both — decomposed for real play, flat for scenario tests.

## Monster stat block shape

Same minimum shape as PCs, with action list instead of skills:

```
name, AC, HP, speed, attacks: [ { name, bonus, damage_dice, damage_type } ], traits: []
```

| Monster | AC | HP | Attack |
|---|---|---|---|
| Living Icicle (×5) | 10 | 7 | Claws +2 (1d6 cold) |
| Egg Snatcher (×3) | 12 | 18 | Bite +4 (1d6+2 piercing) |

**Observation:** no saves, no skills, no resistances on these tier-0 monsters. Engine's monster schema should allow sparse population.

## DC calibration

Explicit DCs from the adventure map to 2024 RAW bands:

| DC | Band | Examples in adventure |
|---|---|---|
| 10 | Easy | Climb ledge (Ath/Acro), spot ice slides (Inv/Perc) |
| 15 | Medium | Control a slide (Acro/Ath) |

All multi-ability checks — adventure writes "Athletics or Acrobatics" — giving the player choice. Engine Intent schema must accept `{ skill: "athletics" | "acrobatics" }` with DM/engine-chosen DC, not hardcode one.

## Encounter patterns

Four encounters, each a distinct pattern our engine should handle:

1. **Social framing** (Not-So-Fearsome Dragon) — no mechanics, pure dialogue with baby silver dragon Nytha. Validates that the engine can run a scene without rolling anything.
2. **Swarm combat** (Living Icicles) — 5× identical weak monsters vs 4 PCs. Validates initiative, multi-target AoO, action economy for many low-HP enemies.
3. **Skill challenge** (Dangerous Lair) — 3 sequential gates: climb check, riddle (pure RP, no roll), slide check. Validates that the engine can sequence non-combat checks with progress state.
4. **Boss-adjacent combat + social pivot** (Dragon Eggs → Rorn) — 3× Egg Snatchers combat, then mother dragon Rorn arrives; outcome is negotiation, not fight. Validates that combat → social transition preserves state (who's alive, how much HP, who did what).

## Treasure

50gp diamond per PC — flat reward. Validates inventory delta + session-log update.

## Use as test fixture

This scenario becomes the basis for `tests/scenarios/pinebrook.rs` once combat is implemented:

- Seed RNG → pregens vs Living Icicles → assert combat resolves in <N rounds, PCs survive ≥75% of runs at DC 10.
- Dangerous Lair → replay a fixed Intent sequence → assert final state matches expected progress gates.
- Rorn confrontation → assert engine does not auto-resolve to combat when Intent is `Influence`.

## Open questions

- Should pregens live in the project vault or the campaign vault? — lean *campaign*, under `campaigns/pinebrook/players/`. Decide when we spin up a Pinebrook campaign vault.
- Fire Bolt auto-damage (7) vs rolled (2d10) — adventure pre-rolls for simplicity. Engine should support both; tests use the adventure's flat number.
