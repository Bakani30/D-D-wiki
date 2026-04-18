from __future__ import annotations

import argparse
import json
import random
from pathlib import Path
from typing import Dict, List

from .config import SynthConfig


SKILLS = [
    "Athletics", "Acrobatics", "Sleight of Hand", "Stealth",
    "Arcana", "History", "Investigation", "Nature", "Religion",
    "Animal Handling", "Insight", "Medicine", "Perception", "Survival",
    "Deception", "Intimidation", "Performance", "Persuasion",
]

ABILITIES = ["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"]

DC_VALUES = [5, 10, 12, 13, 15, 17, 18, 20, 22, 25]

ROLEPLAY_TONES = [
    "อย่างสุภาพ", "อย่างระวัง", "อย่างมั่นใจ", "ด้วยน้ำเสียงเย็นชา",
    "ด้วยรอยยิ้ม", "อย่างประชดประชัน", "อย่างอ่อนโยน", "อย่างเกรี้ยวกราด",
    "เบา ๆ", "ตรงไปตรงมา",
]

ROLEPLAY_ACTIONS = [
    "ถามหาข้อมูลเกี่ยวกับเมืองนี้",
    "ขอซื้อเสบียงสำหรับการเดินทาง",
    "เล่าเรื่องในอดีตของตัวเอง",
    "ชวนคุยเรื่องดินฟ้าอากาศ",
    "ถามทางไปยังเมืองถัดไป",
    "เสนอความช่วยเหลือ",
    "ต่อรองราคาสินค้า",
    "ขอเข้าร่วมวงสนทนา",
    "เล่าเรื่องตลกเพื่อผ่อนบรรยากาศ",
    "ถามถึงข่าวสารล่าสุด",
    "ชมการแต่งตัวของเขา",
    "สารภาพความในใจ",
]


class PinebrookVocabulary:
    def __init__(self, lexicon_path: Path):
        self.lexicon_path = lexicon_path
        self.monsters: List[str] = []
        self.weapons: List[str] = []
        self.items: List[str] = []
        self.spells: List[str] = []
        self.monster_actions: List[str] = []

    def load(self) -> None:
        with self.lexicon_path.open("r", encoding="utf-8") as fh:
            data = json.load(fh)
        self.monsters = list(data.get("monsters", []))
        self.weapons = list(data.get("weapons", []))
        self.items = list(data.get("items", []))
        self.spells = list(data.get("spells", []))
        self.monster_actions = list(data.get("monster_actions", []))

    def is_loaded(self) -> bool:
        return bool(self.monsters and self.weapons and self.spells and self.items)


class IntentTemplateBank:
    def __init__(self, vocab: PinebrookVocabulary, rng: random.Random):
        self.vocab = vocab
        self.rng = rng

    def attack_templates(self) -> List[str]:
        return [
            "ฉันโจมตี {monster} ด้วย {weapon}",
            "ขอฟัน {monster} ด้วย {weapon}",
            "เข้าโจมตี {monster} ด้วย {weapon} ในมือ",
            "ใช้ {weapon} ฟาด {monster}",
            "ฟัน {monster} เต็มแรง",
            "ชัก {weapon} ออกมาแล้วโจมตี {monster}",
            "ฉันพุ่งเข้าไปฟัน {monster}",
            "เล็ง {monster} แล้วยิงด้วย {weapon}",
            "ฟาด {weapon} ใส่ {monster} ให้เต็มแรง",
            "ขอตีถึง {monster} ด้วย {weapon}",
            "โจมตี {monster} ตรงหน้า",
            "ขอตบหัว {monster} ด้วย {weapon}",
            "ชิงโจมตี {monster} ก่อนมันขยับ",
            "กระโดดตะลุมบอน {monster} ด้วย {weapon}",
            "ฉันเหวี่ยง {weapon} เข้าที่คอ {monster}",
            "ยิง {weapon} เข้าหา {monster}",
            "ฉันใช้ {weapon} ปักลงบน {monster}",
            "สวนหมัดด้วย {weapon} ใส่ {monster}",
            "เปิดศึกกับ {monster} ทันที",
            "ใช้ {weapon} เสียบเข้าให้ {monster}",
        ]

    def ability_check_templates(self) -> List[str]:
        return [
            "ขอทอย {skill} check",
            "ฉันขอเช็ค {skill}",
            "ลอง {skill} ดู",
            "ฉันอยากใช้ {skill} เพื่อดู {item}",
            "ขอ {skill} DC {dc}",
            "ทอย {ability} saving throw",
            "ฉันจะใช้ {skill} กับ {monster}",
            "ขอเช็ค {skill} ที่ประตู",
            "ฉันขอ {skill} ดูร่องรอย",
            "ตรวจสอบ {item} ด้วย {skill}",
            "ฉันลอบเข้าไปด้วย {skill}",
            "ขอ Perception หา {item} ในห้องนี้",
            "ใช้ Investigation กับ {item}",
            "ฉันขอโน้มน้าว {monster} ด้วย {skill}",
            "ลอง Insight อ่านใจ {monster}",
            "Survival หาทางออกจากป่านี้",
            "ขอปีน {item} ด้วย Athletics DC {dc}",
            "ทอย {ability} check ระดับ {dc}",
            "ฉันพยายามซ่อนตัวจาก {monster} ด้วย Stealth",
            "ขอใช้ Arcana ดู {spell} นี้คืออะไร",
        ]

    def cast_spell_templates(self) -> List[str]:
        return [
            "ฉันร่าย {spell} ใส่ {monster}",
            "ขอ cast {spell} ใส่ {monster}",
            "ใช้ {spell} โจมตี {monster}",
            "ร่าย {spell} ลงบน {monster}",
            "ฉันจะใช้ {spell} ในรอบนี้",
            "ขอร่าย {spell} เป็น bonus action",
            "cast {spell} ที่ระดับสูงขึ้น",
            "ใช้ spell slot ระดับ 2 ร่าย {spell}",
            "ร่าย {spell} ใส่ตัวเอง",
            "ร่าย {spell} ใส่เพื่อน",
            "ฉันหยิบคัมภีร์ออกมาร่าย {spell}",
            "สวดมนตร์ {spell} ใส่ {monster}",
            "ขอยิง {spell} เข้าใส่ {monster}",
            "ใช้ {spell} ป้องกัน {monster}",
            "ร่าย {spell} ครอบพื้นที่",
            "ฉัน concentrate ร่าย {spell}",
            "ขอ upcast {spell}",
            "ร่าย {spell} เพื่อรักษา",
            "ฉันปล่อย {spell} เต็มพลัง",
            "ใช้ {spell} ทำให้ {monster} ติดสถานะ",
        ]

    def roleplay_templates(self) -> List[str]:
        return [
            "ฉันเดินเข้าไปหา {monster} แล้ว{action}",
            "พูดกับ {monster} {tone} ว่า {action}",
            "ฉัน{action}{tone}",
            "ฉันทักทาย {monster} {tone}",
            "ฉันชวน {monster} คุยเรื่อง {item}",
            "ขอเล่าเรื่อง {item} ให้ {monster} ฟัง",
            "ฉันยื่น {item} ให้ {monster} {tone}",
            "หันไปหา {monster} แล้ว{action}",
            "ฉันโค้งคำนับ {monster} {tone}",
            "นั่งลงข้าง {monster} แล้ว{action}",
            "ฉันเปิดบทสนทนาด้วยการ{action}",
            "ฉันแอบกระซิบกับ {monster} {tone}",
            "ฉันเล่าเรื่องของตัวเองให้ {monster} ฟัง",
            "พยายาม{action}กับ {monster}",
            "หยิบ {item} ขึ้นมาดู{tone}",
            "ฉันแค่{action}โดยไม่ใช้พลัง",
            "ฉันหยุดและสังเกต {monster}",
            "ฉันหายใจลึกแล้ว{action}",
            "ฉันก้มลงมอง {item} อย่างครุ่นคิด",
            "ขอใช้เวลาสักครู่เพื่อ{action}",
        ]

    def render_attack(self) -> str:
        template = self.rng.choice(self.attack_templates())
        return template.format(
            monster=self.rng.choice(self.vocab.monsters),
            weapon=self.rng.choice(self.vocab.weapons),
        )

    def render_ability_check(self) -> str:
        template = self.rng.choice(self.ability_check_templates())
        return template.format(
            skill=self.rng.choice(SKILLS),
            ability=self.rng.choice(ABILITIES),
            dc=self.rng.choice(DC_VALUES),
            item=self.rng.choice(self.vocab.items),
            monster=self.rng.choice(self.vocab.monsters),
            spell=self.rng.choice(self.vocab.spells),
        )

    def render_cast_spell(self) -> str:
        template = self.rng.choice(self.cast_spell_templates())
        return template.format(
            spell=self.rng.choice(self.vocab.spells),
            monster=self.rng.choice(self.vocab.monsters),
        )

    def render_roleplay(self) -> str:
        template = self.rng.choice(self.roleplay_templates())
        return template.format(
            monster=self.rng.choice(self.vocab.monsters),
            item=self.rng.choice(self.vocab.items),
            tone=self.rng.choice(ROLEPLAY_TONES),
            action=self.rng.choice(ROLEPLAY_ACTIONS),
        )


class SyntheticDataGenerator:
    def __init__(self, config: SynthConfig):
        self.config = config
        self.rng = random.Random(config.seed)
        self.vocab = PinebrookVocabulary(lexicon_path=config.lexicon_path)
        self.templates = IntentTemplateBank(vocab=self.vocab, rng=self.rng)
        self.renderers = {
            "attack": self.templates.render_attack,
            "ability_check": self.templates.render_ability_check,
            "cast_spell": self.templates.render_cast_spell,
            "roleplay": self.templates.render_roleplay,
        }

    def prepare(self) -> None:
        self.vocab.load()
        if not self.vocab.is_loaded():
            raise RuntimeError(f"lexicon missing required keys: {self.config.lexicon_path}")
        missing = [l for l in self.config.labels if l not in self.renderers]
        if missing:
            raise ValueError(f"no renderer for labels: {missing}")

    def generate_for_label(self, label: str, target: int, seen: set) -> List[Dict]:
        render = self.renderers[label]
        rows: List[Dict] = []
        attempts = 0
        max_attempts = target * 200
        while len(rows) < target and attempts < max_attempts:
            attempts += 1
            text = render()
            if text in seen:
                continue
            seen.add(text)
            rows.append({"text": text, "label": label})
        if len(rows) < target:
            raise RuntimeError(
                f"could not generate {target} unique samples for '{label}' "
                f"(got {len(rows)} after {attempts} attempts)"
            )
        return rows

    def generate_all(self) -> List[Dict]:
        per_label = self.config.total_samples // len(self.config.labels)
        remainder = self.config.total_samples - per_label * len(self.config.labels)
        seen: set = set()
        rows: List[Dict] = []
        for i, label in enumerate(self.config.labels):
            quota = per_label + (1 if i < remainder else 0)
            rows.extend(self.generate_for_label(label, quota, seen))
        self.rng.shuffle(rows)
        return rows

    def write_jsonl(self, rows: List[Dict], path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, ensure_ascii=False) + "\n")

    def run(self) -> Path:
        self.prepare()
        rows = self.generate_all()
        self.write_jsonl(rows, self.config.output_path)
        return self.config.output_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate synthetic intent data")
    parser.add_argument("--total-samples", type=int, default=None)
    parser.add_argument("--output-path", type=Path, default=None)
    parser.add_argument("--lexicon-path", type=Path, default=None)
    parser.add_argument("--seed", type=int, default=None)
    parser.add_argument("--val-fraction", type=float, default=None)
    return parser.parse_args()


def build_config(args: argparse.Namespace) -> SynthConfig:
    cfg = SynthConfig()
    if args.total_samples is not None:
        cfg.total_samples = args.total_samples
    if args.output_path is not None:
        cfg.output_path = args.output_path
    if args.lexicon_path is not None:
        cfg.lexicon_path = args.lexicon_path
    if args.seed is not None:
        cfg.seed = args.seed
    if args.val_fraction is not None:
        cfg.val_fraction = args.val_fraction
    return cfg


def main() -> None:
    args = parse_args()
    config = build_config(args)
    generator = SyntheticDataGenerator(config)
    out = generator.run()
    print(f"wrote {config.total_samples} rows to {out}")


if __name__ == "__main__":
    main()
