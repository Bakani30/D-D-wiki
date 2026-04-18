# D&D AI DM

A hybrid Rust + Python system that runs a persistent D&D 5e campaign with Claude as the Dungeon Master. The Rust core owns all deterministic game logic; Python handles offline tooling and the embedding microservice.

## Architecture

Claude emits an `Intent` via Anthropic tool-use (strict JSON schema). The Rust rules engine parses the Intent, rolls dice, applies 5e mechanics, and produces an `Effect`. The LLM never computes numbers directly. See `decisions.md` and `design/architecture.md` for the full rationale.

## Layout

```
Cargo.toml          # workspace root
crates/
  dm/               # CLI binary
  dm-core/          # rules engine, dice, combat, state (lib)
  dm-wiki/          # Obsidian vault read/write (lib)
  dm-claude/        # Anthropic client, Intent schemas (lib)
  dm-api/           # axum HTTP + WebSocket server (bin)
py-tools/
  ingest/           # SRD and source ingestion
  embed/            # embeddings microservice (HTTP)
  evals/            # evaluation harnesses
  notebooks/        # exploratory analysis
  requirements.txt
```

Rust and Python communicate only via HTTP or static YAML/JSON files. No FFI.

## Build

```sh
cargo build              # builds all crates
cargo build --release    # optimized
cargo test               # runs unit + proptest suites
```

Python tooling:

```sh
cd py-tools
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
```

## Key references

- `SKILL.md` — engineering standards and tech-stack rules
- `decisions.md` — Architecture Decision Records
- `CLAUDE.md` — campaign vault directive (schema + workflows)
- `active_task.md` — current phase objective
- `design/` — architecture, roadmap, memory system
