from clusters.error import ClusterModelNotFound
from fastembed import TextEmbedding
import aetherium_ml_pb2_grpc
import aetherium_ml_pb2
from clusters.reducers import ReducerConfig
from clusters import ClusterConfig
from clusters.pipeline import Pipeline
from pathlib import Path

from config import ServerConfig

DEFAULT_MODEL_NAME = "BAAI/bge-small-en-v1.5"
DEFAULT_VERSION = 1
DEFAULT_DIMS = 384


class MLServicer(aetherium_ml_pb2_grpc.AetheriumMLServiceServicer):
    def __init__(self, config: ServerConfig):
        assert config.version >= 1
        assert config.dims > 100
        self.model_name = config.model
        self.version = str(config.version)
        self.dims = config.dims
        self.model = TextEmbedding(
            self.model_name,
            cache_dir=str(
                Path.home() / ".cache" / "aetherium" / "models")
        )

    def Health(self, request, context):
        return aetherium_ml_pb2.HealthResponse(status="ok",
                                               version=self.version,
                                               model=self.model_name,
                                               dims=self.dims)

    def EmbedBatch(self, request, context):
        texts = [chunk.text for chunk in request.chunks]
        vectors = list(self.model.embed(texts))
        embeddings = [
            aetherium_ml_pb2.ChunkEmbedding(
                chunk_id=chunk.chunk_id,
                doc_id=chunk.doc_id,
                vector=vector.tolist(),
            )
            for chunk, vector in zip(request.chunks, vectors)
        ]
        return aetherium_ml_pb2.EmbedBatchResponse(embeddings=embeddings)

    def EmbedQuery(self, request: aetherium_ml_pb2.EmbedQueryRequest, context):
        result = next(self.model.query_embed(request.query))

        return aetherium_ml_pb2.EmbedQueryResponse(vector=result.tolist())

    def Cluster(self, request, context):
        chunk_ids = [c.chunk_id for c in request.chunks]
        reducer_method = request.WhichOneof('reducer_config')
        cluster_method = request.WhichOneof('cluster_config')
        vectors, reducer_config = get_reducer_stuff(reducer_method, request)

        cluster_stuff = get_cluster_stuff(cluster_method, request)

        pipeline = Pipeline(cluster_stuff, reducer_config)

        assignments = pipeline.run_pipeline(
            chunk_ids=chunk_ids, vectors=vectors)

        return aetherium_ml_pb2.ClusterResponse(
            assignments=[
                aetherium_ml_pb2.ChunkCluster(
                    chunk_id=chunk_id,
                    cluster_id=cluster_id
                )

                for chunk_id, cluster_id in assignments
            ],
            n_clusters=len(
                set(label for _, label in assignments if label != -1))
        )


def get_cluster_stuff(method: str, request) -> ClusterConfig:
    if method == "hdbscan":
        return ClusterConfig(
            model=method,
            min_cluster_size=request.hdbscan.min_cluster_size,
            min_samples=request.hdbscan.min_samples,
            metric=request.hdbscan.metric
        )

    elif method == "dbscan":
        raise NotImplementedError()
    elif method == "kmeans":
        raise NotImplementedError()

    raise ClusterModelNotFound(method)


def get_reducer_stuff(method: str, request) -> tuple[list, ReducerConfig]:

    if method == 'umap':
        vectors = [list(c.embedding.values) for c in request.chunks]
        reducer_config = ReducerConfig(
            reducer=method,
            n_components=request.umap.n_components,
            n_neighbors=request.umap.n_neighbors,
            min_dist=request.umap.min_distance,
            metric=request.umap.metric
        )

        return vectors, reducer_config
    elif method == 'lda':
        vectors = [list(c.tfidf.values) for c in request.chunks]
        reducer_config = ReducerConfig(
            reducer=method,
            n_components=request.lda.n_components,
            max_iter=request.lda.max_iter,
            learning_method=request.lda.learning_method
        )

        return vectors, reducer_config

    elif method == 'lsa':
        vectors = [list(c.tfidf.values) for c in request.chunks]
        reducer_config = ReducerConfig(
            reducer=method,
            n_components=request.lsa.n_components,
        )

        return vectors, reducer_config
    else:
        vectors = [list(c.embedding.values) for c in request.chunks]
        return vectors, ReducerConfig(reducer="none")
