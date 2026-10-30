"""Builds a pinned CUAD sample (sample.json and its held-out siblings) from CUAD's own files.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/cuad-eval/make-sample.py MASTER_CLAUSES.csv TREE_DIR OFFSET OUT.json PDF_DIR

MASTER_CLAUSES.csv is CUAD v1's master_clauses.csv. TREE_DIR holds the Hugging Face tree listings
of CUAD_v1/full_contract_pdf/Part_I, Part_II and Part_III at the pinned revision, saved as
tree-Part_I.json and so on, from

    https://huggingface.co/api/datasets/theatticusproject/cuad/tree/REVISION/CUAD_v1/full_contract_pdf/PART?recursive=true

The sample is every 12th contract of the 510, sorted by file name, starting at OFFSET: 0 gave
sample.json, which #45's CUAD rules were written from; 6 gave sample-heldout.json, which #47's
were written from; 3 gave sample-third.json, which none of them were. A contract without a PDF
is left out. Each PDF is downloaded into PDF_DIR and pinned by its size and SHA-256 (the dataset
stores them as plain Git objects, so the tree gives no SHA-256).
"""
import csv
import hashlib
import json
import os
import sys
import urllib.parse
import urllib.request

REVISION = "a3c393f5d103fd0c516374e4fdff676c8176dcb1"
STEP = 12
BASE = "https://huggingface.co/datasets/theatticusproject/cuad/resolve"


def main():
    if len(sys.argv) != 6:
        sys.exit("usage: make-sample.py MASTER_CLAUSES.csv TREE_DIR OFFSET OUT.json PDF_DIR")
    master, tree_dir, offset, out, pdf_dir = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4], sys.argv[5]
    os.makedirs(pdf_dir, exist_ok=True)
    pdfs = {}
    for part in ("Part_I", "Part_II", "Part_III"):
        for entry in json.load(open(os.path.join(tree_dir, f"tree-{part}.json"), encoding="utf-8")):
            if entry["type"] == "file" and entry["path"].lower().endswith(".pdf"):
                pdfs.setdefault(os.path.basename(entry["path"]).lower(), entry)
    rows = sorted(csv.DictReader(open(master, encoding="utf-8")), key=lambda row: row["Filename"])
    contracts = []
    for step, index in enumerate(range(offset, len(rows), STEP)):
        row = rows[index]
        entry = pdfs.get(row["Filename"].lower())
        if entry is None:
            continue
        # Numbered by place in the sample, so a contract left out leaves its number unused.
        number = f"cuad-{step:02d}"
        path = os.path.join(pdf_dir, f"{number}.pdf")
        url = f"{BASE}/{REVISION}/{urllib.parse.quote(entry['path'])}"
        with urllib.request.urlopen(url, timeout=120) as response, open(path, "wb") as file:
            file.write(response.read())
        data = open(path, "rb").read()
        if len(data) != entry["size"]:
            sys.exit(f"{entry['path']}: {len(data)} bytes, the tree says {entry['size']}")
        contracts.append({
            "id": number,
            "path": entry["path"],
            "size": entry["size"],
            "sha256": hashlib.sha256(data).hexdigest(),
            "document_name": row["Document Name-Answer"],
            "parties": row["Parties-Answer"],
            "agreement_date": row["Agreement Date-Answer"],
            "effective_date": row["Effective Date-Answer"],
        })
    sample = {
        "source": "CUAD v1 (The Atticus Project), https://huggingface.co/datasets/theatticusproject/cuad",
        "license": "CC BY 4.0",
        "revision": REVISION,
        "selection": f"Every {STEP}th of the {len(rows)} contracts in master_clauses.csv, sorted by file name, "
                     f"starting at the {ordinal(offset + 1)}; a contract without a PDF is left out.",
        "answers": "Document Name, Parties, Agreement Date and Effective Date as CUAD gives them in "
                   "master_clauses.csv, verbatim.",
        "contracts": contracts,
    }
    with open(out, "w", encoding="utf-8") as file:
        json.dump(sample, file, indent=1, ensure_ascii=False)
        file.write("\n")
    print(f"{len(contracts)} contracts in {out}")


def ordinal(n):
    suffix = "th" if 10 <= n % 100 <= 20 else {1: "st", 2: "nd", 3: "rd"}.get(n % 10, "th")
    return f"{n}{suffix}"


if __name__ == "__main__":
    main()
