"""Synthetic contract tests; live official-checkpoint checks are run separately.

  (cd embedding-code && python3 -m unittest test_harrier_server -v)
No books, credentials, private learner text or network models are used here.
"""

import hashlib
import http.client
import json
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

import harrier_server as server


class IntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.model_dir = self.root / "model"
        self.model_dir.mkdir()
        values = {name: b"fixture" for name in server.REQUIRED_FILES}
        values["config.json"] = json.dumps({"model_type": "qwen3", "hidden_size": 1024, "max_position_embeddings": 32768}).encode()
        values["config_sentence_transformers.json"] = json.dumps({"prompts": {"web_search_query": server.QUERY_PREFIX}}).encode()
        files = []
        for name, data in values.items():
            (self.model_dir / name).write_bytes(data)
            files.append({"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
        (self.model_dir / "source-revision.json").write_text(json.dumps({"repository": server.MODEL, "revision": server.REVISION}))
        self.manifest = self.root / "manifest.json"
        self.manifest.write_text(json.dumps({"models": [{"repository": server.MODEL, "revision": server.REVISION, "files": files}]}))

    def verify(self):
        server.verify_model(self.model_dir, self.manifest)

    def test_valid_manifest(self):
        self.verify()

    def test_corrupted_model_same_size(self):
        (self.model_dir / "model.safetensors").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            self.verify()

    def test_wrong_revision(self):
        (self.model_dir / "source-revision.json").write_text(json.dumps({"repository": server.MODEL, "revision": "wrong"}))
        with self.assertRaisesRegex(ValueError, "revision"):
            self.verify()

    def test_wrong_manifest_revision(self):
        data = json.loads(self.manifest.read_text())
        data["models"][0]["revision"] = "wrong"
        self.manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "manifest"):
            self.verify()

    def test_symlink_rejected(self):
        path = self.model_dir / "tokenizer.json"
        path.unlink()
        path.symlink_to(self.model_dir / "vocab.json")
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.verify()

    def test_unverified_alternative_weights_rejected(self):
        (self.model_dir / "pytorch_model.bin").write_bytes(b"different")
        with self.assertRaisesRegex(ValueError, "unexpected"):
            self.verify()

    def test_missing_manifest_files_rejected(self):
        data = json.loads(self.manifest.read_text())
        data["models"][0]["files"] = []
        self.manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "required"):
            self.verify()

    def test_path_escape_rejected(self):
        data = json.loads(self.manifest.read_text())
        data["models"][0]["files"][0]["path"] = "../outside"
        self.manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "path"):
            self.verify()


class EncoderTests(unittest.TestCase):
    def test_loader_is_local_cpu_and_single_load(self):
        from transformers import AutoModel, AutoTokenizer
        model = unittest.mock.Mock()
        model.dtype = torch.bfloat16
        with patch.object(server, "verify_model") as verify, patch.object(server, "progress"), patch.object(AutoModel, "from_pretrained", return_value=model) as load_model, patch.object(AutoTokenizer, "from_pretrained", return_value=object()) as load_tokenizer:
            encoder = server.HarrierEncoder(Path("synthetic-model"), 4)
        verify.assert_called_once_with(Path("synthetic-model"))
        load_model.assert_called_once_with("synthetic-model", local_files_only=True, trust_remote_code=False, use_safetensors=True, dtype="auto", attn_implementation="sdpa")
        load_tokenizer.assert_called_once_with("synthetic-model", local_files_only=True, trust_remote_code=False, fix_mistral_regex=False)
        model.to.assert_called_once_with("cpu")
        model.eval.assert_called_once()
        self.assertIs(encoder.model, model)

    def test_pool_both_padding_sides(self):
        hidden = torch.arange(12).reshape(2, 3, 2)
        right = torch.tensor([[1, 1, 0], [1, 1, 1]])
        left = torch.tensor([[0, 1, 1], [1, 1, 1]])
        self.assertTrue(torch.equal(server.last_token_pool(hidden, right), torch.stack([hidden[0, 1], hidden[1, 2]])))
        self.assertTrue(torch.equal(server.last_token_pool(hidden, left), hidden[:, -1]))

    def make_encoder(self, count=3):
        encoder = object.__new__(server.HarrierEncoder)
        self.seen = []
        self.model_calls = []

        def tokenize(texts, **kwargs):
            self.seen.append((texts, kwargs))
            return {"input_ids": [[1] * count for text in texts], "attention_mask": [[1] * count for text in texts]}

        def model(**kwargs):
            self.model_calls.append(kwargs)
            hidden = torch.arange(1, 1025, dtype=torch.bfloat16).reshape(1, 1, 1024).expand(1, count, 1024)
            return type("Output", (), {"last_hidden_state": hidden})()

        encoder.tokenizer = tokenize
        encoder.model = model
        return encoder

    def test_query_exact_prefix_passage_unchanged_float32_norm(self):
        encoder = self.make_encoder()
        for kind in ("query", "passage"):
            result = encoder.embed(["Example."], kind)
            self.assertEqual(self.seen[-1][0], [server.QUERY_PREFIX + "Example." if kind == "query" else "Example."])
            self.assertEqual(self.seen[-1][1], {"padding": False, "truncation": False, "add_special_tokens": True})
            self.assertEqual(result["token_counts"], [3])
            self.assertEqual(len(result["embeddings"][0]), 1024)
            norm = sum(value * value for value in result["embeddings"][0]) ** 0.5
            self.assertAlmostEqual(norm, 1.0, places=6)
            self.assertFalse(self.model_calls[-1]["use_cache"])
            self.assertEqual(result["profile"], server.PROFILE)

    def test_context_limit_preflight(self):
        encoder = self.make_encoder(32769)
        with self.assertRaisesRegex(ValueError, "context limit"):
            encoder.embed(["Example."], "query")
        self.assertEqual(self.model_calls, [])

    def test_exact_context_limit_accepted(self):
        encoder = self.make_encoder(32768)
        self.assertEqual(encoder.embed(["Example."], "passage")["token_counts"], [32768])

    def test_batch_order(self):
        encoder = self.make_encoder()
        result = encoder.embed(["First.", "Second."], "passage")
        self.assertEqual(result["token_counts"], [3, 3])
        self.assertEqual(len(self.model_calls), 2)

    def test_invalid_schema(self):
        for payload in (None, {}, {"texts": [], "kind": "query"}, {"texts": ["x"] * 9, "kind": "passage"}, {"texts": [1], "kind": "query"}, {"texts": [" "], "kind": "query"}, {"texts": ["x"], "kind": "other"}, {"texts": ["x"], "kind": "query", "extra": True}, {"texts": ["\ud800"], "kind": "query"}):
            with self.subTest(payload=repr(payload)):
                with self.assertRaises(ValueError):
                    server.validate_request(payload)


class FakeEncoder:
    def embed(self, texts, kind):
        if texts == ["over-limit"]:
            raise ValueError("context")
        if texts == ["failure"]:
            raise RuntimeError("do not leak text")
        return dict(server.identity(), embeddings=[[1.0] + [0.0] * 1023 for text in texts], token_counts=[1] * len(texts))


class HTTPTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.httpd = server.HarrierHTTPServer(("127.0.0.1", 0), FakeEncoder())
        cls.thread = threading.Thread(target=cls.httpd.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.httpd.shutdown()
        cls.httpd.server_close()
        cls.thread.join()

    def request(self, method, path, body=None, headers=None):
        connection = http.client.HTTPConnection("127.0.0.1", self.httpd.server_port, timeout=5)
        self.addCleanup(connection.close)
        connection.request(method, path, body, headers or {})
        response = connection.getresponse()
        return response.status, json.loads(response.read())

    def test_health_identity(self):
        status, result = self.request("GET", "/health")
        self.assertEqual(status, 200)
        for name, value in server.identity().items():
            self.assertEqual(result[name], value)
        self.assertEqual(result["context_limit"], 32768)

    def test_embed_contract_no_private_logs(self):
        with patch.object(server, "progress") as progress:
            status, result = self.request("POST", "/embed", json.dumps({"texts": ["synthetic private phrase"], "kind": "query"}), {"Content-Type": "application/json"})
        self.assertEqual(status, 200)
        self.assertEqual(len(result["embeddings"][0]), 1024)
        self.assertNotIn("synthetic private phrase", str(progress.call_args_list))

    def test_bytes_bound_before_read(self):
        status, result = self.request("POST", "/embed", b"", {"Content-Type": "application/json", "Content-Length": str(server.MAX_REQUEST_BYTES + 1)})
        self.assertEqual(status, 413)

    def test_invalid_json_and_schema(self):
        for body in (b"{", b"{}", b"\xff", json.dumps({"texts": ["x"] * 9, "kind": "query"}).encode()):
            self.assertEqual(self.request("POST", "/embed", body, {"Content-Type": "application/json"})[0], 400)

    def test_content_type_and_route(self):
        self.assertEqual(self.request("POST", "/embed", b"{}")[0], 415)
        self.assertEqual(self.request("GET", "/unknown")[0], 404)
        self.assertEqual(self.request("POST", "/unknown", b"{}")[0], 404)

    def test_transfer_encoding_rejected(self):
        self.assertEqual(self.request("POST", "/embed", b"{}", {"Transfer-Encoding": "chunked"})[0], 400)

    def test_context_and_safe_failure(self):
        for text, expected in (("over-limit", 422), ("failure", 500)):
            status, result = self.request("POST", "/embed", json.dumps({"texts": [text], "kind": "passage"}), {"Content-Type": "application/json"})
            self.assertEqual(status, expected)
            self.assertNotIn("do not leak", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
