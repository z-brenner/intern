"""Qualification must reject incomplete or incomparable evidence, even when scores improve."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("cpu_compare", Path(__file__).with_name("compare.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class QualificationEvidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.gold = self.root / "gold.json"
        self.gold.write_text(json.dumps({"documents": [{"id": "letter", "gold": {"expected_readiness": "ready"}}]}))
        self.dirs = [self.root / name for name in ("a1", "b1", "a2", "b2")]
        for index, directory in enumerate(self.dirs):
            directory.mkdir()
            model = "control" if index % 2 == 0 else "candidate"
            report = {"schema_version": 1, "suite": "internbench", "mode": "live", "timings_source": "measured",
                      "git_commit": "commit", "model": {"sha256": model},
                      "corpus": {"gold_sha256": module.digest(self.gold)}, "machine": {"cpu": "test"},
                      "records": [{"id": "letter", "status": "completed", "readiness": "ready",
                                   "timings": {"inference_ms": 100, "total_ms": 120},
                                   "scores": {"date_correct": True, "parties_correct": index % 2 == 1,
                                              "readiness_match": True, "date_forbidden": False,
                                              "party_forbidden": False, "unsafe_ready": False}}]}
            provenance = {k: "same" for k in ("runtime_sha256", "parser_runtime_sha256", "worker_sha256", "bench_sha256",
                                               "corpus_sha256", "profile_sha256", "server_args", "cgroup_cpu_max", "cgroup_memory_max")}
            provenance.update({"threads": 4, "git_commit": "commit", "gold_sha256": module.digest(self.gold),
                               "model_sha256": model, "exit_code": 0})
            for name, data in (("report.json", report), ("provenance.json", provenance), ("rss.json", {"peak_total_mib": 4000})):
                (directory / name).write_text(json.dumps(data))
            (directory / "startup.txt").write_text("startup_s 2.5\n")

    def update(self, run, file, change):
        path = self.dirs[run] / file
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def result(self):
        return module.compare(self.gold, self.dirs)

    def test_repeated_gain_is_screening_only(self):
        result = self.result()
        self.assertTrue(result["screening_pass"])
        self.assertFalse(result["release_qualified"])

    def test_duplicate_repeat_is_not_evidence(self):
        with self.assertRaisesRegex(ValueError, "distinct"):
            module.compare(self.gold, [self.dirs[0], self.dirs[1]] * 2)

    def test_failed_model_cannot_win_by_filing_nothing(self):
        self.update(1, "report.json", lambda d: d["records"][0].update(status="model_failed"))
        with self.assertRaisesRegex(ValueError, "incomplete"):
            self.result()

    def test_incomplete_cohort_rejected(self):
        self.update(1, "report.json", lambda d: d.update(records=[]))
        with self.assertRaisesRegex(ValueError, "cohort"):
            self.result()

    def test_replay_cannot_measure_new_model(self):
        self.update(1, "report.json", lambda d: d.update(mode="replay"))
        with self.assertRaisesRegex(ValueError, "live"):
            self.result()

    def test_missing_score_cannot_shrink_denominator(self):
        self.update(1, "report.json", lambda d: d["records"][0]["scores"].pop("date_correct"))
        with self.assertRaisesRegex(ValueError, "denominator"):
            self.result()

    def test_incompatible_runtime_rejected(self):
        self.update(1, "provenance.json", lambda d: d.update(runtime_sha256="different"))
        with self.assertRaisesRegex(ValueError, "runtime_sha256"):
            self.result()

    def test_missing_memory_is_not_zero(self):
        self.update(1, "rss.json", lambda d: d.update(peak_total_mib=0))
        with self.assertRaisesRegex(ValueError, "memory"):
            self.result()

    def test_infinite_latency_rejected(self):
        self.update(1, "report.json", lambda d: d["records"][0]["timings"].update(inference_ms=float("inf")))
        with self.assertRaisesRegex(ValueError, "latency"):
            self.result()

    def test_unrecognized_gold_readiness_rejected(self):
        self.gold.write_text(json.dumps({"documents": [{"id": "letter", "gold": {"expected_readiness": "review"}}]}))
        with self.assertRaisesRegex(ValueError, "expected_readiness"):
            self.result()

    def test_every_repeat_must_gain(self):
        self.update(3, "report.json", lambda d: d["records"][0]["scores"].update(parties_correct=False))
        self.assertFalse(self.result()["screening_pass"])

    def test_unsafe_or_trap_date_fails(self):
        for metric in ("unsafe_ready", "date_forbidden", "party_forbidden"):
            with self.subTest(metric=metric):
                self.update(1, "report.json", lambda d: d["records"][0]["scores"].update({metric: True}))
                self.assertFalse(self.result()["screening_pass"])
                self.update(1, "report.json", lambda d: d["records"][0]["scores"].update({metric: False}))

    def test_memory_and_speed_budgets_fail(self):
        self.update(1, "rss.json", lambda d: d.update(peak_total_mib=13000))
        self.update(1, "report.json", lambda d: d["records"][0]["timings"].update(inference_ms=200))
        reasons = self.result()["reasons"]
        self.assertTrue(any("12 GiB" in r for r in reasons))
        self.assertTrue(any("1.5x" in r for r in reasons))


if __name__ == "__main__":
    unittest.main()
