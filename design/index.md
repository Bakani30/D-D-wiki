# Design Index

Project-meta synthesis — **not** in-fiction world state. Read these to understand what we're building and why.

## Docs
- [[architecture]] — Intent/Effect, Multi-Model, Rules Engine, tech stack, full gameplay data flow
- [[roadmap]] — 4 phases: MVP → Agentic → Persistence → Web Platform
- [[memory-system]] — Wiki as Truth, Importance Decay, Consequence Seeds, State Inference
- [[references]] — Chronicler, dnd_ai_dm, Improved Initiative — what we borrow from each
- [[rules-5e-2024]] — 2024 d20 tests, 15 conditions, 12 actions, combat loop — engine mechanical scope
- [[adventure-pinebrook]] — Peril in Pinebrook as end-to-end reference scenario + test fixture
- [[prototype-1-summary]] — Prototype 1 milestone: Living Icicles CLI, dice-visible Intent/Effect loop, 69 tests

## Raw sources (read-only)
- [[plan]] — user's original roadmap & design philosophy (Thai)
- [[chronicler]] — Rust AI DM with persistent memory — closest reference for our architecture
- [[dnd_ai_dm]] — Python AI DM — reference for Phase 1 module layout
- [[improved-initiative]] — Node.js combat tracker — UI reference for Phase 4
- [[กฏฟรี D&D (2024)]] — free 2024 rules index (Thai, TOC/links to dnd-th.com)
- [[Peril_in_Pinebrook_COMPLETE1-1]] — WotC 2023 intro adventure (22 pages, 4 pregens)

## Decisions locked in (2026-04-17)
- ✅ **Language: Hybrid** — Rust hot path (`dm`, `dm-core`, `dm-wiki`, `dm-claude`, `dm-api`) + Python tooling (`py-tools/{ingest,embed,evals,notebooks}`)
- ✅ **Vault split** — this vault = design/project-meta only; campaigns live in separate vaults (dev at `campaigns/example/`)
- ✅ **Multiplayer** — 4 players from Phase 1 (axum WebSocket rooms)
- ✅ **Intent schema** — strict JSON via Claude tool use
- ✅ **Dice numbers** — shown to players in narrative

## Remaining minor question
- Dice roll log format for debug/replay — decide when implementing `dm-core::dice`
