# Indra OCR reference goldens

Intern's Indra engine (`crates/intern-worker/src/indra/`) is a Rust port of
Datalab's `indra-ocr` 1.1.2. These scripts run the reference Python package to
make the files the port is tested against: the CLI's own output, per-stage
values, the first-call tensors of every network, and language-model vectors.

The goldens are not committed (they run to tens of megabytes); the four small
JSON lines files for Intern's own fixtures are, in
`crates/intern-worker/tests/fixtures/indra/latin-only/`.

## Pins

| What | Pin |
|---|---|
| Code | `github.com/datalab-to/indra` commit `0e96b93c6810f17cf8352acbb4487cc9e45b66d5` ("Indra OCR 1.1.2"); PyPI `indra-ocr==1.1.2` is byte-identical |
| Models | `huggingface.co/datalab-to/indra-ocr` tag `v1.1.0` = commit `a1283c4b42a273c09480ad50e71461daff50fcc7`; SHA-256 of every file in `fetch_models.sh` |
| Runtime | `requirements.txt`: Python 3.12, onnxruntime 1.31.0, opencv-python-headless 5.0.0.93, numpy 2.5.4, pypdfium2 5.14.0 |

OpenCV's version changes the reference's own output (its `warpAffine` rounding
changed between 4.x and 5.0), so a golden made with another wheel is not a
golden. Every run uses one ONNX Runtime thread.

## Making them

Never run these from inside a checkout of the `indra` repository, and keep the
`-I` on every `python`: a script run next to the package's source would import
it in place of the pinned install.

```sh
python3.12 -m venv ~/.cache/indra-venv
~/.cache/indra-venv/bin/pip install -r bench/tools/indra/requirements.txt

bench/tools/indra/fetch_models.sh ~/.cache/indra-models

# The pages, in one directory:
#   document-image.{png,jpg,tiff}, rotated-low-resolution-scan.png,
#   scanned-lease.pdf, mixed-signature.pdf, vendor-invoice.pdf
#     <- fixtures/generated/ (npm run fixtures)
#   multi_column.png, rotated_letter.png, table_pmc.png, tilted_memo.png
#     <- assets/examples/ of the indra repository at the pinned commit
#   scan.png
#     <- examples/samples/ of the same
bench/tools/indra/regenerate.sh ~/.cache/indra-venv ~/.cache/indra-models \
  ~/indra-inputs ~/indra-reference
```

SHA-256 of the pages the committed goldens were made from:

```
72fb556d97f4496ba416fc4659d5d2ec3134addb3626dcbcb1b029b51c3b1a59  document-image.jpg
3581da8a5018524713b74353a7866b413fc177e8c36b1302ea67c2880e07b900  document-image.png
0d092e8da079b04807700b8881b5ae120b3a13294012b46d91f8c80b9b30d53b  document-image.tiff
27ef06096a9b8ad47288fd7f05f073e21976c594a10cc6507fdbd89e4da50f4f  rotated-low-resolution-scan.png
41890ce82fcce3181122f9b821b0d87b7145793171249413aac40e5efa49807f  multi_column.png
06927fb2206e291936fe79c97a53a7409617b28ef47bedffae54a964c1a12dfd  rotated_letter.png
92a06a8ce1666ee20aad458fb20e17b7d6a372a2266384392ca14d1e6e0f7200  table_pmc.png
64a3a6b5f8feca9c585689a7525dafa22c10de332db72518fd48eb0f4c08d3f2  tilted_memo.png
5580833d8920e9f0e74605b1045bc226944cd137a3e3426ba766782e7aa9e171  scan.png
00279cc6f51d27778190cd00f33bcef8f6595cf49172c5d3745c6d3717763821  scanned-lease.pdf
26c140134a8496f5baba2a629b72444124f9dcbcefd566c704ee0dca14f760f0  mixed-signature.pdf
30930d6f7ce6fade7750f914d094541d3580d2b7cfcf365ffd61e231dadd2198  vendor-invoice.pdf
```

## What comes out

| Path | Script | Content |
|---|---|---|
| `auto/<page>.jsonl` | `indra-ocr --format json` | the CLI in its default mode |
| `ocr/<pdf>.jsonl` | `indra-ocr --mode ocr` | the PDFs read by OCR at 200 DPI |
| `latin-only/<image>.jsonl` | `dump_latin_only.py` | script routing off: every line to `rec.onnx` - what Intern ports |
| `stages/<page>.stages.json`, `.upright.npy` | `dump_stages.py` | skew, quarter turn, the levelled page, detector boxes, script groups, each line's recognizer reading with its tokens, final lines, blocks, markdown |
| `intermediate/<page>/` | `dump_intermediates.py` | the decoded page; each network's inputs and outputs on its first call; `summary.json` with every call's shapes, the orientation probabilities and the beam searches run |
| `lm_vectors.json` | `dump_lm_vectors.py` | beam-search cases (inputs and the chosen ids and steps) and `logp_keys` queries for each language model |
| `onnx_dump.txt` | `dump_onnx.py` | every network's inputs, outputs and operators |

`probe_recognizer.py MODEL_DIR IMAGE` prints the recognizer facts spec §1 and
§6 rest on (time steps per width, padding and batch invariance).

## Where they are read

- `cargo test -p intern-worker --features onnx-ocr,native-pdfium --test indra_ocr`
  with `INTERN_RUNTIME_DIR` holding ONNX Runtime and `indra-models/`: the Intern
  fixtures against the committed goldens; with `INDRA_REFERENCE_DIR` (and
  `INDRA_INPUTS_DIR`, by default `indra-inputs` beside it) Indra's example pages
  too.
- The stage unit tests in `src/indra/` read `INDRA_REFERENCE_DIR` and pass with a
  note when it is unset.
