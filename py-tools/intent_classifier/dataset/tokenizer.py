from __future__ import annotations

from typing import Dict, List

from datasets import Dataset
from transformers import AutoTokenizer, PreTrainedTokenizerBase


class IntentTokenizer:
    def __init__(self, base_model: str, max_seq_length: int):
        self.base_model = base_model
        self.max_seq_length = max_seq_length
        self._tokenizer: PreTrainedTokenizerBase | None = None

    def load(self) -> PreTrainedTokenizerBase:
        raise NotImplementedError

    def tokenize_batch(self, batch: Dict[str, List]) -> Dict[str, List]:
        raise NotImplementedError

    def tokenize_dataset(self, dataset: Dataset) -> Dataset:
        raise NotImplementedError
