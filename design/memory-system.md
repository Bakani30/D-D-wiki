---
type: design
name: Memory System
description: Persistent World State — Wiki as Truth, Importance Decay, Consequence Seeds, State Inference
status: draft
sources: ["[[plan]]", "[[chronicler]]"]
tags: [design, memory, persistence]
---

# ระบบความจำของ AI DM

ปัญหา: LLM ลืมหลัง ~20 turn. การ scroll ไป re-read chat history ไม่ scale และทำให้ hallucinate
ทางแก้: **ไม่พึ่ง context window. ใช้ structured storage แทน**

---

## 3 ชั้นของความจำ

### Layer 1 — Wiki as Truth (ระยะยาว, deterministic)

ทุก entity ในโลกของเกมเป็นไฟล์ Markdown ใน vault นี้

| Entity | Folder | ตัวอย่าง field |
|---|---|---|
| NPC | `npcs/` | hp, disposition, location, faction |
| Location | `locations/` | category, ruler, discovered |
| Faction | `factions/` | leader, allies, enemies |
| Item | `items/` | rarity, attunement, location |
| Quest | `quests/` | status, giver, objective |
| Session | `sessions/` | date, players, summary |

ดูรายละเอียด schema ใน `CLAUDE.md`

**คุณสมบัติ:**
- Source of truth ที่เถียงไม่ได้ — Narrative Agent ต้องเชื่อไฟล์เสมอ
- Diff-able ด้วย git (audit ได้ว่า NPC X เปลี่ยน disposition เมื่อ session ไหน)
- Human-editable (DM มนุษย์เข้ามาแก้ได้)

### Layer 2 — Structured Fact Memory (ระยะกลาง, decaying)

ข้อเท็จจริงที่ไม่เหมาะเป็น entity page — เก็บเป็น record พร้อม importance score

```json
{
  "id": "fact_042",
  "subject": "[[Mira]]",
  "category": "secret",
  "fact": "เป็นคนวางยาพ่อค้าน้ำ แต่โยนความผิดให้ [[Old Thomas]]",
  "importance": 0.9,
  "created_turn": 127,
  "related_entities": ["[[Riverside]]", "[[Old Thomas]]"]
}
```

**Importance Decay:**
- ทุก turn ผ่านไป importance ลด 2% (`importance *= 0.98`)
- event ที่มี importance < 0.1 → drop ออก (หรือ archive)
- **trigger จาก player action** ดัน importance กลับขึ้น (เช่น ผู้เล่นพูดชื่อ Mira → fact ที่ related ดันขึ้น 0.9 อีกครั้ง)

**หมวด (category) แนะนำ:**
- `secret` — สิ่งที่ผู้เล่นยังไม่รู้
- `promise` — คำสัญญาที่ NPC ให้ไว้
- `grudge` — ความแค้น
- `debt` — หนี้สิน (ทางใจหรือทางวัตถุ)
- `observation` — รายละเอียดเล็กๆ ที่อาจมีค่าในอนาคต

**จัดเก็บยังไง (ตามเฟส):**
- Phase 3: ไฟล์ `memory/facts.jsonl` (append-only)
- Phase 4: ตาราง Postgres พร้อม full-text index

### Layer 3 — Consequence Seeds (ระยะยาวแบบ conditional)

"เมล็ดพันธุ์แห่งผลลัพธ์" ที่รอ trigger

```json
{
  "id": "seed_007",
  "trigger": "player พูดถึง 'missing children' กับใครก็ได้ใน [[Riverside]]",
  "consequence": "ช่างตีเหล็ก [[Boran]] ได้ยิน → disposition เปลี่ยนเป็น hostile — ลูกของเขาอยู่ในกลุ่มที่หายไป",
  "severity": "major",
  "status": "pending",
  "created_turn": 89
}
```

**Matching:**
- ทุกเทิร์น **Relevance Checker** (Haiku) เช็ค player action กับ pending seeds
- ใช้ **semantic matching** ไม่ใช่ keyword (เพื่อจับเรื่องเดียวกันที่พูดคนละคำ)
- match → trigger fires → seed status → `fired`, inject consequence เข้า Narrative Agent

**Semantic embedding service (Python sidecar):**
- `py-tools/embed/` เป็น FastAPI microservice ใช้ `sentence-transformers`
- Rust `dm-core::agent::relevance` เรียก HTTP endpoint `/embed` → ได้ vector กลับมา
- เก็บ embedding ของ seed trigger ไว้ล่วงหน้า, เทียบ cosine similarity ตอน player พูด
- **Fallback:** ถ้า Python service ล่ม → Rust ใช้ keyword match ธรรมดา (เกมไม่พัง แค่ accuracy ลด)
- Phase 1 ข้ามไปก่อน (keyword-only), เปิดใช้ตอน Phase 2-3

**Pattern ที่ใช้ได้:**
- **Revenge plot** — ไว้ชีวิตโจร → โจรกลับมาแก้แค้น
- **Reputation cascade** — ช่วยหมู่บ้าน → พ่อค้าให้ส่วนลด
- **Ticking clock** — ไม่สน cultist → ritual สำเร็จ, world state เปลี่ยน
- **Chekhov's gun** — NPC พูดถึงไอเทมลึกลับใน session 1 → โผล่จริงใน session 5

---

## Post-Narrative State Inference

**ปัญหา:** Narrative Agent เขียน *"Captain Voss storms out, muttering about incompetent adventurers"* แต่ไม่ได้เรียก `update_npc(disposition="hostile")`

**วิธีแก้:** หลัง response ออก, ส่ง State Inference Agent (Haiku) ไปอ่านซ้ำ — output เป็น structured delta + confidence

```json
{
  "entity": "[[Captain Voss]]",
  "changes": {
    "location": "outside the tavern",
    "disposition": "hostile"
  },
  "confidence": 0.92,
  "evidence": "storms out... incompetent adventurers"
}
```

**Policy:**
- `confidence > 0.8` → apply ทันที + บันทึกใน History
- `0.5 < confidence <= 0.8` → flag รอ human-DM approve
- `confidence <= 0.5` → drop

นี่คือสิ่งที่ปิด **"narrative-state gap"** — ป้องกัน wiki drift ห่างจาก narrative

---

## Proactive World Design

ก่อนผู้เล่นเดินเข้า location ใหม่ — DM ต้อง **design ไว้ก่อน** ไม่ใช่ improvise ตอนผู้เล่นมา

ตอนสร้าง location ใหม่ให้ตั้งค่า:
- **NPCs:** มี goal, secret, daily routine
- **Relationships:** ใครเกลียดใคร ใครติดหนี้ใคร
- **Scheduled events:** เกิดขึ้นแม้ไม่มีผู้เล่น (evening market, temple ritual)
- **Consequence seeds:** 2–3 seeds ต่อ location — ให้ world รู้สึก reactive

ผลลัพธ์: ผู้หญิงในโรงเตี๊ยมที่มองประตูด้วยความกังวล **มีเหตุผลจริงๆ** (เพราะเราออกแบบ secret ไว้แล้วว่ารอใครบางคน)

---

## Context Assembly (ตอน query)

เมื่อถึง turn ของผู้เล่น — ไม่ใช่ยัด wiki ทั้งก้อนใส่ prompt. ทำแบบนี้:

1. **Current scene context** (always): location ปัจจุบัน + NPC ที่อยู่ที่นั่น + quest active ที่เกี่ยวข้อง
2. **Fact memory** (top-N by importance, filtered by related_entities ∩ current scene)
3. **Consequence seeds** (ส่งให้ Relevance Checker, ไม่ส่งให้ Narrative เว้น trigger fires)
4. **Recent turns** (last 3–5 turns, raw)
5. **Character sheets** (PC)

ใช้ **prompt caching** ของ Anthropic กับทุกส่วนที่ไม่เปลี่ยน (character sheet, static world info) — ลด cost 90%+

---

## Migration plan (Markdown → DB)

ใน Phase 4 ต้องย้ายจาก flat file → DB

| Markdown | Postgres |
|---|---|
| YAML frontmatter | JSONB column |
| History section | events table (append-only) |
| `facts.jsonl` | facts table + pgvector สำหรับ semantic search |
| Consequence seeds | seeds table + cron/trigger system |
| wikilinks | foreign keys (entity_id) |

เหตุผลที่ไม่เริ่มด้วย DB เลย:
- Phase 1–3 ต้องการ readability และ manual edit
- Diff-friendly สำหรับ campaign review
- Onboarding ง่าย (ไม่ต้องตั้ง DB)
