"""GLiNER2 candidate recall over Intern's fixture corpus (offline tool).

Question: if GLiNER2 proposed candidate parties and dates for the LLM to pick
from, how often would the reviewed answer be among the candidates? This only
scores the model's candidates. It does not touch the LLM or the Rust engine.

Inputs (read from the repo, nothing written to it):
  * fixtures/corpus-recording.json: fixtures[].extraction.source.pages[].text,
    the text Intern's engine sees before distillation. Failed extractions are
    skipped.
  * fixtures/expected.json: parties, document_date, acceptable_dates.

Party matching mirrors `party_matches` in
crates/intern-engine/src/bin/intern-evaluate.rs: lowercase, drop punctuation,
collapse whitespace, then match when either string contains the other
("lenient"). Because that rule accepts a fragment such as "Northstar" for
"Northstar Lantern Works LLC", the script also reports "strict" recall, where a
candidate must equal the reviewed name after the same normalisation.

Dates: each date-like span is parsed with dateutil. A span counts only if it
names a full day (year, month and day), checked by parsing with two different
defaults. The fixture is a hit when the reviewed date or any acceptable date is
among the normalised candidates.

Usage (from its own venv, not a repo dependency):

    python -m venv .venv-gliner2
    .venv-gliner2/Scripts/python -m pip install torch --index-url https://download.pytorch.org/whl/cpu
    .venv-gliner2/Scripts/python -m pip install "gliner2[local]" python-dateutil psutil
    .venv-gliner2/Scripts/python scripts/gliner2-candidate-recall.py \
        --model fastino/gliner2-base-v1 --out gliner2-base.json

Options: --repo (default: the parent of this script's folder), --labels
(primary | alternate), --thresholds (comma list; spans are extracted once at
the lowest one and filtered offline), --chunk-size (words per window).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import statistics
import sys
import time
from datetime import datetime
from pathlib import Path

LABEL_SETS = {
    # Chosen before looking at any result: the three plain types the
    # model-candidates.md protocol names.
    "primary": {
        "party": ["organization", "person"],
        "date": ["date"],
    },
    # Role-flavoured phrasing, reported as a sensitivity check only.
    "alternate": {
        "party": ["company", "person", "contracting party"],
        "date": ["date", "document date", "effective date"],
    },
}


def normalize_party(value: str) -> str:
    kept = "".join(ch for ch in value if ch.isalnum() or ch.isspace()).lower()
    return " ".join(kept.split())


def party_matches(gold: str, actual: str) -> bool:
    g, a = normalize_party(gold), normalize_party(actual)
    return bool(g) and bool(a) and (g in a or a in g)


def party_matches_strict(gold: str, actual: str) -> bool:
    g = normalize_party(gold)
    return bool(g) and g == normalize_party(actual)


def normalize_date(span: str) -> str | None:
    from dateutil import parser

    text = span.strip()
    if not re.search(r"\d", text):
        return None
    try:
        a = parser.parse(text, default=datetime(1901, 1, 1), fuzzy=True)
        b = parser.parse(text, default=datetime(1902, 2, 2), fuzzy=True)
    except (ValueError, OverflowError):
        return None
    if a != b:  # some component came from the default, so not a full day
        return None
    return a.date().isoformat()


_MONTH = r"(?:jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?"
_DATE_PATTERNS = [
    rf"\b{_MONTH}\s+\d{{1,2}}(?:st|nd|rd|th)?,?\s+\d{{4}}\b",
    rf"\b\d{{1,2}}(?:st|nd|rd|th)?\s+(?:day\s+of\s+)?{_MONTH},?\s+\d{{4}}\b",
    r"\b\d{4}-\d{2}-\d{2}\b",
    r"\b\d{1,2}/\d{1,2}/\d{4}\b",
]


def text_dates(text: str) -> set[str]:
    """Every full date written literally in the text (the reachability oracle)."""
    found = set()
    for pattern in _DATE_PATTERNS:
        for m in re.finditer(pattern, text, flags=re.IGNORECASE):
            d = normalize_date(m.group(0))
            if d:
                found.add(d)
    return found


def load_documents(repo: Path):
    recording = json.loads((repo / "fixtures/corpus-recording.json").read_text("utf-8"))
    expected = json.loads((repo / "fixtures/expected.json").read_text("utf-8"))
    truth = {f["file"]: f for f in expected["fixtures"]}
    docs = []
    for fixture in recording["fixtures"]:
        source = (fixture.get("extraction") or {}).get("source")
        gold = truth.get(fixture["file"])
        if not source or gold is None:
            continue
        text = "\n\n".join(p.get("text", "") for p in source["pages"]).strip()
        if not text:
            continue
        docs.append({"file": fixture["file"], "text": text, "gold": gold})
    return docs


def spans_for(model, text, labels, threshold, chunk_size):
    result = model.extract_entities_long(
        text,
        labels,
        threshold=threshold,
        chunk_size=chunk_size,
        chunk_overlap=min(64, chunk_size // 4),
        include_confidence=True,
    )
    out = []
    for label, items in (result.get("entities") or {}).items():
        for item in items or []:
            if isinstance(item, dict):
                out.append((label, item.get("text", ""), float(item.get("confidence", 1.0))))
            else:
                out.append((label, str(item), 1.0))
    return out


def score(docs, raw, label_set, threshold):
    party_labels = set(label_set["party"])
    date_labels = set(label_set["date"])
    rows = []
    for doc in docs:
        spans = [s for s in raw[doc["file"]] if s[2] >= threshold]
        parties = sorted({t.strip() for l, t, _ in spans if l in party_labels and t.strip()},
                         key=str.lower)
        # dedupe by normalised form
        seen, party_cands = set(), []
        for p in parties:
            k = normalize_party(p)
            if k and k not in seen:
                seen.add(k)
                party_cands.append(p)
        date_spans = sorted({t.strip() for l, t, _ in spans if l in date_labels and t.strip()})
        date_cands = sorted({d for d in map(normalize_date, date_spans) if d})
        gold = doc["gold"]
        gold_parties = gold.get("parties") or []
        gold_dates = [d for d in [gold.get("document_date"), *(gold.get("acceptable_dates") or [])] if d]
        row = {
            "file": doc["file"],
            "party_candidates": party_cands,
            "date_spans": date_spans,
            "date_candidates": date_cands,
            "gold_parties": gold_parties,
            "gold_dates": gold_dates,
            "party_hits": [any(party_matches(g, c) for c in party_cands) for g in gold_parties],
            "party_hits_strict": [any(party_matches_strict(g, c) for c in party_cands) for g in gold_parties],
            "date_hit": (any(d in date_cands for d in gold_dates) if gold_dates else None),
            # Reachable = the reviewed value is literally in the text, so a span
            # extractor (or Intern's verbatim check) could return it at all.
            "party_reachable": [normalize_party(g) in normalize_party(doc["text"]) for g in gold_parties],
            "date_reachable": (any(d in text_dates(doc["text"]) for d in gold_dates) if gold_dates else None),
        }
        rows.append(row)
    return rows


def summarize(rows):
    def pct(n, d):
        return None if d == 0 else round(100.0 * n / d, 1)

    hits = [h for r in rows for h in r["party_hits"]]
    strict = [h for r in rows for h in r["party_hits_strict"]]
    with_parties = [r for r in rows if r["gold_parties"]]
    with_dates = [r for r in rows if r["date_hit"] is not None]
    pc = [len(r["party_candidates"]) for r in rows]
    dc = [len(r["date_candidates"]) for r in rows]
    reach_party = [h for r in rows for h, ok in zip(r["party_hits"], r["party_reachable"]) if ok]
    reach_date = [r["date_hit"] for r in with_dates if r["date_reachable"]]
    return {
        "party_recall_reachable": pct(sum(reach_party), len(reach_party)),
        "party_reachable": f"{sum(reach_party)}/{len(reach_party)}",
        "date_recall_reachable": pct(sum(reach_date), len(reach_date)),
        "date_reachable": f"{sum(reach_date)}/{len(reach_date)}",
        "documents": len(rows),
        "party_names": len(hits),
        "party_recall_lenient": pct(sum(hits), len(hits)),
        "party_recall_strict": pct(sum(strict), len(strict)),
        "party_docs_all_found_lenient": f"{sum(all(r['party_hits']) for r in with_parties)}/{len(with_parties)}",
        "party_docs_all_found_strict": f"{sum(all(r['party_hits_strict']) for r in with_parties)}/{len(with_parties)}",
        "date_recall": pct(sum(r["date_hit"] for r in with_dates), len(with_dates)),
        "date_docs": f"{sum(r['date_hit'] for r in with_dates)}/{len(with_dates)}",
        "party_candidates_mean": round(statistics.mean(pc), 1) if pc else 0,
        "party_candidates_max": max(pc) if pc else 0,
        "date_candidates_mean": round(statistics.mean(dc), 1) if dc else 0,
        "date_candidates_max": max(dc) if dc else 0,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--repo", type=Path, default=Path(__file__).resolve().parent.parent)
    ap.add_argument("--model", default="fastino/gliner2-base-v1")
    ap.add_argument("--labels", choices=sorted(LABEL_SETS), default="primary")
    ap.add_argument("--thresholds", default="0.5,0.3,0.1")
    ap.add_argument("--chunk-size", type=int, default=256)
    ap.add_argument("--out", type=Path, default=None)
    args = ap.parse_args()
    # gliner2 prints emoji while loading; a cp1252 Windows console rejects them.
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    import psutil
    import torch
    from gliner2 import GLiNER2

    torch.set_grad_enabled(False)
    thresholds = sorted({float(t) for t in args.thresholds.split(",")}, reverse=True)
    label_set = LABEL_SETS[args.labels]
    labels = list(dict.fromkeys(label_set["party"] + label_set["date"]))
    proc = psutil.Process(os.getpid())

    docs = load_documents(args.repo)
    print(f"{len(docs)} documents with text and labels; model {args.model}; labels {labels}", flush=True)
    t0 = time.perf_counter()
    model = GLiNER2.from_pretrained(args.model)
    load_s = time.perf_counter() - t0
    print(f"loaded in {load_s:.1f}s", flush=True)

    # Warm-up so the first document's latency is not the allocator's.
    spans_for(model, "Agreement dated March 3, 2025 between Acme LLC and Jane Roe.", labels, 0.5, args.chunk_size)

    raw, latency = {}, {}
    floor = min(thresholds)
    for doc in docs:
        t = time.perf_counter()
        raw[doc["file"]] = spans_for(model, doc["text"], labels, floor, args.chunk_size)
        latency[doc["file"]] = round(time.perf_counter() - t, 3)
        print(f"  {doc['file']}: {len(doc['text'])} chars, {len(raw[doc['file']])} spans, {latency[doc['file']]}s", flush=True)

    mem = proc.memory_info()
    peak = getattr(mem, "peak_wset", None) or getattr(mem, "rss", 0)

    report = {
        "model": args.model,
        "labels": args.labels,
        "label_list": labels,
        "chunk_size": args.chunk_size,
        "versions": {
            "python": sys.version.split()[0],
            "torch": torch.__version__,
            "gliner2": __import__("importlib.metadata").metadata.version("gliner2"),
        },
        "load_seconds": round(load_s, 1),
        "latency_seconds": {
            "median": round(statistics.median(latency.values()), 3),
            "max": max(latency.values()),
            "per_document": latency,
        },
        "peak_rss_mb": round(peak / 2**20),
        "by_threshold": {},
    }
    for th in thresholds:
        rows = score(docs, raw, label_set, th)
        report["by_threshold"][str(th)] = {"summary": summarize(rows), "rows": rows}
        s = report["by_threshold"][str(th)]["summary"]
        print(f"\nthreshold {th}: {json.dumps(s)}")
        for r in rows:
            for g, hit, strict in zip(r["gold_parties"], r["party_hits"], r["party_hits_strict"]):
                if not strict:
                    print(f"  party {'fragment-only' if hit else 'MISS'}: {r['file']}: {g!r} not in {r['party_candidates']}")
            if r["date_hit"] is False:
                print(f"  date MISS: {r['file']}: {r['gold_dates']} not in {r['date_candidates']} (spans {r['date_spans']})")
    print(f"\nlatency median {report['latency_seconds']['median']}s max {report['latency_seconds']['max']}s; peak RSS {report['peak_rss_mb']} MB")
    if args.out:
        args.out.write_text(json.dumps(report, indent=2), "utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
