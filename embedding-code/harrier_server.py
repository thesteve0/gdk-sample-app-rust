#!/usr/bin/env python3
"""Local official Harrier checkpoint service (CPU, sequential, no private-text logs).

Install requirements-harrier.txt in a separate Python 3.11+ environment. Launch:
  python3 embedding-code/harrier_server.py
Only loopback is served. Requests are capped at 8 texts and 2 MiB of JSON.
Token counts include tokenizer special tokens and the official query prefix.
Each text is inferred separately to avoid padding long inputs across the batch.
No truncation, network downloads, chat templates, or GGUF conversion is used.
The preparation manifest is required even for --model-dir overrides: copied model
files must match its hashes. All listed upstream files are verified; the unrelated
converted GGUF is deliberately not loaded or checked. Metadata/hashes establish
local integrity, not an independently signed upstream provenance attestation.
"""

import argparse
import hashlib
import json
import math
import sys
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Any, Dict, List, Tuple

MODEL = "microsoft/harrier-oss-v1-0.6b"
REVISION = "f9b9dc8d367d443f2479d27aa5d8d2850c0774ee"
DIMENSIONS = 1024
PROFILE = "harrier-hf-last-token-l2-web-search-v1"
CONTEXT_LIMIT = 32768
MAX_BATCH = 8
MAX_REQUEST_BYTES = 2 * 1024 * 1024
QUERY_PREFIX = "Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery: "
REPO_ROOT = Path(__file__).resolve().parent.parent
PREPARATION = REPO_ROOT / "day_trading" / "model-preparation"
DEFAULT_MODEL_DIR = PREPARATION / "models" / "harrier-0.6b"
MANIFEST = PREPARATION / "preparation-manifest.json"
REQUIRED_FILES = {
    "config.json", "config_sentence_transformers.json", "model.safetensors",
    "tokenizer.json", "tokenizer_config.json", "special_tokens_map.json",
    "added_tokens.json", "merges.txt", "vocab.json", "chat_template.jinja",
}


def progress(message: str) -> None:
    print("harrier: " + message, file=sys.stderr, flush=True)


def identity() -> Dict[str, Any]:
    return {"model": MODEL, "revision": REVISION, "dimensions": DIMENSIONS, "profile": PROFILE}


def regular_file(path: Path) -> None:
    if path.is_symlink() or not path.is_file():
        raise ValueError("required local file is missing, nonregular, or a symlink")


def verify_model(model_dir: Path, manifest_path: Path = MANIFEST) -> None:
    """Fail closed on revision, config, prompt or upstream artifact mismatch."""
    if model_dir.is_symlink() or not model_dir.is_dir():
        raise ValueError("model directory must be a real directory")
    regular_file(model_dir / "source-revision.json")
    metadata = json.loads((model_dir / "source-revision.json").read_text(encoding="utf-8"))
    if metadata != {"repository": MODEL, "revision": REVISION}:
        raise ValueError("source revision metadata does not match the pinned checkpoint")
    regular_file(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = [entry for entry in manifest["models"] if entry["repository"] == MODEL]
    if len(entries) != 1 or entries[0]["revision"] != REVISION:
        raise ValueError("preparation manifest does not match the pinned checkpoint")
    verified = set()
    for artifact in entries[0]["files"]:
        relative = Path(artifact["path"])
        if relative.is_absolute() or ".." in relative.parts or relative.as_posix() in verified:
            raise ValueError("invalid or duplicate manifest path")
        if relative.suffix == ".gguf":
            continue  # Only original HF files are used by this service.
        path = model_dir / relative
        for parent in relative.parents:
            if (model_dir / parent).is_symlink():
                raise ValueError("symlink in model artifact path")
        regular_file(path)
        if path.stat().st_size != artifact["bytes"]:
            raise ValueError("model artifact size mismatch")
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            while True:
                block = handle.read(1024 * 1024)
                if not block:
                    break
                digest.update(block)
        if digest.hexdigest() != artifact["sha256"]:
            raise ValueError("model artifact SHA-256 mismatch")
        verified.add(relative.as_posix())
    if not REQUIRED_FILES.issubset(verified):
        raise ValueError("manifest lacks required model/tokenizer files")
    # Prevent alternative local weights/configs from changing AutoModel selection.
    allowed = {Path(name).parts[0] for name in verified} | {"source-revision.json", ".cache", "harrier-0.6b-F16.gguf"}
    if any(path.name not in allowed for path in model_dir.iterdir()):
        raise ValueError("unexpected file in model directory")
    config = json.loads((model_dir / "config.json").read_text(encoding="utf-8"))
    if (config.get("model_type"), config.get("hidden_size"), config.get("max_position_embeddings")) != ("qwen3", DIMENSIONS, CONTEXT_LIMIT):
        raise ValueError("unexpected model architecture/dimensions/context")
    prompts = json.loads((model_dir / "config_sentence_transformers.json").read_text(encoding="utf-8"))
    if prompts.get("prompts", {}).get("web_search_query") != QUERY_PREFIX:
        raise ValueError("official web_search_query prompt mismatch")


def validate_request(payload: Any) -> Tuple[List[str], str]:
    if not isinstance(payload, dict) or set(payload) != {"texts", "kind"}:
        raise ValueError("expected exactly texts and kind fields")
    kind = payload["kind"]
    if kind not in ("passage", "query"):
        raise ValueError("kind must be passage or query")
    texts = payload["texts"]
    if not isinstance(texts, list) or not 1 <= len(texts) <= MAX_BATCH:
        raise ValueError("texts must contain 1 to 8 strings")
    if any(not isinstance(text, str) or not text.strip() for text in texts):
        raise ValueError("texts must be nonblank strings")
    try:
        for text in texts:
            text.encode("utf-8")
    except UnicodeEncodeError:
        raise ValueError("texts must be valid Unicode") from None
    return texts, kind


def last_token_pool(last_hidden_states: Any, attention_mask: Any) -> Any:
    """Official pooling: last real token for either left or right padding."""
    import torch
    if bool((attention_mask[:, -1].sum() == attention_mask.shape[0]).item()):
        return last_hidden_states[:, -1]
    sequence_lengths = attention_mask.sum(dim=1) - 1
    return last_hidden_states[torch.arange(last_hidden_states.shape[0], device=last_hidden_states.device), sequence_lengths]


class HarrierEncoder:
    def __init__(self, model_dir: Path, threads: int) -> None:
        progress("verifying pinned revision and local upstream SHA-256 hashes")
        verify_model(model_dir)
        import torch
        import transformers
        from transformers import AutoModel, AutoTokenizer
        torch.set_num_threads(threads)
        progress("loading official checkpoint once on CPU; torch={} transformers={}".format(torch.__version__, transformers.__version__))
        # Transformers 4.57.6 misidentifies this local Qwen3 tokenizer as Mistral.
        # Keep the verified upstream regex unchanged, rather than applying a
        # Mistral-specific rewrite (checked against raw tokenizers.Tokenizer).
        self.tokenizer = AutoTokenizer.from_pretrained(str(model_dir), local_files_only=True, trust_remote_code=False, fix_mistral_regex=False)
        self.model = AutoModel.from_pretrained(str(model_dir), local_files_only=True, trust_remote_code=False, use_safetensors=True, dtype="auto", attn_implementation="sdpa")
        self.model.to("cpu")
        self.model.eval()
        progress("model ready; dtype={} threads={}".format(self.model.dtype, threads))

    def embed(self, texts: List[str], kind: str) -> Dict[str, Any]:
        import torch
        import torch.nn.functional as functional
        prepared = [QUERY_PREFIX + text if kind == "query" else text for text in texts]
        # Preflight every input before any model work; never silently truncate.
        tokens = self.tokenizer(prepared, padding=False, truncation=False, add_special_tokens=True)
        counts = [len(ids) for ids in tokens["input_ids"]]
        if any(count < 1 or count > CONTEXT_LIMIT for count in counts):
            raise ValueError("token count exceeds the 32768-token context limit or is empty")
        embeddings = []
        with torch.inference_mode():
            for index in range(len(texts)):
                batch = {name: torch.tensor([values[index]], dtype=torch.long) for name, values in tokens.items()}
                outputs = self.model(**batch, use_cache=False)
                pooled = last_token_pool(outputs.last_hidden_state, batch["attention_mask"]).float()
                normalized = functional.normalize(pooled, p=2, dim=1)
                if normalized.shape != (1, DIMENSIONS) or not bool(torch.isfinite(normalized).all().item()):
                    raise RuntimeError("invalid embedding shape or nonfinite values")
                if not math.isclose(float(normalized.norm().item()), 1.0, abs_tol=1e-5):
                    raise RuntimeError("embedding is not unit normalized")
                embeddings.append(normalized[0].tolist())
                del outputs, pooled, normalized
        result = identity()
        result.update(embeddings=embeddings, token_counts=counts)
        return result


class HarrierHTTPServer(HTTPServer):
    def __init__(self, address: Tuple[str, int], encoder: Any) -> None:
        self.encoder = encoder
        super().__init__(address, HarrierHandler)


class HarrierHandler(BaseHTTPRequestHandler):
    server_version = "HarrierLocal/1"

    def setup(self) -> None:
        super().setup()
        self.connection.settimeout(30)

    def log_message(self, format: str, *args: Any) -> None:
        pass  # Do not log URLs, request bodies or exception messages.

    def respond(self, status: int, payload: Dict[str, Any]) -> None:
        data = json.dumps(payload, allow_nan=False, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(data)
        self.close_connection = True

    def do_GET(self) -> None:
        if self.path != "/health":
            self.respond(404, {"error": "not found"})
            return
        result = identity()
        result.update(status="ready", max_batch=MAX_BATCH, max_request_bytes=MAX_REQUEST_BYTES, context_limit=CONTEXT_LIMIT)
        self.respond(200, result)

    def do_POST(self) -> None:
        if self.path != "/embed":
            self.respond(404, {"error": "not found"})
            return
        if self.headers.get("Transfer-Encoding") is not None:
            self.respond(400, {"error": "transfer encoding is unsupported"})
            return
        lengths = self.headers.get_all("Content-Length", [])
        if len(lengths) != 1 or not lengths[0].isascii() or not lengths[0].isdigit():
            self.respond(411, {"error": "one valid Content-Length is required"})
            return
        # Avoid converting arbitrarily large decimal header values.
        if len(lengths[0]) > 10 or int(lengths[0]) > MAX_REQUEST_BYTES:
            self.respond(413, {"error": "request exceeds 2 MiB"})
            return
        length = int(lengths[0])
        if self.headers.get_content_type() != "application/json":
            self.respond(415, {"error": "Content-Type must be application/json"})
            return
        try:
            body = self.rfile.read(length)
            if len(body) != length:
                raise ValueError("incomplete body")
            payload = json.loads(body.decode("utf-8"))
            texts, kind = validate_request(payload)
        except (ValueError, UnicodeError, TimeoutError):
            self.respond(400, {"error": "invalid JSON body or request schema"})
            return
        started = time.monotonic()
        progress("embedding batch count={} kind={}".format(len(texts), kind))
        try:
            result = self.server.encoder.embed(texts, kind)
        except ValueError:
            self.respond(422, {"error": "input exceeds token context limit"})
            return
        except Exception as failure:
            progress("inference failed ({})".format(type(failure).__name__))
            self.respond(500, {"error": "embedding inference failed"})
            return
        progress("completed count={} elapsed={:.2f}s".format(len(texts), time.monotonic() - started))
        self.respond(200, result)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model-dir", type=Path, default=DEFAULT_MODEL_DIR)
    parser.add_argument("--port", type=int, default=8091)
    parser.add_argument("--threads", type=int, default=4)
    args = parser.parse_args()
    if not 1 <= args.port <= 65535 or not 1 <= args.threads <= 128:
        parser.error("port must be 1..65535; threads must be 1..128")
    try:
        encoder = HarrierEncoder(args.model_dir, args.threads)
        with HarrierHTTPServer(("127.0.0.1", args.port), encoder) as server:
            progress("serving sequential requests at http://127.0.0.1:{}".format(args.port))
            server.serve_forever()
    except KeyboardInterrupt:
        progress("stopped")
    except Exception as failure:
        progress("startup/server failure ({})".format(type(failure).__name__))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
