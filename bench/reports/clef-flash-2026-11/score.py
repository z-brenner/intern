"""Score Clef-Flash's answers against the gold and against the 9B's recorded reading.

Usage: python3 -I score.py REQUESTS.jsonl [ANSWERS.jsonl] > summary.json
Party matching and the relation rule follow crates/intern-bench/src/score.rs (party_matches,
judge_parties, relation_correct), ported to Python.
"""
import json, sys, collections


def norm(v):
    return ' '.join(''.join(c for c in (v or '').lower() if c.isalnum() or c.isspace()).split())


def party_matches(gold, actual):
    g, a = norm(gold), norm(actual)
    return bool(g) and bool(a) and (g in a or a in g)


def party_sets(gold):
    sets = []
    if gold.get('parties') is not None:
        sets.append({'parties': gold.get('parties') or [], 'relation': gold.get('party_relation')})
    for s in gold.get('acceptable_party_sets') or []:
        sets.append({'parties': s.get('parties') or [], 'relation': s.get('relation')})
    return sets


def judge(gold, parties, relation):
    """(parties_correct, relation_correct, both) the way intern-bench scores a reading."""
    sets = party_sets(gold)
    if not sets:
        return None
    exact, overlap = [], []
    for i, s in enumerate(sets):
        matched = sum(1 for g in s['parties'] if any(party_matches(g, p) for p in parties))
        spurious = sum(1 for p in parties if not any(party_matches(g, p) for g in s['parties']))
        if matched == len(s['parties']) and spurious == 0:
            exact.append(i)
        if matched:
            overlap.append(i)
    pc = bool(exact)
    judged = exact or overlap or [0]
    rc = bool(parties) and relation is not None and any(sets[i]['relation'] == relation for i in judged)
    both = any(sets[i]['relation'] == relation for i in exact)
    return pc, rc, both


def gold_dates(gold):
    d = [gold.get('document_date')] if gold.get('document_date') else []
    return set(d + list(gold.get('acceptable_dates') or []))


def clef_reading(ans, relation):
    s = ans.get('sender', {}).get('choice')
    r = ans.get('recipient', {}).get('choice')
    s = None if s == 'none' else s
    r = None if r == 'none' else r
    if relation == 'from':
        ps = [s] if s else []
    elif relation in ('to', 'for'):
        ps = [r] if r else []
    elif relation == 'between':
        ps = [x for x in (s, r) if x]
        if len(ps) == 2 and norm(ps[0]) == norm(ps[1]):
            ps = ps[:1]
    else:
        ps = []
    return ps


def main():
    reqs = [json.loads(l) for l in open(sys.argv[1])]
    answers = {}
    if len(sys.argv) > 2:
        for l in open(sys.argv[2]):
            a = json.loads(l)
            answers[(a['set'], a['id'])] = a
    rows = []
    for q in reqs:
        m = q['meta']
        gold = m['gold']
        opts_d = set(q['request']['questions']['date']['criteria']) - {'none'}
        opts_p = list(q['request']['questions']['sender']['criteria'])
        gd = gold_dates(gold)
        row = {'set': m['set'], 'id': m['id'], 'gold_relation': gold.get('party_relation')}
        row['date_recall'] = bool(gd & opts_d) if gd else None
        sets = party_sets(gold)
        row['party_recall'] = any(party_matches(g, p) for s in sets for g in s['parties'] for p in opts_p) if sets else None
        qw = m['qwen']
        row['qwen_date'] = (qw['document_date'] in gd) if gd else None
        j = judge(gold, qw['parties'], qw['party_relation'])
        row['qwen_parties'], row['qwen_relation'], row['qwen_name'] = j if j else (None, None, None)
        row['qwen_token_min'] = qw.get('token_min')
        a = answers.get((m['set'], m['id']))
        if a and 'answers' in a:
            ans = a['answers']
            dc = ans['date']['choice']
            row['clef_date'] = (dc in gd) if gd else None
            row['clef_date_p'] = ans['date']['probabilities'].get(dc)
            rel = ans['relation']['choice']
            row['clef_relation_choice'] = rel
            j2 = judge(gold, clef_reading(ans, rel), rel)
            row['clef_parties'], row['clef_relation'], row['clef_name'] = j2 if j2 else (None, None, None)
            # slot fill: keep the 9B's word, let Clef pick who fills it
            j3 = judge(gold, clef_reading(ans, qw['party_relation']), qw['party_relation'])
            row['slot_parties'], row['slot_relation'], row['slot_name'] = j3 if j3 else (None, None, None)
            row['ms'] = a.get('ms')
            row['input_tokens'] = (a.get('usage') or {}).get('input_tokens')
        rows.append(row)

    def rate(key, filt=lambda r: True):
        v = [r[key] for r in rows if filt(r) and r.get(key) is not None]
        return [sum(v), len(v)]

    keys = ['date_recall', 'party_recall', 'qwen_date', 'qwen_parties', 'qwen_relation', 'qwen_name',
            'clef_date', 'clef_parties', 'clef_relation', 'clef_name', 'slot_parties', 'slot_relation', 'slot_name']
    summary = {'all': {k: rate(k) for k in keys}}
    for s in sorted({r['set'] for r in rows}):
        summary[s] = {k: rate(k, lambda r, s=s: r['set'] == s) for k in keys}
    answered = [r for r in rows if 'clef_date' in r]
    if answered:
        # where the two disagree
        def flips(a, b):
            return {'clef_right_qwen_wrong': sum(1 for r in answered if r.get(a) and r.get(b) is False),
                    'qwen_right_clef_wrong': sum(1 for r in answered if r.get(b) and r.get(a) is False)}
        summary['flips'] = {'date': flips('clef_date', 'qwen_date'), 'name': flips('clef_name', 'qwen_name'),
                            'slot_name': flips('slot_name', 'qwen_name')}
        ms = sorted(r['ms'] for r in answered if r.get('ms'))
        if ms:
            summary['ms_median'] = ms[len(ms) // 2]
            summary['ms_p90'] = ms[int(len(ms) * 0.9)]
        summary['relation_confusion'] = collections.Counter(
            f"{r['gold_relation']}->{r['clef_relation_choice']}" for r in answered).most_common()
    json.dump({'summary': summary, 'rows': rows}, sys.stdout, indent=1)


if __name__ == '__main__':
    main()
