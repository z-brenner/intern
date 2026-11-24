#!/usr/bin/env python3
"""Turns an ATLAS teacher job log back into answers files. Laboratory tooling.

    unpack_log.py JOB_LOG OUT_DIR

Writes OUT_DIR/answers-<set>.json.gz for every complete ATLAS-ANSWERS block whose checksum
matches, and OUT_DIR/model.json from the ATLAS-MODEL line. A block that is cut off or fails its
checksum is reported and skipped, never guessed at.
"""
import base64
import gzip
import hashlib
import json
import os
import re
import sys


def main():
    text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    out = sys.argv[2]
    os.makedirs(out, exist_ok=True)
    found = 0
    for match in re.finditer(r"=====ATLAS-ANSWERS-BEGIN ([a-z0-9._-]+)=====\s*(.*?)\s*"
                             r"=====ATLAS-ANSWERS-END \1 sha256=([0-9a-f]{64})=====", text, re.S):
        name, body, digest = match.group(1), re.sub(r"\s+", "", match.group(2)), match.group(3)
        if hashlib.sha256(body.encode()).hexdigest() != digest:
            print(f"{name}: checksum mismatch, skipped", file=sys.stderr)
            continue
        answers = json.loads(gzip.decompress(base64.b64decode(body)))
        with gzip.open(os.path.join(out, f"answers-{name}.json.gz"), "wt", encoding="utf-8") as file:
            json.dump(answers, file)
        print(f"{name}: {len(answers)} answers")
        found += 1
    model = re.search(r"=====ATLAS-MODEL (\{.*?\})=====", text)
    if model:
        json.dump(json.loads(model.group(1)), open(os.path.join(out, "model.json"), "w"), indent=1)
    sys.exit(0 if found else 1)


if __name__ == "__main__":
    main()
