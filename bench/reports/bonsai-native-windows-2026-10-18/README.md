# Native Windows Bonsai comparison

Read the [evaluation and recommendation](../../../docs/bonsai-native-windows-evaluation.md) and [qualification procedure](../../../docs/native-model-qualification.md).

- `comparison.json`: matched metrics, per-case accepted outputs, timing, process startup and Windows model peak working sets.
- `*-synthetic-raw.jsonl`: twelve synthetic cases per model, one JSON object per line. Each contains extracted source, raw model request/response and accepted result. The exported object removes only the report's local `native_report` file path. Original audit-input hashes are recorded in the comparison.
- `*-semantic-audit.md`: independent source-based findings. Filename scores and lexical description metrics do not establish factual fidelity.
- `matched-profile.json`: shared settings/prompt values. `bonsai2-frozen-profile.json` also retains the earlier selection metadata. Selected profiles and Qwen/4B calibration scores are included separately.
- `plans/`: all twelve exact native plans and their profile files. Plans verify runtime/model/executable/fixture hashes; binaries and model weights are not committed. New builds require a newly reviewed plan and hashes, rather than bypassing verification.
- `build-provenance.json`: reviewed source hashes, toolchain metadata and test/build log hashes.
- `sha256.json`: SHA-256 of every published evidence file except itself.

These are actual Windows CPU results on a four-GiB/two-vCPU guest. All documents and examples are fictional; no private documents, API credentials, VM disks or security event logs are included. Original raw captures and complete Defender logs remain preserved in the local evaluation workspace. Calibration `source` fields and build-environment metadata deliberately retain original local task-asset paths for provenance; these are generated evaluation paths, not private document paths or credentials.

Verify published bytes from the repository root:

```sh
python3 - <<'PYVERIFY'
import hashlib, json
from pathlib import Path
root = Path('bench/reports/bonsai-native-windows-2026-10-18')
for name, expected in json.loads((root / 'sha256.json').read_text()).items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
print('Evidence hashes verified')
PYVERIFY
```
