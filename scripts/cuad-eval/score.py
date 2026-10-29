"""Scores an InternBench report.json of the CUAD sample against CUAD's own answers.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/cuad-eval/score.py REPORT.json [--all]

prints the totals, then every document that is not right (all of them with --all).

- Type: exact when Intern's words are CUAD's Document Name's; lenient when every word of
  Intern's (two or more) is in it and both end on the same noun.
- Date: CUAD's agreement or effective date. An answer with a blank ("[]/[]/2000") or none means
  the contract states no full date of its own, and then giving none is right.
- Parties: every party Intern names is a different one of CUAD's, defined terms and legal
  suffixes aside, and an agreement names two when CUAD lists two or more.
- Right: lenient type, date and parties together. Ready: filed without review.

CUAD's answers are spans of the filings as annotated, so they carry the filings' typos ("Bank of
America Corporatino") and partial dates ("09/14/200[]"); a reading the scorer marks wrong is
worth a look before it counts as Intern's error.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

MONTHS = {
    month: number + 1
    for number, month in enumerate(
        ["january", "february", "march", "april", "may", "june", "july", "august",
         "september", "october", "november", "december"]
    )
}
MONTHS.update({month[:3]: number for month, number in list(MONTHS.items())})
MONTHS["sept"] = 9
SUFFIXES = {"inc", "incorporated", "llc", "ltd", "limited", "corp", "corporation", "company",
            "co", "lp", "llp", "plc", "sa", "ag", "gmbh", "bv", "nv", "pty", "the", "lc", "pc"}


def words(text):
    return re.findall(r"[a-z0-9]+", (text or "").lower().replace("&", " and "))


def parse_date(text):
    """The ISO date of the one date written in a CUAD answer; None for a blank or partial one."""
    text = (text or "").strip()
    if not text or "[" in text or "_" in text:
        return None
    found = re.search(r"\b(\d{1,2})/(\d{1,2})/(\d{2,4})\b", text)
    if found:
        month, day, year = map(int, found.groups())
        if year < 100:
            year += 2000 if year < 50 else 1900
        return f"{year:04d}-{month:02d}-{day:02d}"
    found = re.search(r"\b([A-Za-z]+)\.?\s+(\d{1,2})(?:st|nd|rd|th)?,?\s+(\d{4})", text)
    if found and found.group(1).lower() in MONTHS:
        return f"{int(found.group(3)):04d}-{MONTHS[found.group(1).lower()]:02d}-{int(found.group(2)):02d}"
    found = re.search(r"\b(\d{1,2})(?:st|nd|rd|th)?\s+(?:day\s+of\s+)?([A-Za-z]+),?\s+(\d{4})", text)
    if found and found.group(2).lower() in MONTHS:
        return f"{int(found.group(3)):04d}-{MONTHS[found.group(2).lower()]:02d}-{int(found.group(1)):02d}"
    return None


def type_scores(intern, cuad):
    a, b = words(intern), words(cuad)
    a, b = a[1:] if a[:1] == ["the"] else a, b[1:] if b[:1] == ["the"] else b
    exact = a == b
    return exact, exact or (len(a) >= 2 and a[-1:] == b[-1:] and all(word in b for word in a))


def cuad_parties(text):
    parties = []
    for part in (text or "").split(";"):
        name = re.sub(r"[\"“”']", " ", re.sub(r"\([^)]*\)?", " ", part))
        core = [word for word in words(name) if word not in SUFFIXES]
        if core:
            parties.append(core)
    return parties


def names(name, party):
    core = [word for word in words(name) if word not in SUFFIXES]
    a, b = " ".join(core), " ".join(party)
    return bool(core) and (a == b or (len(a) >= 4 and (f" {a} " in f" {b} " or f" {b} " in f" {a} ")))


def main():
    if len(sys.argv) < 2:
        sys.exit("usage: score.py REPORT.json [--all]")
    contracts = {c["id"]: c for c in json.load(open(os.path.join(HERE, "sample.json"), encoding="utf-8"))["contracts"]}
    rows = []
    for record in json.load(open(sys.argv[1], encoding="utf-8"))["records"]:
        contract = contracts[record["id"]]
        proposal = record.get("proposal") or {}
        exact, lenient = type_scores(proposal.get("document_type"), contract["document_name"])
        dates = {d for d in (parse_date(contract["agreement_date"]), parse_date(contract["effective_date"])) if d}
        date = proposal.get("document_date")
        date_right = date in dates if dates else date is None
        parties = cuad_parties(contract["parties"])
        named = proposal.get("parties") or []
        used, each = set(), True
        for name in named:
            hit = next((k for k, party in enumerate(parties) if k not in used and names(name, party)), None)
            each = each and hit is not None
            used.add(hit)
        parties_right = bool(named) and each and len(named) >= min(2, len(parties))
        rows.append({
            "id": record["id"], "ready": record.get("readiness") == "ready", "exact": exact,
            "type": lenient, "date": date_right, "parties": parties_right,
            "right": lenient and date_right and parties_right,
            "intern": (proposal.get("document_type"), date, named),
            "cuad": (contract["document_name"], contract["agreement_date"], contract["parties"][:100]),
            "reasons": record.get("review_reasons"), "error": record.get("error"),
        })
    count = lambda key: sum(1 for row in rows if row[key])
    print(f"contracts {len(rows)}: right {count('right')}, ready {count('ready')}, "
          f"wrong but ready {sum(1 for row in rows if row['ready'] and not row['right'])}")
    print(f"type exact {count('exact')}, type {count('type')}, date {count('date')}, parties {count('parties')}")
    for row in rows:
        if "--all" in sys.argv or not row["right"]:
            flags = "".join(c for c, key in (("T", "type"), ("D", "date"), ("P", "parties")) if not row[key])
            print(f"{row['id']} {'R' if row['ready'] else '-'} {flags or 'ok':3} intern={row['intern']}\n"
                  f"          cuad={row['cuad']} reasons={row['reasons']}{' error=' + row['error'] if row['error'] else ''}")


if __name__ == "__main__":
    main()
