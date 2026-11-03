import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("cpu_run", Path(__file__).with_name("run.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class AssetPreflight(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.corpus = self.root / "corpus"
        self.corpus.mkdir()
        (self.corpus / "letter.txt").write_text("synthetic letter")
        self.gold = self.root / "gold.json"
        self.gold.write_text(json.dumps({"documents": [{"id": "letter", "file": "letter.txt",
                                                       "gold": {"expected_readiness": "either"}}]}))
        model = self.root / "model.gguf"
        model.write_bytes(b"test artifact, not model weights")
        self.pins = {"models": [{"file": model.name, "size": model.stat().st_size, "sha256": module.digest(model)}]}
        original_load = module.load
        self.mock = patch.object(module, "load", side_effect=lambda p: self.pins if Path(p).name == "candidates.json" else original_load(p))
        self.mock.start()
        self.addCleanup(self.mock.stop)

    def preflight(self):
        return module.preflight(self.root, self.corpus, self.gold)

    def test_hashes_valid_assets_and_accepts_deliberate_either_routing(self):
        pins, before = self.preflight()
        self.assertEqual(pins, self.pins["models"])
        (self.corpus / "letter.txt").write_text("changed synthetic letter")
        self.assertNotEqual(before, self.preflight()[1])

    def test_same_size_corrupt_model_fails(self):
        model = self.root / "model.gguf"
        model.write_bytes(b"x" * model.stat().st_size)
        with self.assertRaisesRegex(ValueError, "unverified model"):
            self.preflight()

    def test_missing_model_fails_before_any_execution(self):
        (self.root / "model.gguf").unlink()
        with self.assertRaisesRegex(ValueError, "unverified model"):
            self.preflight()

    def test_source_path_outside_corpus_fails(self):
        gold = json.loads(self.gold.read_text())
        gold["documents"][0]["file"] = "../model.gguf"
        self.gold.write_text(json.dumps(gold))
        with self.assertRaisesRegex(ValueError, "unsafe corpus"):
            self.preflight()

    def test_silent_readiness_skip_fails(self):
        gold = json.loads(self.gold.read_text())
        gold["documents"][0]["gold"]["expected_readiness"] = "review"
        self.gold.write_text(json.dumps(gold))
        with self.assertRaisesRegex(ValueError, "expected_readiness"):
            self.preflight()

    def test_duplicate_ids_fail(self):
        gold = json.loads(self.gold.read_text())
        gold["documents"] *= 2
        self.gold.write_text(json.dumps(gold))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.preflight()


class Pins(unittest.TestCase):
    def test_control_is_exactly_the_shipping_accurate_model(self):
        root = Path(__file__).resolve().parents[2]
        pins = json.loads(Path(__file__).with_name("candidates.json").read_text())
        shipping = json.loads((root / "src-tauri/resources/model-manifest.json").read_text())
        control = pins["models"][0]
        actual = next(m for m in shipping["models"] if m["tier"] == "accurate")["files"][0]
        self.assertEqual((control["file"], control["size"], control["sha256"]),
                         (actual["name"], actual["size"], actual["sha256"]))
        self.assertIn(control["revision"], actual["url"])


if __name__ == "__main__":
    unittest.main()
