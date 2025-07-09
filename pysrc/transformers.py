from sentence_transformers import SentenceTransformer
import numpy as np
import os

from torch import Tensor
os.environ["TOKENIZERS_PARALLELISM"] = "false"

model = SentenceTransformer("all-MiniLM-L6-v2")
EMBEDDING_LEN = 384

model.max_seq_length = 256

def embed_sentences(sentences: list[str]) -> list[Tensor]:
    return model.encode(sentences, normalize_embeddings=True) # type: ignore

def cosine_similarity(v1: Tensor, v2: Tensor) -> float:
    out = np.dot(v1, v2)
    return abs(round(out.item() * 100, 2))