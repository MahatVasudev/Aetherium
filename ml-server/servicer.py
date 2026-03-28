from fastembed import TextEmbedding
import aetherium_ml_pb2_grpc
import aetherium_ml_pb2

DEFAULT_MODEL_NAME = "BAAI/bge-small-en-v1.5"
DEFAULT_VERSION = 1
DEFAULT_DIMS = 384

class MLServicer(aetherium_ml_pb2_grpc.AetheriumMLServiceServicer):
    def __init__(self, model_name: str = DEFAULT_MODEL_NAME, version: int = DEFAULT_VERSION, dims: int = DEFAULT_DIMS):
        assert version >= 1
        assert dims > 100
        self.model_name = model_name
        self.version = str(version)
        self.dims = dims
        self.model = TextEmbedding(model_name)

    def Health(self, request, context):
        return aetherium_ml_pb2.HealthResponse(status="ok",
                                               version = self.version, 
                                               model = self.model_name, 
                                               dims = self.dims)

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

        return aetherium_ml_pb2.EmbedQueryResponse(vector = result.tolist())

