import grpc
from concurrent import futures
import aetherium_ml_pb2_grpc
import argparse
from servicer import MLServicer

def serve(port: int):
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=2))
    aetherium_ml_pb2_grpc.add_AetheriumMLServiceServicer_to_server(MLServicer(), server)
    server.add_insecure_port(f"[::]:{port}")
    server.start()
    print(f"ML Server started at port {port}...")
    server.wait_for_termination()

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=50032)
    args = parser.parse_args()
    serve(args.port)
