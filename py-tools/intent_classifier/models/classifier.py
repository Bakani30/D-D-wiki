from __future__ import annotations

from pathlib import Path
from typing import Dict, List

from transformers import AutoModelForSequenceClassification, PreTrainedModel


class MiniLMIntentClassifier:
    def __init__(self, base_model: str, labels: List[str]):
        self.base_model = base_model
        self.labels = labels
        self.label2id: Dict[str, int] = {l: i for i, l in enumerate(labels)}
        self.id2label: Dict[int, str] = {i: l for l, i in self.label2id.items()}
        self._model: PreTrainedModel | None = None

    def build(self) -> PreTrainedModel:
        raise NotImplementedError

    def save(self, output_dir: Path) -> None:
        raise NotImplementedError

    @classmethod
    def load(cls, checkpoint_dir: Path) -> "MiniLMIntentClassifier":
        raise NotImplementedError
