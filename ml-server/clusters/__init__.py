from typing import Literal, Optional
from dataclasses import dataclass
from clusters.error import ClusterModelNotFound, notfoundparameter_check
from hdbscan import HDBSCAN
from sklearn.base import ClusterMixin
import numpy as np

@dataclass
class ClusterConfig:
    model: Literal['hdbscan'] | Literal['dbscan'] | Literal['kmeans'] = "hdbscan"
    min_cluster_size: Optional[int] = None # for HDBSCAN
    min_samples: Optional[int] = None # for DBSCAN and HDBSCAN
    metric: Literal['euclidean'] | Literal['cosine'] = 'euclidean' # For Mostly every thing
    eps_distance: Optional[float] = None # for dbscan
    n_clusters: Optional[int] = None # for k means



class ClusterModel:
    def __init__(self, cluster_config: ClusterConfig, /):
        self.model = self.match_model(cluster_config)

    def fit(self, vectors: np.ndarray) -> np.ndarray:
        return self.model.fit_predict(vectors)

    def match_model(self, model_config: ClusterConfig) -> HDBSCAN | ClusterMixin:
        match model_config.model.lower():
            case "hdbscan":
                notfoundparameter_check(min_cluster_size = model_config.min_cluster_size)
                return HDBSCAN(min_cluster_size= model_config.min_cluster_size,
                               min_samples= model_config.min_samples,
                               metric=model_config.metric)

            case "kmeans":
                raise NotImplemented(f"{model_config.model} not implemented yet")
            case "dbscan":
                raise NotImplemented(f"{model_config.model} not implemented yet")
            case _:
                raise ClusterModelNotFound(model_config.model)
