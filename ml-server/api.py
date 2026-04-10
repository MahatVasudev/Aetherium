import grpc
import sys
from concurrent import futures
import aetherium_ml_pb2_grpc
import argparse
from config import load_config, make_dirs
from servicer import MLServicer


def serve(port: int):
    config = load_config()
    make_dirs()
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=2), options=[
        ('grpc.max_receive_message_length',
         config.message_size_mb * 1024 * 1024),
        ('grpc.max_send_message_length',
         config.message_size_mb * 1024 * 1024),
    ])
    print("running till here")
    aetherium_ml_pb2_grpc.add_AetheriumMLServiceServicer_to_server(
        MLServicer(config=config), server)
    print("running till here")
    server.add_insecure_port(f"[::]:{config.port}")
    print("running till here")
    server.start()
    print(f"ML Server started at port {
          config.port}, max message size {config.message_size_mb}...")
    server.wait_for_termination()


if __name__ == "__main__":

    try:
        parser = argparse.ArgumentParser()  # Not needed now
        parser.add_argument("--port", type=int,
                            default=50032)  # Not needed now
        args = parser.parse_args()  # Not needed now
        serve(args.port)  # Not needed now
    except KeyboardInterrupt as ke:
        print("Keyboard Interrupted... ML Server Exiting", ke)
        sys.exit(1)

    except MemoryError as me:
        print("Out of Memory, ML Server Exiting", me)
        sys.exit(1)

    except Exception as e:
        print("Exiting Due to Error: ", e)
        sys.exit(1)
