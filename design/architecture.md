---
type: design
name: Architecture
description: System architecture for the D&D AI DM — Intent/Effect, Multi-Model, Rules Engine, Memory
status: draft
sources: ["[[plan]]", "[[chronicler]]", "[[improved-initiative]]", "[[dnd_ai_dm]]"]
tags: [design, architecture]
---

# ปรัชญาหลัก (Core Philosophy)

โปรเจกต์นี้ไม่ใช่ "แชทบอทที่เล่น D&D" — เป้าหมายคือ **AI Dungeon Master ที่กฎแม่น จำได้จริง และสเกลได้**

เสาหลัก 3 ข้อ:
1. **Intent / Effect Separation** — AI ไม่แก้ไข state โดยตรง
2. **Multi-Model** — แบ่งงานให้โมเดลที่เหมาะกับต้นทุนและความเร็ว
3. **Wiki as Truth** — โลกของเกมคือไฟล์ Markdown ไม่ใช่ context window

---

## 1. Intent / Effect Separation

AI เป็น "นักเล่าเรื่อง" ไม่ใช่ "ผู้ตัดสิน"

```
Player: "I swing my axe at the goblin"
    ↓
Narrative Agent → Intent { action: attack, target: goblin, weapon: greataxe }
    ↓
Rules Engine (code, not LLM):
    - ทอย d20 + modifiers
    - เทียบกับ AC
    - ทอย damage ถ้า hit
    ↓
Effect { hit: true, damage: 8, target_hp: 3 }
    ↓
Narrative Agent บรรยายผล: "ขวานฟันเฉือนเกราะกอบลิน เลือดสาด..."
    ↓
World State update (file/DB)
```

**ทำไมต้องแยก:**
- กัน LLM แต่งตัวเลขเอง (hallucination)
- กันโมเดลลำเอียง (AI อาจเมตตาผู้เล่นเกินไป)
- ทดสอบได้ (Rules Engine เป็น deterministic code)
- เปลี่ยนโมเดลได้โดยไม่กระทบกฎ

**สิ่งที่ Rules Engine ต้องทำเอง (ห้าม LLM ทำ):**
- ทอยเต๋าทุกชนิด (attack, save, skill check, damage)
- คำนวณ modifier, advantage/disadvantage
- ตรวจ AC, DC, condition
- หัก HP / ให้ status effect
- initiative order

---

## 2. Multi-Model Architecture

ไม่ใช้ LLM ตัวเดียวทำทุกงาน — แพงเกินและช้าเกิน

| บทบาท | โมเดลแนะนำ | หน้าที่ | ความถี่ |
|---|---|---|---|
| **Narrative Agent** | Opus / Sonnet | สวมบท NPC, บรรยายฉาก, วางแผนแคมเปญ | ทุกเทิร์นที่ผู้เล่นพูด |
| **Rules Adjudicator** | Sonnet (หรือ code-only) | แปลง narrative → Intent JSON | ทุกเทิร์น |
| **State Inference Agent** | Haiku | อ่านบทสนทนาแล้วอนุมาน state change (NPC disposition, location, etc.) | หลังทุก response |
| **Relevance Checker** | Haiku | เช็คว่าผู้เล่น trigger consequence seed ตัวไหนมั้ย | ทุกเทิร์น |
| **Ingest Agent** | Sonnet | สรุป session เป็น wiki update | ครั้ง/session |

อ้างอิงโดยตรงจาก Chronicler ที่ใช้ pattern เดียวกัน

---

## 3. Rules Engine (ต้องเป็นโค้ด ไม่ใช่ prompt)

### Dice Roller
- `roll(expr: "1d20+5")` → integer
- รองรับ advantage/disadvantage: `roll_advantage("1d20+3")`
- seeded RNG สำหรับ replay/debug

### Combat Module
- `resolve_attack(attacker, target, weapon)` → `{hit, crit, damage, damage_type}`
- `apply_damage(target, amount, type)` → updated HP + resistances/vulnerabilities
- `resolve_save(creature, ability, dc)` → pass/fail
- death saves, concentration, condition effects

### Turn Manager
- initiative queue
- action economy (action / bonus / reaction / movement)
- round counter

**หมายเหตุ:** improved-initiative เป็นตัวอย่างที่ดีของ combat tracker UI — ใช้เป็น reference ได้เวลาออกแบบ frontend (Phase 4)

---

## 4. State Persistence (สรุป — ดูรายละเอียดใน [[memory-system]])

- **Phase 1–3:** Markdown ใน Obsidian vault (ไฟล์นี้แหละ)
- **Phase 4:** ย้ายเป็น DB จริง (Postgres/MongoDB) เพื่อ multiplayer

โครงสร้าง entity ดูใน `CLAUDE.md` (npcs, locations, factions, items, quests, sessions)

---

## 5. Technology Stack (Hybrid: Rust core + Python tooling)

**Decision (2026-04-17):**
- **Rust** = ทั้ง game engine hot path (rules, combat, orchestration, realtime server, wiki I/O, Claude client) — ให้เกมไหลลื่น
- **Python** = off-path tooling + ML-heavy services ที่ Python ได้เปรียบ (ingest SRD, semantic embeddings, evals, notebook experiments)

### 5.1 Rust layer (hot path)

| Layer | Choice | เหตุผล |
|---|---|---|
| Rules Engine | **Rust** | Type-safe dice/combat, exhaustive pattern matching บน spell/condition effects, fearless concurrency |
| Orchestrator | **Rust** | เก็บ game loop ทั้งก้อนไว้ในภาษาเดียว → latency ต่ำ, state machine เทสต์ได้ |
| LLM Client | **Minimal Rust HTTP client** (`dm-claude`, แบบ `chronicler/claude`) | ไม่มี official Anthropic Rust SDK — เขียนเอง ~200 บรรทัดบน `reqwest` + `eventsource-stream` |
| Async runtime | **tokio** | Standard; จำเป็นสำหรับ concurrent agent calls + multiplayer connections |
| Serialization | **serde** + `serde_yaml` + `serde_json` | Frontmatter + Intent/Effect JSON |
| Testing | **cargo test** + `proptest` | Property-based test เหมาะกับ dice/combat invariants |
| CLI (Phase 1) | **clap** + `ratatui` (optional TUI) | Composable, standard |
| Realtime (Phase 1+) | **axum** + `tokio-tungstenite` | WebSocket room สำหรับ 4 ผู้เล่น ตั้งแต่ Phase 1 |
| Storage (Phase 1–3) | **Markdown + YAML** ผ่าน `pulldown-cmark` + `serde_yaml` | Obsidian-friendly, diff-able ด้วย git |
| Storage (Phase 4) | **Postgres** via `sqlx` (compile-time checked) | JSONB สำหรับ state |
| Desktop (optional Phase 4) | **Tauri** | ห่อ React frontend เป็น desktop app |
| Web API (Phase 4) | **axum** + **tower** | Rust web framework มาตรฐาน |
| Frontend (Phase 4) | **React + Vite + TypeScript** | Combat tracker, map view, inventory — reusable ระหว่าง web และ Tauri |

### 5.2 Python layer (off-path, tooling)

| Service | Stack | หน้าที่ |
|---|---|---|
| **SRD Ingest** | `beautifulsoup4`, `pypdf2`, `pydantic` | แปลง SRD/stat blocks → YAML ให้ Rust โหลด; one-shot scripts ไม่ใช่ runtime service |
| **Embedding Service** | `sentence-transformers`, `fastapi` + `uvicorn` | HTTP microservice: ให้ embedding vector สำหรับ consequence seed semantic matching — Rust เรียกผ่าน HTTP ตอนต้องการ |
| **Evals / Regression** | `pytest`, `pandas`, `anthropic` (Python SDK) | รัน scenario suite ต่อ PR, LLM-as-judge, balance analysis |
| **Notebooks** | Jupyter, `matplotlib` | Prompt iteration, session analytics, decay-curve tuning |
| **Local LLM (optional)** | `llama-cpp-python` หรือ `ollama` client | ทดลอง self-hosted model สำหรับ inference agent (Haiku alternative) |

**หลักการ:** Python ไม่อยู่ใน gameplay request path ที่ผู้เล่นรอ. ถ้าเป็น hot path → Rust เสมอ. ข้อยกเว้นเดียวคือ embedding service (เรียก async non-blocking, ผลไม่ทำให้ narrative response ช้า).

### 5.3 Service Boundary (Rust ↔ Python)

```
Rust game engine ──HTTP──▶ Python embedding service  (semantic seed matching)
Rust game engine ◀──YAML── Python ingest scripts     (offline, one-shot)
Python evals    ──HTTP──▶ Rust game engine           (regression runs)
```

- ไม่ใช้ PyO3 bindings — service boundary ผ่าน HTTP ชัดกว่า, debug ง่ายกว่า, deploy แยกได้
- Python services พัง ≠ เกมพัง (embedding service ถ้าไม่ตอบ Rust fallback เป็น keyword match ธรรมดา)

### 5.4 Repo / Workspace Layout

```
dnd-dm/                               # project root (แยกจาก design vault นี้)
├── Cargo.toml                        # Rust workspace root
├── crates/
│   ├── dm/                           # entry binary: CLI (P1), server (P1+), Tauri shell (P4)
│   │   └── src/main.rs
│   ├── dm-core/                      # game engine: rules, combat, orchestration
│   │   └── src/
│   │       ├── dice.rs
│   │       ├── combat/
│   │       ├── intent.rs             # Intent enum + serde (strict tool-use schema)
│   │       ├── effect.rs             # Effect enum + serde
│   │       ├── agent/
│   │       │   ├── narrative.rs      # Sonnet/Opus
│   │       │   ├── inference.rs      # Haiku — state inference
│   │       │   └── relevance.rs      # Haiku — consequence seeds (calls py-tools/embed)
│   │       ├── memory/
│   │       │   ├── facts.rs          # importance decay
│   │       │   └── seeds.rs          # consequence triggers
│   │       └── world.rs              # World State aggregate
│   ├── dm-wiki/                      # Markdown vault I/O (swap to Postgres in P4)
│   │   └── src/{frontmatter,entity,lint}.rs
│   ├── dm-claude/                    # minimal Anthropic HTTP client
│   │   └── src/lib.rs
│   └── dm-api/                       # axum server: REST + WebSocket multiplayer rooms
│       └── src/{room,router,stream}.rs
├── py-tools/                         # Python side — pyproject.toml
│   ├── ingest/                       # SRD → YAML one-shot scripts
│   ├── embed/                        # fastapi embedding microservice
│   ├── evals/                        # pytest suite, regression scenarios
│   └── notebooks/                    # Jupyter experiments
└── frontend/                         # React + Vite (Phase 4, reusable for Tauri)
    └── src/...
```

**แยก crate เพราะอะไร:**
- `dm-core` ไม่ควรรู้จัก disk/IO → testable ด้วย in-memory World
- `dm-wiki` แยกเพราะ Phase 4 จะสลับไป Postgres โดยแทน trait เดียว
- `dm-claude` แยกเพราะ Phase 4 อาจเพิ่ม local LLM / mock สำหรับ test
- `dm-api` แยกเพราะ multiplayer room + WebSocket เริ่มจาก Phase 1 (4 players) และจะขยายใน Phase 4

### 5.5 Intent Schema — Strict via Claude Tool Use

**Decision (2026-04-17):** Intent ทุกตัวส่งผ่าน Claude **tool use API** — ไม่ใช่ free-form JSON ใน text

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intent {
    Attack { target_id: EntityId, weapon_id: WeaponId },
    CastSpell { spell_id: SpellId, targets: Vec<EntityId>, slot_level: u8 },
    Move { to: Position },
    Interact { target_id: EntityId, action: String },
    Skill { ability: Ability, dc: Option<u8>, purpose: String },
    // ...
}
```

Benefits:
- Narrative Agent ใช้ tool definition schema → ไม่มี parse failure
- Rust ได้ typed struct มาทันที ด้วย `serde_json::from_value`
- Schema versioning: `IntentV1`, `IntentV2` เพิ่มได้โดยไม่กระทบของเก่า
- Test-able: ทำ golden JSON + replay ได้

---

## 6. Data Flow (Gameplay loop — 4-player multiplayer, Phase 1+)

```
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│ Player 1 │   │ Player 2 │   │ Player 3 │   │ Player 4 │   (React / CLI)
└──────────┘   └──────────┘   └──────────┘   └──────────┘
      │              │              │              │
      └──────────────┴──────────────┴──────────────┘
                           ↓ WebSocket
┌────────────────────────────────────────────────────────────┐
│ dm-api (Rust, axum)                                        │
│  - Session Room (1 ห้อง = 4 ผู้เล่น + 1 AI DM)             │
│  - Player turn queue (ตามผู้เล่นที่ input มา)             │
│  - Broadcast narrative chunks กลับทุกคน                    │
└────────────────────────────────────────────────────────────┘
                           ↓ (Rust in-process)
┌────────────────────────────────────────────────────────────┐
│ dm-core (Rust) — Orchestrator                              │
│ ① Relevance Check (Haiku) — match consequence seeds        │
│   └── optional: Python embed service (HTTP) สำหรับ semantic│
│ ② Context Builder — query wiki + memory                    │
│ ③ Narrative Agent (Sonnet) — call via tool use API         │
│   → Intent (typed Rust enum via tool-use JSON schema)      │
│ ④ Rules Engine — dice, AC check, damage, conditions        │
│   → Effect (typed Rust enum) **numbers visible to players**│
│ ⑤ Narrative Agent round 2 (Sonnet) — narrate Effect        │
│   → stream back via dm-api → players                       │
│ ⑥ State Inference (Haiku, async) — infer implicit changes  │
└────────────────────────────────────────────────────────────┘
                           ↓
┌────────────────────────────────────────────────────────────┐
│ dm-wiki — persist to Markdown (P1–3) / Postgres (P4)       │
└────────────────────────────────────────────────────────────┘
```

**Players see dice numbers.** Narrative Agent prompt ระบุให้แทรกตัวเลขผลทอย เช่น
*"คุณเหวี่ยงขวาน — (Attack: 1d20+5 = **18** vs AC 15, hit) — (Damage: 1d12+3 = **11** slashing) เลือดกระเซ็น..."*

---

## 7. Open Questions — ตัดสินใจครบแล้ว (2026-04-17)

- [x] ใช้ภาษาอะไร — ✅ **Hybrid: Rust hot path + Python tooling/ML**
- [x] Intent schema — ✅ **Strict JSON ผ่าน Claude tool use** (typed Rust enum via serde)
- [x] ผู้เล่นเห็นตัวเลข dice — ✅ **เห็น**, ฝังในคำบรรยาย
- [x] Multiplayer — ✅ **4 ผู้เล่น ตั้งแต่ Phase 1** (dm-api + WebSocket rooms)
- [x] Vault scope — ✅ **Vault นี้ = design/project-meta only**; campaigns แยก vault (dev: `campaigns/example/`)
- [ ] เก็บ dice roll log ไว้ไหน? (debug/replay) — minor, แก้ทีหลังได้
