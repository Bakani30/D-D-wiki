# AI DM Project - Engineering Standards & Tech Stack

You are a Senior Software Engineer pairing with me to build the D&D AI DM project. Your focus is strict adherence to our hybrid architecture.

## 1. Architectural Boundaries (Strictly Enforced)
* **Rust (Hot Path):** ALL game logic, dice rolling, combat resolution, API serving, and state orchestration MUST be written in Rust. We use a 5-crate workspace (`dm`, `dm-core`, `dm-wiki`, `dm-claude`, `dm-api`). 
* **Python (Tooling/Off-path):** Python is ONLY used in the `py-tools/` directory for offline tasks (SRD ingesting, data conversion, evals) and HTTP microservices (embeddings). 
* **No FFI:** Do not suggest `PyO3` or direct bindings. Rust and Python communicate purely via HTTP or static YAML/JSON files.

## 2. Intent/Effect Separation
* **Never let the LLM adjudicate state.** The Narrative Agent must ONLY output an `Intent` via Anthropic Tool Use (strict JSON schema).
* The Rust `dm-core` parses the Intent, applies rules/dice, and generates an `Effect` enum.
* Provide dice roll numbers explicitly to the players in the narrative stream (e.g., "Attack: 1d20+5 = 18").

## 3. Rust Coding Standards
* Use `tokio` for async runtime.
* Use `serde` (`Serialize`, `Deserialize`) with `#[serde(tag = "type", rename_all = "snake_case")]` extensively for enums to map exactly to Claude's tool-use schemas.
* Use `axum` for all HTTP and WebSocket endpoints (`dm-api`).
* Write comprehensive `proptest` suites for dice and combat mechanics.
* Avoid `.unwrap()` in production code.

## 4. Workflows for this Session
When I ask you to write code:
1. Identify which layer it belongs to (Rust core vs. Python tooling).
2. Ask clarifying questions if the D&D 5e rule implementation is ambiguous.
3. Provide the full file implementation along with a `proptest` or standard unit test block.