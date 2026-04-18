Architecture Decision Records (ADR)

ไฟล์นี้เก็บบันทึกการตัดสินใจเชิงเทคนิคที่สำคัญสำหรับโปรเจกต์ D&D AI DM ห้ามเสนอทางเลือกที่ขัดแย้งกับข้อตกลงที่ได้รับอนุมัติแล้ว (Accepted)
[2026-04-17] Hybrid Stack Decision

    Status: Accepted

    Decision: ใช้แนวทาง Hybrid — Rust สำหรับ Hot path (Game Engine) และ Python สำหรับ Off-path (Tooling)

    Structure: * Rust: แยกเป็น 5 crates (dm, dm-core, dm-wiki, dm-claude, dm-api) เพื่อรองรับประสิทธิภาพและการขยายตัว

        Python: อยู่ใน py-tools/ สำหรับงาน Ingest, Evaluation และ Embedding

    Reason: ต้องการประสิทธิภาพในการตัดสินกฎกติกา (Deterministic) และความปลอดภัยของ Type ในส่วนของ Core เกม

[2026-04-17] Intent as Tool Use (Strict Schema) — Anthropic API

    Status: Deprecated (superseded by "Local-First AI Stack" 2026-04-17)

    Decision: การกระทำเชิงกลไกทั้งหมดของ AI DM ต้องส่งออกเป็น JSON ผ่าน Anthropic Tool Use API เท่านั้น

    Implementation: ใช้ Rust Enum พร้อม serde เพื่อแปลง Tool-use JSON เป็นข้อมูลที่ Rules Engine อ่านได้ทันที

    Reason: ป้องกัน AI ทำงานผิดพลาดหรือคำนวณตัวเลขเอง (Hallucination)

    Why deprecated: Pivot ไปใช้ Self-Hosted ML + Local LLM เพื่อ (1) ลด latency และ cost ต่อ session, (2) กำจัด network dependency ระหว่างเล่น, (3) privacy — chat ของผู้เล่นไม่ต้องออกนอกเครื่อง. Intent schema (Rust enum + serde) ยังคงเดิม แต่ producer เปลี่ยนจาก Claude tool-use → custom NLP classifier (in-distribution) + Local LLM router (out-of-distribution fallback).

[2026-04-17] Local-First AI Stack

    Status: Accepted

    Decision: ยกเลิกการพึ่ง Anthropic API สำหรับ runtime narrative/intent. เปลี่ยนมาใช้สถาปัตยกรรม Self-Hosted ML + Local LLM ทั้งหมด:

        1. Intent Classifier (Custom ML): โมเดล NLP เล็กเร็ว (fine-tuned transformer ขนาด ~100M params ระดับ DistilBERT/MiniLM, หรือ sentence-embedding + classifier head) train ด้วย Hugging Face + PyTorch ใน py-tools/. หน้าที่: parse player chat → Intent JSON ที่ Rust enum รับได้โดยตรง. รันเป็น sidecar process (HTTP หรือ stdio) ในฝั่ง Python.

        2. Local LLM Router & Narrator: Llama-family / Mistral / Qwen quantized (GGUF) รันผ่าน llama.cpp หรือ Ollama, optimized for Apple Silicon (Metal / MLX). หน้าที่: (a) out-of-distribution intent routing เมื่อ classifier confidence ต่ำ, (b) narrative generation (scene description, NPC dialogue, outcome flavor).

        3. Rust ↔ Python/LLM Transport: dm-core → HTTP ไปยัง local ports (intent classifier + LLM server). ไม่มี network call ออกนอกเครื่องในช่วง runtime play.

    Consequence: dm-claude crate ถูก rescope — จากเดิม Anthropic SDK wrapper → ตอนนี้เป็น local-inference client (HTTP). ตั้งชื่อใหม่ได้ภายหลัง (dm-brain / dm-llm / dm-infer). Intent enum + tool-schema ยังคงเป็น source of truth ระหว่าง classifier และ Rust.

    Reason:
        - Latency: local inference < 100ms ต่อ intent parse; network API round-trip 300–2000ms
        - Cost: $0 per-session vs. API token cost; enables long campaigns and stress-test loops
        - Privacy: player chat ไม่ออกจากเครื่อง — สำคัญสำหรับ campaign data และ personal role-play content
        - Offline: รันเล่นได้ไม่ต้องต่อเน็ต
        - Control: fine-tune intent classifier ให้เก่งเฉพาะ D&D action vocabulary; ไม่โดน model-version drift จาก API vendor

    Trade-offs ที่ยอมรับ:
        - Narrative quality ต่ำกว่า frontier API models (Opus/Sonnet) — ยอมรับสำหรับ MVP; ถ้า local narrator ไม่พอ จะพิจารณา hybrid (local intent + optional cloud narrator) ใน phase หลัง
        - Setup burden — ผู้ใช้ต้องมี Llama.cpp/Ollama ติดตั้ง + model weights — documentation ต้องครอบคลุม
        - Training pipeline ต้องมี labelled dataset — เริ่มจาก synthetic data + Pinebrook scenario transcripts + manual annotation

    ขอบเขต Prototype 1 (เสร็จแล้ว 2026-04-17): ไม่กระทบ — P1 ไม่มี LLM เลย. การ pivot นี้มีผลตั้งแต่ Prototype 2 เป็นต้นไป.

[2026-04-17] Multiplayer-First (4 Players)

    Status: Accepted

    Decision: รองรับผู้เล่น 4 คนต่อ 1 ห้อง Session ตั้งแต่ Phase 1 ผ่าน WebSocket (ใช้ axum)

    Reason: เพื่อให้โครงสร้างพื้นฐานรองรับการสเกลเป็น Web Platform ใน Phase 4 ได้โดยไม่ต้องรื้อระบบใหม่

[2026-04-17] Dice Visibility

    Status: Accepted

    Decision: ผู้เล่นต้องเห็นตัวเลขการทอยเต๋าจริงที่คำนวณจาก Rules Engine แทรกอยู่ในคำบรรยาย (เช่น "Attack: 1d20+5 = 18")

    Reason: เพื่อความโปร่งใสและสร้างอารมณ์ร่วมแบบการเล่น Tabletop RPG จริง

[2026-04-17] Vault & Campaign Isolation

    Status: Accepted

    Decision: แยก Project Vault (สำหรับการออกแบบ/วิศวกรรม) ออกจาก Campaign Vault (เนื้อเรื่อง/ข้อมูลเกม) อย่างเด็ดขาด

    Implementation: ไฟล์ CLAUDE.md ใน Project Vault จะทำหน้าที่เป็นแม่แบบ (Template Directive) สำหรับ Campaign Vault อื่นๆ

    Reason: ป้องกันข้อมูลเนื้อเรื่องปนเปื้อนกับสถาปัตยกรรมระบบ และเพื่อให้หนึ่งระบบจัดการได้หลายแคมเปญ