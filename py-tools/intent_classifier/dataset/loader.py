from __future__ import annotations

from pathlib import Path
from typing import Dict, List, Tuple

from datasets import Dataset, DatasetDict


class IntentDatasetLoader:
    def __init__(self, labels: List[str]):
        self.labels = labels
        self.label2id: Dict[str, int] = {l: i for i, l in enumerate(labels)}
        self.id2label: Dict[int, str] = {i: l for l, i in self.label2id.items()}

    def load_jsonl(self, path: Path) -> List[Dict]:
        raise NotImplementedError

    def encode_labels(self, rows: List[Dict]) -> List[Dict]:
        raise NotImplementedError

    def to_hf_dataset(self, rows: List[Dict]) -> Dataset:
        raise NotImplementedError

    def load_split(self, train_path: Path, val_path: Path) -> DatasetDict:
        raise NotImplementedError
