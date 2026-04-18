---
type: design
name: Prototype 1 — Thin Vertical Slice Summary
description: สรุปสิ่งที่สร้างใน Prototype 1 (Living Icicles CLI), ไฟล์ที่เพิ่ม/แก้, เทสต์, และตัวอย่างการรัน
status: approved
sources:
  - "[[architecture]]"
  - "[[roadmap]]"
  - "[[rules-5e-2024]]"
  - "[[adventure-pinebrook]]"
tags:
  - design
  - prototype
  - milestone
---

# Prototype 1 — Thin Vertical Slice (Living Icicles CLI)

**วันที่เสร็จ:** 2026-04-17
**สถานะ:** ✅ ผ่าน Definition of Done ทุกข้อ

## เป้าหมาย

พิสูจน์สถาปัตยกรรม **Intent → Rules Engine → Effect** แบบ end-to-end โดยยังไม่แตะ LLM / WebSocket / multiplayer — ให้รัน combat จริง 1 ฉากได้จาก CLI พร้อมแสดงตัวเลขทอยเต๋าทุกครั้ง (dice visibility) และเขียน session log ลงดิสก์

## สิ่งที่ชิปออกมา

| ส่วน | รายละเอียด |
|---|---|
| **Scenario** | Peril in Pinebrook — Encounter 2: 4 PCs (Shalefire/Noorah/Gallantine/Evandon) vs 5 × Living Icicle |
| **CLI** | `dm encounter pinebrook [--seed N] [--fixtures PATH] [--out DIR] [--auto]` |
| **Determinism** | `StdRng::seed_from_u64` — seed เดียวกัน → ผลลัพธ์เดียวกันเป๊ะ |
| **Session log** | `campaigns/example/sessions/session01.md` พร้อม YAML frontmatter ตาม CLAUDE.md schema |
| **Dice visibility** | ทุก attack line แสดง `1d20+X → [nat]+X = total vs AC Y → hit/miss` และ damage roll แบบเดียวกัน |
| **Data shape** | ทั้ง const fixtures (`scenario::pinebrook::default_creatures()`) และ YAML loader (`dm_wiki::fixtures::load_creatures`) |

## ไฟล์ที่เพิ่ม/แก้

### เพิ่มใหม่
```
crates/dm-core/src/entity.rs              # EntityId, Side, DamageType, Attack, Creature
crates/dm-core/src/intent.rs              # Intent enum (Attack | EndTurn)
crates/dm-core/src/effect.rs              # Effect enum + narrative()
crates/dm-core/src/combat.rs              # Encounter + initiative + resolve()
crates/dm-core/src/scenario/mod.rs
crates/dm-core/src/scenario/pinebrook.rs  # default_creatures()
crates/dm-core/tests/pinebrook_scenario.rs # integration tests
crates/dm-wiki/src/fixtures.rs            # YAML loader
crates/dm-wiki/src/session.rs             # SessionWriter
campaigns/example/fixtures/pinebrook-icicles.yaml
campaigns/example/sessions/session01.md   # generated
```

### แก้ไข
```
Cargo.toml                         # +serde_yaml, +clap, +chrono
crates/dm-core/src/lib.rs          # +5 mod
crates/dm-wiki/Cargo.toml          # +serde_yaml, +chrono, +dm-core
crates/dm-wiki/src/lib.rs          # +fixtures, +session
crates/dm/Cargo.toml               # +dm-wiki, +clap, +rand
crates/dm/src/main.rs              # เขียนใหม่เป็น clap CLI
log.md                             # entry Prototype 1
active_task.md                     # ชี้ไป Prototype 2
```

## การตัดสินใจเชิงกลไก

- **Attack crit rule (2024):** nat-20 = auto-hit + crit (ทอย damage dice × 2, modifier ไม่เพิ่ม); nat-1 = auto-miss ทั้งที่ total อาจสูงพอก็ตาม
- **Check rule (strict RAW):** `D20Test` / `Check` ไม่มี nat-20 auto-success และไม่มี nat-1 auto-fail — ใช้กฎนี้กับ initiative ด้วย
- **Crit dice doubling:** `double_dice_on_crit(&Roll)` doubles each `DiceTerm.count` ด้วย `saturating_mul(2)`, `Modifier` ไม่แตะ
- **Intent / Effect separation:** engine รับ `Intent::Attack { attacker, target, attack_idx }` → คืน `Vec<Effect>`; ไม่มีทางลัดให้ฝั่งอื่นยัดค่า damage โดยตรง
- **Fire Bolt "7" trick:** parser `Roll::from_str` ไม่ยอม `d1` (MIN_SIDES=2) — flat damage ใช้ Modifier-only Roll (`"7"`) — ถูกต้องตาม `Roll::new` validation, narrative จะออกมาเป็น `7 → 7 = 7 fire`

## Testing

**รวม 69 tests ผ่านหมด**

| Crate | ชนิด | จำนวน |
|---|---|---:|
| `dm-core` | unit + proptest (dice, check, entity, combat, scenario) | 61 |
| `dm-core` | integration (`tests/pinebrook_scenario.rs`) | 4 |
| `dm-wiki` | unit (fixtures, session) | 4 |

Key tests ใน `combat.rs`:
- `attack_hits_when_total_meets_ac` / `attack_misses_when_total_below_ac`
- `nat_20_always_crits` + ตรวจว่า damage dice doubled จริง
- `nat_1_auto_misses_even_with_huge_bonus` — +100 vs AC 10 ยังมิสถ้าทอย 1
- `huge_bonus_always_hits_except_nat_1`
- `damage_reduces_hp` + `encounter_ends_when_one_side_down`
- **proptest `hp_monotonic_under_attacks`** — HP ลดอย่างเดียว ไม่เพิ่ม
- **proptest `combat_terminates`** — random seed 100 ครั้ง ≤ 50 rounds ทุกครั้ง

Integration tests (`pinebrook_scenario.rs`):
- `pinebrook_terminates_under_seed_42`
- `pinebrook_winner_stable_across_seeds` ([1, 42, 100, 999, 12345])
- `pinebrook_seed_42_is_deterministic` — ทอยด้วย seed เดียวกัน 2 ครั้ง → winner + rounds ตรงกัน
- `pinebrook_emits_dice_visible_narrative` — assert ทุก attack line มี `"1d20"` และ `"vs AC"`

## Re-use จาก Phase 0

ไม่เขียนซ้ำเลย:
- `dice::Roll::from_str` + `Roll::execute` — parse "1d6+4" และทอย
- `dice::RollResult::narrative` — แสดง `[4]+4 = 8` style
- `check::D20Test::new(bonus).execute(rng)` — attack roll + initiative
- `check::RollMode` (Adv/Dis) — พร้อมใช้ แม้ P1 ยังไม่ดึงมา

## ตัวอย่างการรัน (seed 42)

```
$ cargo run --bin dm -- encounter pinebrook --seed 42 --auto
Living Icicles encounter — seed 42
Session file: campaigns/example/sessions/session01.md

Initiative order:
  1. Evandon Haart (nat 20, total 21)
  2. Noorah Eldenfield (nat 18, total 21)
  3. Living Icicle 1 (nat 13, total 13)
  4. Living Icicle 3 (nat 13, total 13)
  5. Shalefire Stoutheart (nat 11, total 12)
  6. Gallantine Birchenbough (nat 9, total 11)
  7. Living Icicle 4 (nat 10, total 10)
  8. Living Icicle 2 (nat 9, total 9)
  9. Living Icicle 5 (nat 1, total 1)

Evandon Haart uses Mace on Living Icicle 1 — 1d20+5 → [17]+5 = 22 vs AC 10 → hit — damage 1d6+3 → [6]+3 = 9 bludgeoning (HP 7 → -2)
Living Icicle 1 is down.
Evandon Haart ends their turn.
Noorah Eldenfield uses Shortsword on Living Icicle 2 — 1d20+5 → [11]+5 = 16 vs AC 10 → hit — damage 1d6+3 → [6]+3 = 9 piercing (HP 7 → -2)
Living Icicle 2 is down.
Noorah Eldenfield ends their turn.
Living Icicle 3 uses Claws on Shalefire Stoutheart — 1d20+2 → [15]+2 = 17 vs AC 16 → hit — damage 1d6 → [4] = 4 cold (HP 13 → 9)
Living Icicle 3 ends their turn.
Shalefire Stoutheart uses Handaxe on Living Icicle 3 — 1d20+6 → [17]+6 = 23 vs AC 10 → hit — damage 1d6+4 → [2]+4 = 6 slashing (HP 7 → 1)
Shalefire Stoutheart ends their turn.
Gallantine Birchenbough uses Fire Bolt on Living Icicle 3 — 1d20+5 → [1]+5 = 6 vs AC 10 → miss
Gallantine Birchenbough ends their turn.
Living Icicle 4 uses Claws on Shalefire Stoutheart — 1d20+2 → [4]+2 = 6 vs AC 16 → miss
Living Icicle 4 ends their turn.
Living Icicle 5 uses Claws on Shalefire Stoutheart — 1d20+2 → [11]+2 = 13 vs AC 16 → miss
Living Icicle 5 ends their turn.
— Round 2 —
Evandon Haart uses Mace on Living Icicle 3 — 1d20+5 → [10]+5 = 15 vs AC 10 → hit — damage 1d6+3 → [1]+3 = 4 bludgeoning (HP 1 → -3)
Living Icicle 3 is down.
Evandon Haart ends their turn.
Noorah Eldenfield uses Shortsword on Living Icicle 4 — 1d20+5 → [14]+5 = 19 vs AC 10 → hit — damage 1d6+3 → [2]+3 = 5 piercing (HP 7 → 2)
Noorah Eldenfield ends their turn.
Shalefire Stoutheart uses Handaxe on Living Icicle 4 — 1d20+6 → [10]+6 = 16 vs AC 10 → hit — damage 1d6+4 → [4]+4 = 8 slashing (HP 2 → -6)
Living Icicle 4 is down.
Shalefire Stoutheart ends their turn.
Gallantine Birchenbough uses Fire Bolt on Living Icicle 5 — 1d20+5 → [9]+5 = 14 vs AC 10 → hit — damage 7 → 7 = 7 fire (HP 7 → 0)
Living Icicle 5 is down.
Encounter ends after 2 round(s). Winner: Pcs.

--- Encounter complete ---
Session saved to: campaigns/example/sessions/session01.md
```

## Definition of Done — ✅ ผ่านครบ

- ✅ `cargo build --workspace` ผ่าน
- ✅ `cargo test --workspace` ผ่าน (69 tests)
- ✅ `cargo run --bin dm -- encounter pinebrook --seed 42` จบ combat, exit 0
- ✅ `campaigns/example/sessions/session01.md` มี frontmatter + combat log dice-visible
- ✅ YAML fixture โหลดได้ผลเทียบเคียง const
- ✅ ไม่แตะ `dm-claude`, `dm-api` (ยังเป็น stub)
- ✅ `log.md` มี entry Prototype 1
- ✅ `active_task.md` ชี้ไป Prototype 2

## Out of scope → ยกไป Prototype 2+

- LLM narrative layer (`dm-claude`) — ตัวต่อไป
- WebSocket multiplayer (`dm-api`)
- เงื่อนไข 15 อย่างเต็มชุด, spell system, movement/grid, resistances/vulnerabilities
- Consequence seeds, importance decay, state inference
- Vault ingestion (อ่าน NPC/location pages เข้า context)

## Next: Prototype 2

ดูราย­ละเอียดใน `active_task.md` — หัวข้อ "Claude narrates Living Icicles" — wrap Intent/Effect loop ด้วย Claude tool-use API ให้ narrate scene + outcome โดยห้ามแตะเลข (เลขทุกตัวมาจาก `Effect`)
