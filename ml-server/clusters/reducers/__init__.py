from typing import Literal, Optional
import numpy as np
from abc import ABC, abstractmethod
from dataclasses import dataclass

class BaseReducer(ABC):
    @abstractmethod
    def fit_transform(self, vectors: np.ndarray) -> np.ndarray:
        ...


@dataclass
class ReducerConfig:
    reducer: Literal['umap','lda','lsa','none'] = 'none'
    n_components: Optional[int] = None
    n_neighbors: Optional[int] = None 
    min_dist: Optional[float] = None
    metric: Optional[Literal['cosine'] | Literal['euclidean']] = None
    learning_method: Optional[Literal['online']] = None
    max_iter: Optional[int] = None
