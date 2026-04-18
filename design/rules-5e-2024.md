---
type: design
name: Rules 5e 2024 — Engine Scope
description: Mechanical scope extracted from the 2024 free rules — what dm-core must resolve and what dm-claude must expose as Intent.
status: draft
sources:
  - "[[กฏฟรี D&D (2024)]]"
tags:
  - design
  - rules
---

# Rules 5e 2024 — Engine Scope

Synthesis of the free 2024 rules TOC into the **mechanical surface area** our engine must support. Not a rules reproduction — the raw source is a catalog of links to dnd-th.com; this doc extracts the *shapes* our code must model.

## Core resolution: the d20 test

Every non-damage resolution in 5e 2024 is a **d20 test** — one of three flavors:

| Flavor | Compared against | Modifier source | Crit rules |
|---|---|---|---|
| Ability check | DC (set by DM/engine) | ability mod + (proficiency if skill applies) | none (RAW — no nat-20 auto-success) |
| Saving throw | DC (from spell/effect) | ability mod + (proficiency if prof save) | none |
| Attack roll | AC (target's) | ability mod + proficiency + bonuses | nat 20 = crit hit; nat 1 = miss |

**Engine invariant:** checks and saves have **no** natural-20 auto-success and **no** natural-1 auto-fail (strict RAW 2024). Only attack rolls are crit-sensitive. `dm-core::check` must expose `natural_d20: u8` on the result for narrative flavor without *mechanically* triggering success.

## Advantage / Disadvantage

- Roll 2d20, take higher (adv) or lower (dis).
- Adv and dis **cancel** — if both apply, the test is normal. Multiple sources don't stack.
- Already implemented in `dice::RollMode` with `with_mode()` enforcing exactly one d20 term.

## The 15 conditions

These are the full enumeration — `dm-core::condition::Condition` enum:

`Blinded, Charmed, Deafened, Exhaustion, Frightened, Grappled, Incapacitated, Invisible, Paralyzed, Petrified, Poisoned, Prone, Restrained, Stunned, Unconscious`

Exhaustion is **leveled** (1–6) in 2024 with a cumulative -2 per level on all d20 tests. Other conditions are boolean presence. Several compose (Paralyzed → Incapacitated + auto-fail Str/Dex saves + attacker has adv + crit within 5ft).

## The 12 actions

Intent schema (`dm-claude`) must surface exactly these action kinds:

`Attack, Dash, Disengage, Dodge, Help, Hide, Influence, Magic, Ready, Search, Study, Utilize`

Plus free movement and one bonus action. "Influence" (replaces old social-interaction framing) and "Study" (replaces old "recall info" ad hoc) are new in 2024 — both resolve as ability checks, not special subsystems.

## Areas of effect

`Cone, Cube, Cylinder, Emanation, Line, Sphere` — spell templates. Engine only needs to enumerate affected squares/creatures; shape math is pure geometry.

## Combat loop (per round)

1. **Initiative** — d20 + Dex mod per participant, descending order. Set at combat start; no re-roll.
2. **Turn**: move up to speed (splittable), one action, one bonus action (if available), one reaction (off-turn), free object interaction.
3. **Opportunity attacks** — trigger on leaving reach without Disengage.
4. **End of turn** — tick conditions, concentration checks on damage (DC = max(10, damage/2)).

`dm-core` owns: initiative ordering, turn state machine, action economy enforcement, damage application, HP/death-save state, concentration.

## Damage types

Physical: `Bludgeoning, Piercing, Slashing`
Elemental: `Acid, Cold, Fire, Lightning, Thunder`
Other: `Poison, Psychic, Necrotic, Radiant, Force`

Creatures have `resistance`/`immunity`/`vulnerability` sets keyed by type.

## Skills → ability mapping

Str: Athletics
Dex: Acrobatics, Sleight of Hand, Stealth
Int: Arcana, History, Investigation, Nature, Religion
Wis: Animal Handling, Insight, Medicine, Perception, Survival
Cha: Deception, Intimidation, Performance, Persuasion

## What this doc is NOT

- Not a player-facing rulebook. Players reference `raw/กฏฟรี D&D (2024).md` or dnd-th.com directly.
- Not a class/species catalog. Those are data, loaded per campaign vault.
- Not spell mechanics. Individual spell resolution lives in a data-driven spell table, not here.

## Open questions

- Conditions subsystem: one flat enum, or enum + level field only for Exhaustion? — defer until `dm-core::condition` is implemented.
- Influence action: does the engine auto-compute DC from target disposition, or does the DM supply? — revisit in Phase 2.
