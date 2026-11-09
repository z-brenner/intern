"""Keeps, of a recording of a real-document set, only the documents bench/realdocs/SOURCES.md clears.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/realdocs-eval/commit-recording.py SET RECORDING.json OUT.json

SET is real1, real2, real3 or real4. RECORDING.json is a live recording of that set (the candidate
benchmark workflow's recording.json, or scripts/run-internbench.sh's); OUT.json gets its header and
the lines of the documents SOURCES.md lists as cleared for SET, byte for byte as recorded, so a
review of the commit sees which documents a re-recording changed. Nothing else of a recording may be
committed: the other documents' licences do not allow it (bench/realdocs/SOURCES.md says why, per
document). The script says which cleared documents the recording lacks, and which it holds that
SOURCES.md still marks as not recorded: mark those `yes` in their row, or scripts/realdocs-gate.test.ts
fails.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SOURCES = os.path.join(ROOT, "bench", "realdocs", "SOURCES.md")
OPENING = '  "documents": ['
CLOSING = "\n  ]\n}\n"


def cleared(set_name):
    """The ids SOURCES.md clears for set_name, each with whether it says the recording holds it."""
    rows, current = {}, None
    for line in open(SOURCES, encoding="utf-8"):
        heading = re.match(r"## (real\d+): ", line)
        if heading:
            current = heading.group(1)
        elif current == set_name:
            row = re.match(r"\| `([^`]+)` \| (yes|no)\b", line)
            if row:
                rows[row.group(1)] = row.group(2) == "yes"
    if not rows:
        sys.exit(f"{SOURCES} clears no document for {set_name}")
    return rows


def main():
    if len(sys.argv) != 4:
        sys.exit("usage: commit-recording.py SET RECORDING.json OUT.json")
    set_name, source, target = sys.argv[1:]
    allowed = cleared(set_name)
    text = open(source, encoding="utf-8").read()
    header_end = text.index(OPENING) + len(OPENING)
    header, body = text[:header_end], text[header_end:]
    if not body.startswith("\n    ") or not body.endswith(CLOSING):
        sys.exit(f"{source} is not laid out as intern-bench writes a recording (one document per line)")
    lines = body[len("\n    ") : -len(CLOSING)].split(",\n    ")
    # One JSON object per line: a split inside a string would not parse.
    documents = [json.loads(line) for line in lines]
    kept = [line for line, document in zip(lines, documents) if document["id"] in allowed]
    rendered = header + "\n    " + ",\n    ".join(kept) + CLOSING if kept else header + "]\n}\n"
    original = json.loads(text)
    expected = dict(original, documents=[document for document in original["documents"] if document["id"] in allowed])
    if json.loads(rendered) != expected:
        sys.exit("the filtered recording does not parse back to the recording minus the excluded documents")
    with open(target, "w", encoding="utf-8") as file:
        file.write(rendered)
    held = {document["id"] for document in documents}
    print(f"{set_name}: kept {len(kept)} of {len(documents)} documents in {target}")
    lacking = sorted(id for id in allowed if id not in held)
    if lacking:
        print(f"cleared but not in the recording: {', '.join(lacking)}")
    newly = sorted(id for id, recorded in allowed.items() if id in held and not recorded)
    if newly:
        print(f"now recorded, still marked 'no' in {SOURCES}: {', '.join(newly)}")


if __name__ == "__main__":
    main()
