"""Map a recorded token trace of a compact reply onto its fields.

fields(trace) -> dict path -> list of (token_text, p, top1_text, top1_p, overridden)
Paths: 'type', 'type.id', 'date', 'date.role', 'date.id', 'party0', 'party0.role', 'party0.id', ...,
'subject', 'subject.id'. A token is assigned to every scalar whose inside it overlaps.
"""
import json

def scalars(text):
    """Yield (path, start, end, is_string) for every scalar in the JSON text (inside of strings)."""
    out = []
    i = 0
    n = len(text)
    def ws():
        nonlocal i
        while i < n and text[i] in " \t\r\n":
            i += 1
    def value(path):
        nonlocal i
        ws()
        if i >= n:
            return
        c = text[i]
        if c == '{':
            i += 1
            while True:
                ws()
                if i >= n: return
                if text[i] == '}':
                    i += 1; return
                if text[i] == ',':
                    i += 1; continue
                if text[i] != '"': i += 1; continue
                ks = i + 1
                i += 1
                while i < n and text[i] != '"':
                    i += 2 if text[i] == '\\' else 1
                key = text[ks:i]
                i += 1
                ws()
                if i < n and text[i] == ':': i += 1
                value(path + [key])
        elif c == '[':
            i += 1
            k = 0
            while True:
                ws()
                if i >= n: return
                if text[i] == ']':
                    i += 1; return
                if text[i] == ',':
                    i += 1; continue
                value(path + [k]); k += 1
        elif c == '"':
            s = i + 1
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == '\\' else 1
            out.append((path, s, i, True))
            i += 1
        else:
            s = i
            while i < n and text[i] not in ",]}":
                i += 1
            out.append((path, s, i, False))
    value([])
    return out

def name_of(path):
    if not path: return None
    k = path[0]
    if k == 'type':
        if len(path) == 1: return 'type'           # null
        return 'type' if path[1] == 0 else 'type.id'
    if k == 'date':
        if len(path) == 1: return 'date'
        return ['date', 'date.role', 'date.id'][path[1]] if path[1] < 3 else None
    if k == 'parties':
        if len(path) == 1: return 'parties.empty'
        if len(path) < 3: return None
        return [f'party{path[1]}', f'party{path[1]}.role', f'party{path[1]}.id'][path[2]] if path[2] < 3 else None
    if k == 'subject':
        if len(path) == 1: return 'subject'
        return 'subject' if path[1] == 0 else 'subject.id'
    return None

def fields(trace):
    text = ""
    spans = []
    for t in trace:
        s = len(text); text += t["token"]; spans.append((s, len(text)))
    result = {}
    values = {}
    for path, a, b, is_string in scalars(text):
        nm = name_of(path)
        if nm is None: continue
        values[nm] = text[a:b]
        toks = []
        for t, (s, e) in zip(trace, spans):
            if (s < b and a < e) or (a == b and s <= a < e):
                top = t.get("top") or []
                top1 = top[0] if top else None
                over = bool(top1) and top1["token"] != t["token"]
                toks.append((t["token"], t["p"], top1["token"] if top1 else None, top1["p"] if top1 else None, over))
        result[nm] = toks
    return text, values, result

def stats(toks):
    if not toks: return None
    ps = [p for _, p, *_ in toks]
    prod = 1.0
    for p in ps: prod *= p
    return {"min": min(ps), "first": ps[0], "prod": prod, "n": len(ps), "overridden": sum(1 for *_, o in toks if o)}

if __name__ == "__main__":
    reply = '{"type":["Invoice","p1.b2"],"date":["2025-05-01","invoice","p1.b4.f1"],"parties":[["Halvorsen Fixture Works LLC","issuer","p1.b1"],["Quillon Ridge Bakery, Inc.","customer","p1.b6.f1"]],"subject":["display shelving","p1.b7"]}'
    import re
    pieces = re.findall(r'[A-Za-z]+|\d+|\s|.', reply)
    trace = [{"token": p, "p": 0.9, "top": [{"token": p, "p": 0.9}]} for p in pieces]
    text, values, f = fields(trace)
    assert text == reply
    print(values)
    print({k: stats(v) for k, v in f.items()})
