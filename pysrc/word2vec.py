import os
import pickle
from functools import lru_cache
import re
from typing import Dict

import gdown
import numpy as np

WORDS_URL = (
    "https://drive.google.com/uc?id=1hJVHw0gdh9itJBtAn8ZPJGxP2jRe332k"
    # https://drive.google.com/file/d/1hJVHw0gdh9itJBtAn8ZPJGxP2jRe332k/view?usp=sharing
)
WORDS_PATH = os.path.join(
    os.path.dirname(__file__), os.path.pardir, "data", "words.pkl"
)
EMBEDDING_LEN = 300

def _download_words():
    os.makedirs(os.path.dirname(WORDS_PATH), exist_ok=True)
    gdown.download(WORDS_URL, WORDS_PATH)


@lru_cache()
def load_word_vectors() -> Dict[str, np.ndarray]:
    if not os.path.exists(WORDS_PATH):
        _download_words()

    with open(WORDS_PATH, "rb") as f:
        words, vectors = pickle.load(f)
    return {w.lower(): v for w, v in zip(words, vectors.astype(np.float64))}


def cosine_similarity(v1: np.ndarray, v2: np.ndarray) -> float:
    out = np.dot(v1, v2)
    return abs(round(out.item() * 100, 2))


def embed_sentences(sentences: list[str]) -> list[np.ndarray]:
    vectors = load_word_vectors()
    # split on spaces, underscore, brackets, hyphens, and hashtags
    words_list = [re.split(r"[\s\(\)\[\]\{\}_\-#]", s) for s in sentences] 
    embeddings = []
    for words in words_list:
        # normalise: trim, remove empty strings, and lowercase
        words = [w.strip().lower() for w in words if w.strip()]
        vecs = [vectors[w] for w in words if w in vectors]
        if not vecs:
            embeddings.append(np.zeros(EMBEDDING_LEN))
            continue
        vec = np.mean(vecs, axis=0)
        embeddings.append(vec)
    return embeddings
