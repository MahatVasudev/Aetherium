from typing import Any, Optional
from clusters.constants import AVAILABLE_MODELS


class ClusterModelNotFound(Exception):
    def __init__(self, model_name: str, message: str = "Specified Model Not Found"):
        self.model_name = model_name
        self.message = message + "="*10 + \
            f"\nAvailable Models are {AVAILABLE_MODELS}"
        super().__init__(self.message)

    def __str__(self) -> str:
        return self.message


class ParameterNotFound(Exception):
    def __init__(self, model_param: str, message: str = "The parameter for the clustering model is not found"):
        self.model_param = model_param
        self.message = message + "="*10 + \
            f"the parameter in question {model_param}"
        super().__init__(self.message)

    def __str__(self) -> str:
        return self.message


def notfoundparameter_check(**kwargs: Optional[Any]):
    for name, arg in kwargs.items():
        if arg is None:
            raise ParameterNotFound(name)
