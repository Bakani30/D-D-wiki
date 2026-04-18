# Current Task: Prototype 2 — Scaffold Local-First AI Pipeline

**Status:** Not Started
**Goal:** วางโครงสร้าง ML training pipeline ใน `py-tools/` สำหรับ Intent Classifier และตั้ง interface กับ Local LLM (llama.cpp / Ollama) — แทนที่แผน Anthropic API เดิม ตาม ADR "Local-First AI Stack" (2026-04-17)

## Architectural Pivot (2026-04-17)
- ❌ **Deprecated:** Intent via Anthropic Tool Use API
- ✅ **New:** Self-Hosted ML (Intent Classifier) + Local LLM (Router & Narrator)
- Intent JSON schema (Rust enum + serde) ยังคงเป็น contract กลาง — เปลี่ยนแค่ producer

## Prototype 1 — Done (ไม่กระทบจาก pivot)
- dm-core / dm-wiki / dm CLI เสร็จแล้ว, 69 tests ผ่าน
- Living Icicles encounter runnable end-to-end, deterministic, dice-visible
- การ pivot มีผลตั้งแต่ Prototype 2 เท่านั้น

---

## Tasks for Prototype 2

### 1. Scaffold `py-tools/intent_classifier/` (ใหม่)
โครงสร้างขั้นต่ำ:
```
py-tools/intent_classifier/
  pyproject.toml           # หรือใช้ root requirements.txt + setup.cfg
  README.md                # how to train, evaluate, serve
  schema/
    intent.json            # export จาก Rust enum (serde_json::to_writer) — single source of truth
  data/
    synthetic/             # template-generated training data
    pinebrook/             # labelled player chat จากการเล่นจริง / session01.md replay
    raw/                   # unlabelled chat (ถ้ามี)
  src/intent_classifier/
    __init__.py
    schema.py              # pydantic models mirroring Rust Intent enum
    dataset.py             # HuggingFace Dataset loader + train/val/test split
    model.py               # DistilBERT / MiniLM + classification head
    train.py               # CLI: python -m intent_classifier.train
    evaluate.py            # CLI: intent accuracy, confusion matrix, latency
    serve.py               # FastAPI/uvicorn HTTP server on localhost:PORT
  tests/
    test_schema_parity.py  # Rust enum ↔ Python pydantic round-trip
    test_training_smoke.py # 10-sample train runs without crash
```

### 2. Intent Schema Parity Bridge
- เพิ่ม `crates/dm-core/src/bin/dump_intent_schema.rs` → เขียน `intent.json` (JSON Schema ของ Intent enum)
- Python `schema.py` โหลด `intent.json` → generate pydantic models ตอน test (parity check)
- CI test: Rust เปลี่ยน Intent enum → schema.json เปลี่ยน → Python test fail จนกว่าจะ sync

### 3. Synthetic Data Generator
- `py-tools/intent_classifier/src/intent_classifier/synth.py`
- Template-based: "{actor} attacks {target}" × N วิธีพูด → label = `Attack`
- ครอบคลุม Intent enum ตอนนี้ (`Attack`, `EndTurn`) + 12 actions ตาม [[rules-5e-2024]] (เตรียมไว้ขยาย)
- Output: `data/synthetic/train.jsonl` + `val.jsonl`

### 4. Training Loop (MVP)
- Base model: `distilbert-base-uncased` หรือ `sentence-transformers/all-MiniLM-L6-v2` + classification head
- Multi-task head: intent type (categorical) + slot filling (attacker/target entity) ถ้าขยายภายหลัง
- ตั้ง target: ≥95% accuracy บน synthetic + ≥85% บน Pinebrook labelled (initial)
- Log ลง `py-tools/evals/` (มีโฟลเดอร์อยู่แล้ว)

### 5. Local LLM Setup Doc
- สร้าง `design/local-llm-setup.md`:
  - ทางเลือก: **Ollama** (dev ง่าย) vs **llama.cpp** (เร็วกว่า, control มากกว่า)
  - Model recommendations: Llama 3.1 8B Q4 / Qwen 2.5 7B / Mistral 7B — optimized GGUF for Apple Silicon (Metal)
  - HTTP API contract: OpenAI-compatible `/v1/chat/completions` (ทั้ง Ollama และ llama.cpp รองรับ)
  - Prompt template สำหรับ narrator (scene → prose) และ router (chat → Intent JSON เมื่อ classifier confidence ต่ำ)

### 6. Rescope `dm-claude` crate
- Rename plan: `dm-claude` → `dm-infer` (ไว้ทำจริงใน step 7)
- ตอนนี้แค่เพิ่ม `crates/dm-claude/README.md` อธิบายว่า crate นี้จะเปลี่ยนเป็น local-inference client
- Dependencies ที่คาดว่าใช้: `reqwest` (HTTP), `serde_json`, `tokio`
- ยังไม่ต้อง implement — วางแค่ module skeleton ใน step 7

### 7. Rust ↔ Python/LLM Transport (skeleton เท่านั้น)
- `crates/dm-claude/src/lib.rs` export trait `IntentProvider`:
  ```rust
  #[async_trait]
  pub trait IntentProvider {
      async fn classify(&self, utterance: &str, ctx: &SceneContext) -> Result<Intent, ProviderError>;
  }
  ```
- Impls วางไว้ทีหลัง: `HttpClassifierProvider` (ต่อ py-tools sidecar), `LlamaCppProvider` (ต่อ llama.cpp HTTP)

---

## Out of Scope (ยกไป Prototype 3+)
- Narrative LLM integration (รอให้ Local LLM setup เสถียรก่อน)
- Multiplayer / WebSocket (`dm-api`)
- Full 15-condition system, spell system, movement/grid
- Vault ingestion, consequence seeds, importance decay
- Production model training (เริ่มจาก smoke test + synthetic data ก่อน)

---

## Definition of Done
- [ ] `py-tools/intent_classifier/` มี package skeleton + `pyproject.toml` + README
- [ ] `dump_intent_schema` binary export Intent JSON Schema จาก Rust
- [ ] Python `schema.py` โหลด schema + parity test ผ่าน
- [ ] Synthetic data generator produce ≥100 training examples per Intent variant
- [ ] `python -m intent_classifier.train` run ได้ (smoke test ไม่ crash; accuracy ยังไม่ต้อง target)
- [ ] `design/local-llm-setup.md` ครอบคลุม Ollama + llama.cpp + model recommendations + HTTP contract
- [ ] `crates/dm-claude/` มี README อธิบาย rescope + trait skeleton (ไม่ต้อง impl จริง)
- [ ] `log.md` มี entry: `## [Prototype 2 YYYY-MM-DD] | Local-First AI pipeline scaffolded — intent classifier skeleton + local LLM contract`

---

## Constraint
- **Intent schema เป็น source of truth** — อย่าให้ Rust กับ Python drift; parity test ต้องมี
- **Dice / mechanics เข้าใกล้ LLM ไม่ได้** — LLM รับ `Effect` ที่ resolved แล้ว; LLM ผลิตได้แค่ narrative prose + Intent (classifier ผลิต Intent เป็นหลัก, LLM router เป็น fallback เฉพาะ OOD)
- **No Anthropic API calls in runtime play path** — ADR lock
- **Python tooling runs as sidecar** — Rust process เป็น primary; Python เป็น subprocess/HTTP service; ไม่มี FFI / PyO3 ใน Prototype 2

## Open Questions (ถามก่อน implement)
1. Base model ตัวแรก — **DistilBERT** (classification-first, proven) หรือ **MiniLM** (เล็กกว่า, faster inference, embedding-based)?
2. Ollama vs llama.cpp สำหรับ dev setup — เลือกอันไหนเป็น default ใน setup doc? (Ollama ง่ายกว่ามาก แต่ llama.cpp ยืดหยุ่นกว่า)
3. Training data — เริ่มจาก synthetic เท่านั้น หรือ manually label Pinebrook session01.md transcript ตั้งแต่แรก?
