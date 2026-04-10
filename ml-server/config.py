from os import path
import os
import tomllib
from pathlib import Path
from dataclasses import dataclass
from platformdirs import user_config_dir


@dataclass
class ServerConfig:
    localhost: str = "0.0.0.0"
    port: int = 50032
    message_size_mb: int = 64
    model: str = "BAAI/bge-small-en-v1.5"
    version: int = 1
    dims: int = 384


def load_config() -> ServerConfig:
    config_dir = Path(user_config_dir("aetherium"))
    ml_toml = config_dir / "ml.toml"

    if not ml_toml.exists():
        return ServerConfig()

    with open(ml_toml, "rb") as f:
        data = tomllib.load(f)

    return ServerConfig(localhost=data.get("host", "0.0.0.0"),
                        port=data.get("port", 50032),
                        message_size_mb=data.get("message_size_mb", 64),
                        model=data.get("model", "BAAI/bge-small-en-v1.5"),
                        version=data.get("version", 1),
                        dims=data.get("dims", 384))


def make_dirs():
    os.makedirs(
        Path.home() / ".cache" / "aetherium" / "models", exist_ok=True)
