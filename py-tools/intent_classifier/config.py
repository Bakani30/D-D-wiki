from dataclasses import dataclass, field
from pathlib import Path
from typing import List


BASE_MODEL = "xlm-roberta-base"

INTENT_LABELS: List[str] = [
    "attack",
    "ability_check",
    "cast_spell",
    "roleplay",
]


@dataclass
class TrainingConfig:
    base_model: str = BASE_MODEL
    labels: List[str] = field(default_factory=lambda: list(INTENT_LABELS))
    max_seq_length: int = 64
    batch_size: int = 32
    learning_rate: float = 5e-5
    num_epochs: int = 4
    weight_decay: float = 0.01
    seed: int = 42
    output_dir: Path = Path("py-tools/intent_classifier/models/checkpoints")
    train_path: Path = Path("py-tools/intent_classifier/dataset/synthetic/train.jsonl")
    val_path: Path = Path("py-tools/intent_classifier/dataset/synthetic/val.jsonl")


@dataclass
class SynthConfig:
    seed: int = 42
    total_samples: int = 5000
    labels: List[str] = field(default_factory=lambda: list(INTENT_LABELS))
    lexicon_path: Path = Path("py-tools/intent_classifier/dataset/dnd_lexicon.json")
    output_path: Path = Path("py-tools/intent_classifier/dataset/synthetic/dataset.jsonl")
    val_fraction: float = 0.1
