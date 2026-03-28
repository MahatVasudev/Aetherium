
# ML Server (Python)

## Implements 
- Query Embedding
- Make Chunk wise Document Embedding

---

Communicates through **GRPC**, mainly from Aetherium-Engine

---

## Design Choices

**Embedding-Model Dependency**

- `fastembed` NOT `torch`

**REASON**: Torch is a big dependency, since we are running the whole system in the users computer, 
we are using fastembed as they are much lighter, but we are trading this for much less selection of models, and therefore trading accuracy

---

## For Development

Create a local venv

```bash
python -m venv .venv # create a local venv
source .venv/bin/activate # for linux or mac (bash or zsh)
source .venv/bin/activate.fish # for linux or mac (fish)
.venv\Scripts\Activate.ps1 # for windows powershell
.venv\Scripts\activate.bat # for windows command prompt
```

For installing dependency
```bash
pip install -r requirements.txt # its a must
pip install -r requirements-gpu.txt # optional only if you want your gpu to be used
```

Generate GRPC Code for python

```bash
python -m grpc_tools.protoc \   
-I./proto \
--python_out=./grpc_proto \
--grpc_python_out=./grpc_proto \
proto/aetherium_ml.proto
```
though it is not needed, as it will be added with the repository, only needed if anything is changed
