from sklearn.decomposition import TruncatedSVD
from sklearn.preprocessing import Normalizer
from clusters.error import notfoundparameter_check
from clusters.reducers import BaseReducer, ReducerConfig
import numpy as np
from sklearn.pipeline import make_pipeline

class LSAReducer(BaseReducer):
    def __init__(self, config: ReducerConfig, /):
        notfoundparameter_check(n_components=config.n_components)
        self.model = make_pipeline(
                TruncatedSVD(n_components=config.n_components, random_state=321),
                Normalizer(copy=False)
                )

    def fit_transform(self, vectors: np.ndarray) -> np.ndarray:
        return self.model.fit_transform(vectors)
