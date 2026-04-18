---
type: design
name: Roadmap
description: 4-phase development plan, from MVP combat engine to multiplayer web platform
status: draft
sources: ["[[plan]]", "[[chronicler]]", "[[dnd_ai_dm]]", "[[improved-initiative]]"]
tags: [design, roadmap]
---

# Roadmap: จาก MVP ถึง Web Platform

4 Phase ตาม [[plan]] — เพิ่มรายละเอียดจาก [[references]] แต่ละโปรเจกต์

---

## Phase 1 — MVP & Core Engine (multiplayer-4 ตั้งแต่ day 1)

**เป้าหมาย:** 4 ผู้เล่นต่อเข้าห้องเดียวกัน เล่นฉากต่อสู้จบได้ โดยกฎถูกต้องและเห็น dice numbers
**ภาษา:** Rust (hot path) + Python (tooling) — ดู [[architecture]] § 5

### Must-have — Rust side
- [ ] Cargo workspace 5 crates: `dm`, `dm-core`, `dm-wiki`, `dm-claude`, `dm-api`
- [ ] `dm-core::dice` — `Dice::roll("1d20+5")`, advantage/disadvantage, seeded RNG; `proptest` suite; dice log (visible to players)
- [ ] `dm-core::combat` — initiative queue, attack resolution, damage, death saves, condition tracking
- [ ] `dm-core::intent` + `dm-core::effect` — typed Rust enums; **Intent = Claude tool-use schema** (strict JSON)
- [ ] Character model: PC + Monster structs (HP, AC, abilities, attacks) with serde
- [ ] `dm-claude` — minimal HTTP client (reqwest + eventsource-stream) รองรับ tool use, streaming, prompt caching
- [ ] `dm-core::agent::narrative` — ใช้ tool-use API, prompt ระบุให้แสดงตัวเลข dice ในคำบรรยาย
- [ ] `dm-api` (axum) — WebSocket endpoint, session room ที่รับ 4 ผู้เล่น, broadcast narrative chunks
- [ ] `dm-wiki` — `WikiStore` trait + Markdown impl; เขียน session เข้า `campaigns/<name>/sessions/` (ไม่ใช่ vault นี้)
- [ ] `dm` binary — subcommands: `dm serve` (axum server), `dm cli` (local test client)

### Must-have — Python side (minimal Phase 1)
- [ ] `py-tools/ingest/` — สคริปต์ดึงกอบลิน/ออร์ค stat blocks จาก SRD → YAML
- [ ] `py-tools/evals/` — pytest suite 1 ไฟล์: "goblin ambush" golden scenario (regression test)
- [ ] `py-tools/embed/` — **ข้ามใน Phase 1** (seed matching ใช้ keyword fallback ก่อน, เปิดใช้ตอน Phase 2-3)

### Nice-to-have
- [ ] TUI combat tracker ด้วย `ratatui` สำหรับ local debug client
- [ ] Pre-baked stat blocks (goblin, orc, dragon) โหลดจาก YAML
- [ ] Seeded replay สำหรับ debug

**อ้างอิง:**
- [[chronicler]] = reference หลัก — workspace + crate split + `claude` client pattern
- [[dnd_ai_dm]] = inspiration เรื่อง module decomposition เท่านั้น (คนละภาษา)
- [[improved-initiative]] = UI/UX reference สำหรับ Phase 4 frontend

**ออก Phase 1 เมื่อ:** `dm serve` รันอยู่, 4 WebSocket clients ต่อเข้าห้องเดียวกัน, เล่นฉาก "goblin ambush" จบได้, HP / dice / initiative ถูกต้องตาม 5e, player เห็นตัวเลขทอย, session เขียนลง `campaigns/example/sessions/session01.md`

---

## Phase 2 — Agentic Workflow

**เป้าหมาย:** AI ไม่ใช่ตัวเดียว — แตกเป็น agent หลายตัวคุยกัน

### Must-have
- [ ] **State Inference Agent** (Haiku) — อ่าน narrative แล้วอนุมาน state change
- [ ] **Relevance Checker** (Haiku) — เช็ค consequence seeds
- [ ] **Rules Adjudicator** — แยก intent extraction ออกจาก narrative generation
- [ ] Confidence scoring: `>0.8 apply automatically, else flag`
- [ ] Async pipeline (narrative trả lời ผู้เล่นก่อน, inference ทำงานเบื้องหลัง)

### Nice-to-have
- [ ] Model selection ต่อ turn ตามความซับซ้อน (combat → Sonnet, chat NPC → Haiku)
- [ ] Cost/latency metrics ต่อ agent

**อ้างอิง:** [[chronicler]] เป็น implementation ต้นแบบของ pattern นี้ — architecture copy ได้ตรงเพราะภาษาเดียวกัน (Rust)

**ออก Phase 2 เมื่อ:** ผู้เล่นคุยกับ NPC 5 เทิร์น, NPC disposition เปลี่ยนเองโดยที่ DM ไม่ต้องสั่ง

---

## Phase 3 — Persistence & World Building

**เป้าหมาย:** โลกของเกมอยู่ยืน ข้ามหลาย session ได้

### Must-have
- [ ] Wiki integration: Rules Engine อ่าน/เขียนไฟล์ Markdown ใน vault นี้โดยตรง
- [ ] **Ingest workflow** — หลังจบ session: สรุป → update NPC/location/quest → append log
- [ ] **Query workflow** — ระหว่างเล่น: pull context จาก wiki (ไม่มั่ว)
- [ ] **Lint workflow** — ตรวจ contradiction (NPC dead แต่อยู่ใน active quest ฯลฯ)
- [ ] **Importance Decay** — fact แต่ละอันมี importance score, ลด 2%/เทิร์น, หายเมื่อต่ำกว่า threshold
- [ ] **Consequence Seeds** — DB ของ pending triggers + semantic matcher

### Nice-to-have
- [ ] Git integration — ทุก session = 1 commit
- [ ] Timeline view ของ log.md

**อ้างอิง:** Obsidian-as-database pattern (คือสิ่งที่ทำอยู่ตอนนี้) + [[chronicler]] memory system

**ออก Phase 3 เมื่อ:** เล่นข้าม 3 session, NPC ที่ผู้เล่นด่าไว้ใน session 1 จำได้ใน session 3

---

## Phase 4 — Full Web Platform

**เป้าหมาย:** multiplayer, visual, shareable

### Must-have
- [ ] ย้าย storage: Markdown → Postgres ผ่าน `sqlx` (compile-time checked queries); แทนที่ `WikiStore` impl
- [ ] API layer: `axum` + `tower` ครอบ `dm-core`; JSON-over-HTTP
- [ ] SSE streaming endpoint สำหรับ narrative push (ใช้ `axum` response stream)
- [ ] **React + Vite + TypeScript frontend**:
  - Combat Tracker (inspired by [[improved-initiative]])
  - Character sheet
  - Inventory grid
  - Map view
  - Chat panel (streaming narrative via SSE)
- [ ] Authentication (OAuth, หรือ bring-your-own-key pattern แบบ [[chronicler]])
- [ ] Multiplayer session rooms (4–5 players + 1 AI DM) — ใช้ tokio tasks + broadcast channel
- [ ] **Desktop option:** Tauri wrapper ใช้ React frontend ตัวเดียวกัน

### Nice-to-have
- [ ] Voice (TTS สำหรับ NPC dialogue)
- [ ] Campaign sharing / clone
- [ ] Mod system สำหรับ homebrew rules

**อ้างอิง:** [[improved-initiative]] = UI/UX reference สำหรับ combat tracker + deployment pattern (Docker, Node env vars)

**ออก Phase 4 เมื่อ:** 4 ผู้เล่น + 1 AI DM เล่นจบ 1 session ในเบราว์เซอร์

---

## Cross-cutting Concerns (ทุก Phase)

- **Testing:** Rules Engine ต้องมี unit test coverage สูง (dice, combat, condition)
- **Cost tracking:** log token usage ต่อ agent ต่อ session
- **Prompt caching:** ใช้ Anthropic cache สำหรับ wiki context (static) และ character state
- **Licensing:** ถ้า publish ต้องเคารพ SRD 5.2 (CC BY 4.0) และไม่ใช้เนื้อหานอก SRD

---

## Priorities ตอนนี้

Decisions ทั้งหมดปิดแล้ว (2026-04-17):
1. ✅ **Hybrid: Rust hot path + Python tooling**
2. ✅ **Vault นี้ = design-only**; campaigns แยก vault, dev ใช้ `campaigns/example/`
3. ✅ **Multiplayer-4 ตั้งแต่ day 1**
4. ✅ **Intent = strict JSON ผ่าน Claude tool use**
5. ✅ **Players เห็น dice numbers**

**Next:** เริ่ม Phase 1 — scaffold Cargo workspace + py-tools/, implement `dm-core::dice` + `dm-core::combat`, เปิด axum WebSocket room
