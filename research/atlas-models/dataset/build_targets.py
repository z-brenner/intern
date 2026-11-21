#!/usr/bin/env python3
"""Builds the model reply each generated document should get, for the verify loop. Laboratory tooling.

    python3 -I build_targets.py answers --gold G --capture C --variant N --passed P.json --out A.json
    python3 -I build_targets.py record  --gold G --capture C --report R.json --answers A.json --passed P.json
    python3 -I build_targets.py export  --gold G --capture C --passed P.json --out sft.jsonl

The reply is built only from the document's gold and the exact evidence lines the engine sent
(captured by fake_server.py): each fact cites the handle of the line it is printed on, so it
passes the engine's own validation. Party roles are tried in a few orders (variant 0, 1, 2)
because the engine's composition, not this script, decides "to", "from", "for" or "between";
a document is kept only when the bench scores the name the engine composes from the reply as
right. `record` reads that verdict from intern-bench's report.json; `export` writes the kept
documents as chat-format SFT examples: the system and user turns exactly as sent, the grammar,
and the reply.
"""
import argparse
import json
import re
import sys

ROLE_WORDS = ["client", "contractor", "employer", "employee", "buyer", "seller", "landlord", "tenant", "issuer",
              "recipient", "vendor", "customer", "borrower", "lender", "licensor", "licensee", "sender", "addressee",
              "other"]


def load_jsonl(path):
    with open(path, encoding="utf-8") as file:
        return [json.loads(line) for line in file if line.strip()]


def messages_of(body):
    return body.get("messages", [])


def user_turn(body):
    for message in reversed(messages_of(body)):
        if message.get("role") == "user":
            return message.get("content", "")
    return ""


def evidence(prompt):
    begin = prompt.find("--- BEGIN EVIDENCE ---")
    end = prompt.find("--- END EVIDENCE ---")
    lines = []
    for line in prompt[begin:end].splitlines():
        match = re.match(r"^\[([^\]]+)\] (.*)$", line)
        if match:
            lines.append((match.group(1), match.group(2)))
    return lines


def norm(text):
    return re.sub(r"\s+", " ", text).strip().lower()


def find_line(lines, needle, prefer_whole=False):
    """The handle of the line that prints `needle`, preferring a line that is little else."""
    hits = [(handle, text) for handle, text in lines if norm(needle) in norm(text)]
    if not hits:
        return None
    if prefer_whole:
        hits.sort(key=lambda hit: len(hit[1]) - len(needle))
    return hits[0][0]


def printed(lines, needle):
    """`needle` as the line prints it (its own case), from the first line that has it."""
    for _, text in lines:
        at = text.lower().find(needle.lower())
        if at >= 0:
            return text[at:at + len(needle)]
    return needle


def match_documents(gold, captures):
    """Each captured request's document: the one whose date text and parties its evidence prints."""
    by_key = {}
    for row in captures:
        prompt = user_turn(row["body"])
        if row["key"] in by_key:
            continue
        text = norm(prompt)
        found = []
        for document in gold["documents"]:
            g = document["gold"]
            dates = g["evidence"]["date_text"]
            names = g["parties"]
            if all(norm(name) in text for name in names) and any(norm(date) in text for date in dates):
                found.append(document["id"])
        if len(found) == 1:
            by_key[row["key"]] = (found[0], row["body"])
    return by_key


def roles_of(g):
    roles = {}
    for entry in g["party_roles"]:
        roles.setdefault(entry["name"], []).append(entry["role"])
    return roles


def party_variants(g):
    """Ordered lists of (name, role) to try, most likely first."""
    roles = roles_of(g)
    relation = g["party_relation"]
    gold_parties = g["parties"]
    other = [name for name in roles if name not in gold_parties]
    if relation == "between":
        a, b = gold_parties
        return [[(a, "other"), (b, "other")], [(a, "client"), (b, "contractor")]]
    party = gold_parties[0]
    counterpart = other[0] if other else None
    if relation == "to":
        sender_role = "sender" if "sender" in roles.get(counterpart, []) else "issuer"
        variants = [[(party, "recipient")] + ([(counterpart, sender_role)] if counterpart else []),
                    [(party, "addressee")] + ([(counterpart, "sender")] if counterpart else []),
                    [(party, "recipient")]]
    elif relation == "from":
        own = "vendor" if "vendor" in roles.get(party, []) else "issuer"
        variants = [([(counterpart, "customer")] if counterpart else []) + [(party, own)],
                    [(party, own)] + ([(counterpart, "customer")] if counterpart else []),
                    [(party, "issuer")]]
    elif relation == "for":
        variants = [[(party, "other")],
                    [(party, "recipient")],
                    [(party, "other")] + ([(counterpart, "issuer")] if counterpart else [])]
    else:
        variants = [[(name, "other") for name in gold_parties]]
    return variants


def build_reply(document, prompt, variant):
    g = document["gold"]
    lines = evidence(prompt)
    type_handle = find_line(lines, g["document_type"], prefer_whole=True)
    if not type_handle:
        return None
    type_text = printed([(h, t) for h, t in lines if h == type_handle], g["document_type"])
    if variant >= 6:
        type_text = g["document_type"]
    date_handle = None
    for date_text in g["evidence"]["date_text"]:
        date_handle = find_line(lines, date_text, prefer_whole=True)
        if date_handle:
            break
    if not date_handle:
        return None
    variants = party_variants(g)
    chosen = variants[variant % 3] if variant % 3 < len(variants) else None
    if chosen is None:
        return None
    parties = []
    for name, role in chosen:
        handle = find_line(lines, name, prefer_whole=True)
        if not handle:
            return None
        parties.append([printed(lines, name), role, handle])
    reply = {"type": [type_text, type_handle], "date": [g["document_date"], g["date_role"], date_handle], "parties": parties}
    subject = next(((h, t[4:]) for h, t in lines if t.startswith("Re: ")), None)
    if subject:
        words = subject[1].split()[:8]
        text = " ".join(words)[:60].rstrip(" ,;:")
        reply["subject"] = [text, subject[0]]
    # The engine's compact form: no spaces, keys in the grammar's order.
    return json.dumps(reply, ensure_ascii=False, separators=(",", ":"))


def primary(g, filename):
    """Whether a composed name joins the gold's own parties with its own word."""
    def key(text):
        return re.sub(r"[^a-z0-9& ]", "", text.lower())
    parties = g["parties"]
    relation = g["party_relation"]
    if relation == "between":
        return key(f"between {parties[0]} and {parties[1]}") in key(filename)
    if relation == "none":
        return True
    return key(f"{relation} {parties[0]}") in key(filename)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["answers", "record", "export"])
    parser.add_argument("--gold", required=True)
    parser.add_argument("--capture", required=True)
    parser.add_argument("--variant", type=int, default=0)
    parser.add_argument("--passed", required=True)
    parser.add_argument("--report")
    parser.add_argument("--answers")
    parser.add_argument("--out")
    parser.add_argument("--strict", action="store_true",
                        help="keep only names joined to the gold's own party set, not an acceptable alternative")
    args = parser.parse_args()
    gold = json.load(open(args.gold, encoding="utf-8"))
    documents = {document["id"]: document for document in gold["documents"]}
    by_key = match_documents(gold, load_jsonl(args.capture))
    try:
        passed = json.load(open(args.passed, encoding="utf-8"))
    except FileNotFoundError:
        passed = {}

    if args.command == "answers":
        answers = {}
        meta = {}
        for key, (document_id, body) in by_key.items():
            if document_id in passed:
                answers[key] = passed[document_id]["reply"]
                continue
            reply = build_reply(documents[document_id], user_turn(body), args.variant)
            if reply:
                answers[key] = reply
                meta[document_id] = reply
        json.dump(answers, open(args.out, "w", encoding="utf-8"))
        json.dump(meta, open(args.out + ".tried.json", "w", encoding="utf-8"))
        print(f"variant {args.variant}: {len(meta)} new replies, {len(passed)} already kept, "
              f"{len(documents) - len(by_key)} documents with no matched request", file=sys.stderr)
    elif args.command == "record":
        tried = json.load(open(args.answers + ".tried.json", encoding="utf-8"))
        report = json.load(open(args.report, encoding="utf-8"))
        added = 0
        for record in report.get("records", []):
            document_id = record.get("id")
            scores = record.get("scores") or {}
            if document_id in tried and document_id not in passed and scores.get("filename_correct") \
                    and (not args.strict or primary(documents[document_id]["gold"], record.get("filename") or "")):
                passed[document_id] = {"reply": tried[document_id], "filename": record.get("filename"),
                                       "ready": bool(scores.get("ready"))}
                added += 1
        json.dump(passed, open(args.passed, "w", encoding="utf-8"), indent=1)
        print(f"kept {added} more; {len(passed)} of {len(documents)} verified", file=sys.stderr)
    else:
        with open(args.out, "w", encoding="utf-8") as out:
            for key, (document_id, body) in by_key.items():
                if document_id not in passed:
                    continue
                document = documents[document_id]
                example = {
                    "id": document_id,
                    "family": document.get("family"),
                    "format": document.get("format"),
                    "messages": [{"role": m["role"], "content": m["content"]} for m in messages_of(body)]
                    + [{"role": "assistant", "content": passed[document_id]["reply"]}],
                    "grammar": body.get("grammar"),
                    "filename": passed[document_id]["filename"],
                }
                out.write(json.dumps(example, ensure_ascii=False) + "\n")
        print(f"exported {len(passed)} examples to {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
