# Campaign & Project Log

Append-only. Newest at the bottom.

## [Meta-Ingest 2026-04-17] | Synthesized raw/ (plan + chronicler + dnd_ai_dm + improved-initiative) into design/ (architecture, roadmap, memory-system, references); added key-concepts + Meta-Ingest workflow to CLAUDE.md
## [Decision 2026-04-17] | Language: Rust (primary); Cargo workspace = dm / dm-core / dm-wiki / dm-claude; Chronicler = primary architecture reference
## [Decision 2026-04-17] | Revised to hybrid — Rust hot path (adds dm-api crate) + Python off-path (py-tools/{ingest,embed,evals,notebooks}); Python embed is optional sidecar with keyword fallback
## [Decision 2026-04-17] | Multiplayer: 4 players from Phase 1 via axum WebSocket session rooms
## [Decision 2026-04-17] | Intent schema: strict JSON via Claude tool-use API (typed Rust enum via serde)
## [Decision 2026-04-17] | Players see dice numbers embedded in narrative (e.g., "Attack: 1d20+5 = 18 vs AC 15, hit")
## [Decision 2026-04-17] | Vault scope: this vault = design/project-meta only; campaigns live in separate vaults (dev at campaigns/example/); CLAUDE.md doubles as template directive for campaign vaults
## [Meta-Ingest 2026-04-17] | Synthesized raw/ (2024 rules TOC + Peril in Pinebrook) into design/rules-5e-2024.md (engine scope: d20 tests, 15 conditions, 12 actions, combat loop) and design/adventure-pinebrook.md (pregen+monster schema, DC calibration, 4 encounter patterns for test fixture)
## [Prototype 1 2026-04-17] | Living Icicles encounter runnable via CLI — Intent/Effect/dice visible end-to-end; dm-core gains entity/intent/effect/combat/scenario modules, dm-wiki gains fixtures+session writer, `cargo run --bin dm -- encounter pinebrook --seed 42` produces deterministic session01.md
## [Decision 2026-04-17] | PIVOT — Local-First AI Stack: deprecated Anthropic API dependency; Intent produced by custom ML classifier (HuggingFace/PyTorch in py-tools/), narrative + OOD routing by Local LLM (llama.cpp/Ollama); Rust↔Python/LLM via localhost HTTP; no network calls during runtime play
