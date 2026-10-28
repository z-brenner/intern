#!/usr/bin/env python3
"""Asks LiquidAI d1 (a one-pass decision model) about each name a replay
proposed, to see whether its calibrated yes/no and choice answers can send a
wrong name to review before it is marked ready.

Laboratory tooling; the app does not call it. Standard library only.

    d1_verifier.py ask --server http://127.0.0.1:18091 --recording bench/recording.json \
        --report REPLAY_REPORT.json --out answers.jsonl [--budget 8000]
    d1_verifier.py analyze --answers answers.jsonl --report REPLAY_REPORT.json [--markdown out.md]

`ask` sends one /v1/systemone request per document that has a proposed party
(llama.cpp answers noul, choice and score questions; extraction is not
supported there, so none is asked). The state is the document's extracted text
as recorded: the first and last parts when it exceeds --budget characters, so
the letterhead and the signature blocks both stay in. It resumes: a document
already in --out is skipped.

`analyze` treats each signal as a review gate over the documents the pipeline
marked ready, and reports, per threshold, how many wrong names it would hold
(unsafe-ready caught) against how many right names it would also hold. The
threshold is also chosen by leave-one-out over the ready documents, so the
headline number is not fitted to the documents it is scored on.
"""
import argparse
import json
import os
import sys
import time
import urllib.request

RELATIONS = {
    "issuer": "issued, sent, wrote or prepared the document",
    "recipient": "the document is addressed or sent to them",
    "subject": "the document is for, about, or belongs to them (a client, owner, policyholder or account)",
    "counterparty": "they are one of the parties to an agreement",
    "signatory": "they only signed, chaired, inspected or prepared it on behalf of another party",
    "unrelated": "they are not involved in the document",
}
# How Intern's relation word maps onto the choice above.
EXPECTED = {
    "from": {"issuer"},
    "to": {"recipient"},
    "for": {"subject", "recipient"},
    "between": {"counterparty"},
    "with": {"counterparty"},
    "-": {"subject", "issuer", "recipient", "counterparty"},
}


def state_of(document, budget):
    pages = (document.get("extraction") or {}).get("source", {}).get("pages") or []
    text = "\n\n".join(page.get("text") or "" for page in pages).strip()
    if len(text) <= budget:
        return text
    head = int(budget * 0.75)
    tail = budget - head
    return text[:head] + "\n\n[...]\n\n" + text[-tail:]


def questions_for(proposal, filename):
    party = proposal["parties"][0]
    others = proposal["parties"][1:]
    stem = os.path.splitext(filename)[0]
    q = {
        "party_issued": {"type": "noul", "instructions": f"Did {party} issue, send, write or prepare this document?"},
        "party_recipient": {"type": "noul", "instructions": f"Is this document addressed or sent to {party}?"},
        "party_subject": {"type": "noul", "instructions": f"Is this document for, about, or kept on behalf of {party}?"},
        "party_signatory_only": {
            "type": "noul",
            "instructions": f"Is {party} only an individual who signed, chaired, inspected or prepared this document "
            f"for an organization, rather than the organization or person the document belongs to?",
        },
        "party_main": {"type": "noul", "instructions": f"Is {party} the main organization or person this document belongs to or comes from?"},
        "party_relation": {
            "type": "choice",
            "instructions": f"How is {party} related to this document?",
            "criteria": dict(RELATIONS),
        },
        "filename_accurate": {
            "type": "noul",
            "instructions": f"Is \"{stem}\" an accurate filename for this document, with the right date, the right "
            f"kind of document, and the right organization or person in the right relation?",
        },
    }
    if others:
        q["second_party_involved"] = {"type": "noul", "instructions": f"Is {others[0]} a party to this document?"}
    return q


def post(server, body, timeout):
    request = urllib.request.Request(
        server.rstrip("/") + "/v1/systemone",
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with opener.open(request, timeout=timeout) as response:
        return json.load(response)


def ask(args):
    recording = {d["id"]: d for d in json.load(open(args.recording))["documents"]}
    report = json.load(open(args.report))["records"]
    done = set()
    if os.path.exists(args.out):
        for line in open(args.out):
            if line.strip():
                done.add(json.loads(line)["id"])
    with open(args.out, "a") as out:
        for record in report:
            proposal = record.get("proposal") or {}
            if record["id"] in done or not proposal.get("parties"):
                continue
            if args.only_ready and not record["scores"].get("ready"):
                continue
            state = state_of(recording[record["id"]], args.budget)
            questions = questions_for(proposal, record["filename"])
            started = time.monotonic()
            try:
                reply = post(args.server, {"state": state, "questions": questions}, args.timeout)
                error = None
            except Exception as exc:  # recorded, not fatal: one bad document must not stop the run
                reply, error = None, repr(exc)
            row = {
                "id": record["id"],
                "seconds": round(time.monotonic() - started, 3),
                "state_characters": len(state),
                "questions": questions,
                "reply": reply,
                "error": error,
            }
            out.write(json.dumps(row) + "\n")
            out.flush()
            print(f"{record['id']}: {row['seconds']} s{' ERROR ' + error if error else ''}", flush=True)


def signals(row, proposal):
    """Each signal: higher means more likely wrong."""
    answers = (row.get("reply") or {}).get("answers") or {}
    out = {}
    def p_yes(name):
        a = answers.get(name)
        return None if a is None else a.get("noul")
    for name, flip in (
        ("party_main", True),
        ("filename_accurate", True),
        ("party_signatory_only", False),
    ):
        v = p_yes(name)
        if v is not None:
            out[name] = 1 - v if flip else v
    relation = answers.get("party_relation")
    word = (proposal.get("party_relation") or "-").strip() or "-"
    if relation and relation.get("probabilities") and word in EXPECTED:
        probabilities = relation["probabilities"]
        out["relation_mismatch"] = 1 - sum(probabilities.get(k, 0.0) for k in EXPECTED[word])
    # The yes/no question that matches the relation word, when there is one.
    direct = {"from": "party_issued", "to": "party_recipient", "for": "party_subject"}.get(word)
    if direct and p_yes(direct) is not None:
        out["relation_direct_no"] = 1 - p_yes(direct)
    parts = [out[k] for k in ("party_main", "party_signatory_only", "relation_mismatch") if k in out]
    if parts:
        out["max_party_relation"] = max(parts)
    return out


def sweep(items):
    """items: (score, wrong). Returns rows for every distinct threshold."""
    rows = []
    for t in sorted({s for s, _ in items}):
        caught = sum(1 for s, w in items if s >= t and w)
        held = sum(1 for s, w in items if s >= t and not w)
        rows.append((t, caught, held))
    return rows


def best_threshold(items, cost):
    """Threshold that maximises caught - cost * held (cost = right names one wrong name is worth)."""
    best, best_value = float("inf"), 0.0
    for t, caught, held in sweep(items):
        value = caught - cost * held
        if value > best_value:
            best, best_value = t, value
    return best


def analyze(args):
    report = {r["id"]: r for r in json.load(open(args.report))["records"]}
    rows = [json.loads(line) for line in open(args.answers) if line.strip()]
    ready = []
    for row in rows:
        record = report[row["id"]]
        scores = record["scores"]
        if not scores.get("ready"):
            continue
        ready.append((row, record, bool(scores.get("unsafe_ready"))))
    n_wrong = sum(1 for _, _, w in ready if w)
    n_right = len(ready) - n_wrong
    errors = sum(1 for row in rows if row.get("error"))
    seconds = sorted(row["seconds"] for row in rows if not row.get("error"))
    lines = [
        f"# d1 as a review gate over ready names",
        "",
        f"Documents asked: {len(rows)} ({errors} errors). Ready documents with a party: {len(ready)} "
        f"({n_wrong} wrong names, {n_right} right).",
    ]
    if seconds:
        lines.append(f"d1 time per document: median {seconds[len(seconds)//2]:.1f} s, max {seconds[-1]:.1f} s.")
    names = sorted({k for row, record, _ in ready for k in signals(row, record["proposal"])})
    for cost in args.costs:
        lines += ["", f"## Leave-one-out threshold, worth holding {1 / cost:g} right name(s) to hold one wrong one", "",
                  "| Signal | Wrong names held | Right names held |", "| --- | ---: | ---: |"]
        for name in names:
            items = [(signals(row, record["proposal"]).get(name), w) for row, record, w in ready]
            items = [(s, w) for s, w in items if s is not None]
            caught = held = 0
            for i, (s, w) in enumerate(items):
                t = best_threshold(items[:i] + items[i + 1:], cost)
                if s >= t:
                    caught += w
                    held += not w
            lines.append(f"| {name} | {caught}/{n_wrong} | {held}/{n_right} |")
    lines += ["", "## Every threshold (fitted on all ready documents; optimistic)", ""]
    for name in names:
        items = [(signals(row, record["proposal"]).get(name), w) for row, record, w in ready]
        items = [(s, w) for s, w in items if s is not None]
        lines.append(f"### {name}")
        lines.append("")
        lines.append("| Threshold | Wrong held | Right held |")
        lines.append("| ---: | ---: | ---: |")
        for t, caught, held in sweep(items):
            if held <= max(10, n_right // 4):
                lines.append(f"| {t:.3f} | {caught} | {held} |")
        lines.append("")
    lines += ["## Per document", "", "| Document | Wrong | Name | " + " | ".join(names) + " |",
              "| --- | --- | --- | " + " | ".join("---:" for _ in names) + " |"]
    for row, record, w in sorted(ready, key=lambda x: (not x[2], x[1]["id"])):
        s = signals(row, record["proposal"])
        lines.append(f"| {record['id']} | {'yes' if w else ''} | {record['filename']} | "
                     + " | ".join(f"{s[n]:.2f}" if n in s else "" for n in names) + " |")
    text = "\n".join(lines) + "\n"
    if args.markdown:
        open(args.markdown, "w").write(text)
    print(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    a = sub.add_parser("ask")
    a.add_argument("--server", required=True)
    a.add_argument("--recording", required=True)
    a.add_argument("--report", required=True)
    a.add_argument("--out", required=True)
    a.add_argument("--budget", type=int, default=8000)
    a.add_argument("--timeout", type=float, default=600)
    a.add_argument("--only-ready", action="store_true",
                   help="ask only about names the pipeline marked ready (all the gate is scored on); "
                   "llama.cpp re-reads the state for every question, so each document costs minutes on a CPU")
    z = sub.add_parser("analyze")
    z.add_argument("--answers", required=True)
    z.add_argument("--report", required=True)
    z.add_argument("--markdown")
    z.add_argument("--costs", type=float, nargs="+", default=[0.5, 1.0])
    args = parser.parse_args()
    ask(args) if args.command == "ask" else analyze(args)


if __name__ == "__main__":
    sys.exit(main())
