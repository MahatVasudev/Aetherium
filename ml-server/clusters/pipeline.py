from typing import Optional
import numpy as np

from clusters import ClusterConfig, ClusterModel
from clusters.reducers import BaseReducer, ReducerConfig
from clusters.reducers.lda_reducer import LDAReducer
from clusters.reducers.lsa_reducer import LSAReducer
from clusters.reducers.umap_reducer import UMAPReducer


class Pipeline:
    def __init__(self, clusterConfig: ClusterConfig, reducerConfig: ReducerConfig):
        self.clusterConfig = clusterConfig
        self.reducerConfig = reducerConfig

    def get_reducer(self) -> Optional[BaseReducer]:
        match self.reducerConfig.reducer:
            case 'umap':
                return UMAPReducer(self.reducerConfig)

            case 'lda':
                return LDAReducer(self.reducerConfig)

            case 'lsa':
                return LSAReducer(self.reducerConfig)

            case 'none':
                return None

            case _:
                raise ValueError(f"Reducer Not Found... {
                                 self.reducerConfig.reducer}")

    def run_pipeline(self, chunk_ids: list[str], vectors: list[list[float]]):
        matrix = np.array(vectors)

        reducer = self.get_reducer()

        if reducer is not None:

            matrix = reducer.fit_transform(matrix)

        model = ClusterModel(self.clusterConfig)

        labels = model.fit(matrix)

        return list(zip(chunk_ids, labels.tolist()))
