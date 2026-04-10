from sklearn.decomposition import LatentDirichletAllocation
import numpy as np
from clusters.error import notfoundparameter_check
from clusters.reducers import BaseReducer, ReducerConfig

class LDAReducer(BaseReducer):
    def __init__(self, config: ReducerConfig, /):
        notfoundparameter_check(
                n_components=config.n_components,
                max_iter=config.max_iter,
                learning_method=config.learning_method
                )
        self.model = LatentDirichletAllocation(
                n_components=config.n_components,
                max_iter=config.max_iter,
                random_state=42,
                learning_method=config.learning_method
                )

    def fit_transform(self, vectors: np.ndarray) -> np.ndarray:
        return self.model.fit_transform(vectors)
