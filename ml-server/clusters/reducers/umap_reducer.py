from clusters.error import notfoundparameter_check
from clusters.reducers import BaseReducer, ReducerConfig
from umap import UMAP
import numpy as np


class UMAPReducer(BaseReducer):
    def __init__(self, config: ReducerConfig, /) -> None:
        notfoundparameter_check(n_components=config.n_components,
                                n_neighbors=config.n_neighbors,
                                min_distance=config.min_dist,
                                metric=config.metric)
        self.model = UMAP(n_components=config.n_components,
                          n_neighbors=config.n_neighbors,
                          min_dist=config.min_dist,
                          metric=config.metric,
                          random_state=312)

    def fit_transform(self, vectors: np.ndarray) -> np.ndarray:
        return self.model.fit_transform(vectors)
