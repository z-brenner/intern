"""Send prepared requests to a llama-server running a decision model, one at a time.

Usage: python3 -I ask.py REQUESTS.jsonl SET OUT.jsonl [URL]
Writes one line per document: {"set", "id", "answers", "usage", "ms"} or {"set", "id", "error", "ms"}.
"""
import json, sys, time, urllib.request

reqs, want, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
url = sys.argv[4] if len(sys.argv) > 4 else 'http://127.0.0.1:8080/v1/systemone'
done = set()
try:
    for l in open(out_path):
        a = json.loads(l)
        if 'answers' in a:
            done.add(a['id'])
except FileNotFoundError:
    pass
out = open(out_path, 'a')
n = 0
for l in open(reqs):
    q = json.loads(l)
    m = q['meta']
    if m['set'] != want or m['id'] in done:
        continue
    body = json.dumps(dict(q['request'], model='clef-flash')).encode()
    t = time.time()
    try:
        r = urllib.request.urlopen(urllib.request.Request(url, body, {'Content-Type': 'application/json'}),
                                   timeout=1800)
        res = json.loads(r.read())
        row = {'set': want, 'id': m['id'], 'answers': res.get('answers'), 'usage': res.get('usage'),
               'ms': round((time.time() - t) * 1000)}
    except Exception as e:  # keep going; the error is recorded and scored as unanswered
        body_txt = getattr(e, 'read', lambda: b'')()
        row = {'set': want, 'id': m['id'], 'error': f'{e} {body_txt[:500]!r}', 'ms': round((time.time() - t) * 1000)}
    out.write(json.dumps(row) + '\n')
    out.flush()
    n += 1
    print(n, m['id'], row.get('ms'), 'error' if 'error' in row else (row.get('usage') or {}).get('input_tokens'),
          flush=True)
