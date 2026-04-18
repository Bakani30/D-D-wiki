---
type: design
name: References
description: External projects we draw inspiration from, and what we borrow from each
status: draft
sources: ["[[chronicler]]", "[[dnd_ai_dm]]", "[[improved-initiative]]"]
tags: [design, references]
---

# Reference Projects

3 โปรเจกต์ที่เราจะดูดไอเดียมาใช้ — แต่ละโปรเจกต์เด่นคนละด้าน

---

## 1. Chronicler (SamuelSchlesinger/chronicler)

**สโลแกน:** *"An AI Dungeon Master that remembers."*
**Stack:** Rust (Bevy + egui desktop app), Claude API
**License:** CC BY-NC 4.0

### ทำไมสำคัญ
นี่คือ **reference implementation ที่ใกล้ที่สุด** กับสิ่งที่เราอยากสร้าง — architecture pattern copy ได้แทบตรง

### สิ่งที่ยืม
- ✅ **Multi-Model split** (Sonnet narrative / Haiku inference + relevance)
- ✅ **Intent/Effect separation** pattern
- ✅ **Importance decay** (2%/turn) — ตัวเลขเอาไปใช้ได้เลย
- ✅ **Consequence seeds** concept + semantic matching
- ✅ **Post-narrative state inference** with confidence threshold
- ✅ **Proactive world design** (pre-populate NPC/secret ก่อนผู้เล่นถึง)

### สิ่งที่จะทำต่าง
- 🔄 Storage เริ่มด้วย Obsidian Markdown (human-readable) แทน custom format — DM มนุษย์เข้าแก้ได้
- 🔄 Phase 4 ไปเป็น web (multiplayer) ไม่ใช่ desktop solo — ใช้ `axum` server + Tauri wrapper สำหรับ desktop option
- 🔄 Frontend React (ของเรา) vs Bevy+egui (ของเขา) — เราเลือก React เพราะ reusable ระหว่าง web และ Tauri

### ลิงก์
- Repo: https://github.com/SamuelSchlesinger/chronicler
- Deep dive: `docs/HOW_IT_WORKS.md` ในรีโปนั้น
- Transcript ตัวอย่าง: Goblin Ambush, Wizard's Bargain

---

## 2. dnd_ai_dm (msadeqsirjani/dnd_ai_dm)

**สโลแกน:** AI-Powered D&D Game Master (console-based)
**Stack:** Python 3.8+, OpenAI GPT

### ทำไมสำคัญ
**Inspiration เรื่อง separation of concerns** — เราเลือก Rust ไม่ใช่ Python แต่การแบ่ง module (core / models / utils) ยังใช้ได้เป็น mental model ตอนออกแบบ Cargo workspace

### สิ่งที่ยืม (ในรูปแบบ Rust)
- ✅ Separation of concerns: rules (`dm-core::combat`, `dm-core::dice`) vs entities (`dm-core::model`) vs I/O (`dm-wiki`, `dm-claude`)
- ✅ CLI-first development (ง่ายต่อการทดสอบ iterate) — `cargo run -p dm`
- ❌ ไม่ copy code ตรงๆ เพราะคนละภาษา; Chronicler ใกล้กว่าในเชิง implementation

### ข้อจำกัดที่เราต้องแก้
- ❌ ไม่มี structured memory — เก็บแค่ world_state ธรรมดา (เราจะเสริม importance decay + consequence seeds)
- ❌ Single-model (GPT) — เราจะ multi-model
- ❌ ไม่มี Intent/Effect separation — เราต้องเพิ่ม

### ลิงก์
- Repo: https://github.com/msadeqsirjani/dnd_ai_dm

---

## 3. Improved Initiative (cynicaloptimist/improved-initiative)

**สโลแกน:** Combat tracker for D&D 5e
**Stack:** Node.js + TypeScript, React, MongoDB, Docker
**License:** MIT
**Live:** https://improvedinitiative.app/

### ทำไมสำคัญ
**UI/UX reference สำหรับ Combat Tracker** ใน Phase 4 — เป็น tool ที่ DM ใช้งานจริงมานานแล้ว

### สิ่งที่ยืม
- ✅ Combat tracker UX (initiative order, HP bar, condition tags)
- ✅ Deployment pattern (Docker, env var config: `PORT`, `NODE_ENV`, `DB_CONNECTION_STRING`, `SESSION_SECRET`)
- ✅ Patreon-style account tiering (ถ้าจะทำเป็น service)
- ✅ Stat block + monster library UI

### สิ่งที่จะทำต่าง
- 🔄 ของเขาเป็น **tracker manual** (DM มนุษย์กดเอง) — ของเราต้อง **auto-update** จาก Rules Engine
- 🔄 เราต้องผูก tracker กับ narrative stream (เมื่อ AI บรรยายว่า goblin ตาย, tracker ต้อง auto-remove)

### ลิงก์
- Repo: https://github.com/cynicaloptimist/improved-initiative
- Live app: https://improvedinitiative.app/

---

## สรุป: Matrix

| Feature | Chronicler | dnd_ai_dm | Improved Initiative | เรา (target) |
|---|---|---|---|---|
| Intent/Effect separation | ✅ | ❌ | N/A | ✅ |
| Multi-model | ✅ | ❌ | N/A | ✅ |
| Persistent memory | ✅ | partial | N/A | ✅ |
| Importance decay | ✅ | ❌ | N/A | ✅ |
| Consequence seeds | ✅ | ❌ | N/A | ✅ |
| Wiki/Markdown storage | ❌ | ❌ | ❌ | ✅ (unique) |
| Combat tracker UI | basic | ❌ | ✅ | ✅ (Phase 4) |
| Multiplayer web | ❌ | ❌ | ✅ (solo DM) | ✅ (Phase 4) |
| Language | Rust | Python | TS/Node | **Rust (hot path) + Python (tooling/ML) + React (Phase 4)** |
| Multiplayer | ❌ solo | ❌ solo | manual tracker | ✅ 4 players ตั้งแต่ Phase 1 (axum WebSocket) |
| Intent schema | JSON | loose | N/A | **Strict via Claude tool use** |
| Dice visibility | narrative-only | varies | manual | **Numbers shown to players** |

**Edge ของเรา** เทียบกับ Chronicler: human-editable Obsidian wiki (DM มนุษย์เข้าแทรกแซงได้) + multiplayer web platform (Phase 4)
**ช่องว่าง** ที่ต้องเติมเอง: Wiki ↔ Rules integration (`dm-wiki` crate) + axum web API + React UI + multiplayer session management

**Primary reference:** [[chronicler]] (ภาษาเดียวกัน, pattern เดียวกัน)
**Secondary:** [[dnd_ai_dm]] (module decomposition idea เท่านั้น)
**UI reference:** [[improved-initiative]] (Phase 4 frontend only)
