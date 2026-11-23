"""Build Clef-Flash decision requests from recorded 9B readings.

Usage: python3 -I prepare.py SETS.json OUT.jsonl
SETS.json: [{"set": "real1", "recording": ".../recording.json", "report": ".../report.json",
             "gold": "scripts/realdocs-eval/sample.json"}, ...]

For every document with a gold reading, one /v1/systemone request asks three questions:
  date      - which printed date the document is filed under (options: every date found in the text)
  sender    - who issued, sent or wrote it         (options: candidate names, see below)
  recipient - who it is addressed to, or for        (same options)
  relation  - which connecting word the name takes (from / to / for / between)
Candidate names come only from the 9B's own reading (its parties before and after the engine) and
from organization-looking lines in the text. The gold is never used to build a request.
"""
import json, re, sys, datetime

MONTHS = {m: i + 1 for i, m in enumerate(
    ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'])}
MON = r'(jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:t(?:ember)?)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\.?'
PATTERNS = [
    ('mdy', re.compile(MON + r'\s+(\d{1,2})(?:st|nd|rd|th)?,?\s+(\d{4})', re.I)),
    ('dmy', re.compile(r'(\d{1,2})(?:st|nd|rd|th)?(?:\s+|-|\s+day\s+of\s+)' + MON + r'(?:,?\s+|-)(\d{4})', re.I)),
    ('iso', re.compile(r'\b(\d{4})-(\d{2})-(\d{2})\b')),
    ('num', re.compile(r'\b(\d{1,2})[/.](\d{1,2})[/.](\d{4}|\d{2})\b')),
]
LEGAL = re.compile(r'\b(inc|llc|l\.l\.c|ltd|limited|corp|corporation|company|co|plc|lp|llp|gmbh|n\.a|bank|'
                   r'authority|department|office|agency|commission|council|university|board|bureau|ministry|'
                   r'court|county|city|association|trust|foundation|institute|group|partners|services|'
                   r'administration|division|district)\b\.?', re.I)
STATE_CHARS = 10000  # head 7,500 + tail 2,500, about the 9B's 12,000-character budget


def iso(y, m, d):
    try:
        y = int(y)
        if y < 100:
            y += 2000 if y < 50 else 1900
        return datetime.date(y, int(m), int(d)).isoformat()
    except ValueError:
        return None


def find_dates(text):
    found = {}
    for kind, rx in PATTERNS:
        for m in rx.finditer(text):
            g = m.groups()
            if kind == 'mdy':
                v = iso(g[2], MONTHS[g[0][:3].lower()], g[1])
            elif kind == 'dmy':
                v = iso(g[2], MONTHS[g[1][:3].lower()], g[0])
            elif kind == 'iso':
                v = iso(*g)
            else:
                # US order first; the day-first reading too when both are valid (UK paper)
                v = [iso(g[2], g[0], g[1]), iso(g[2], g[1], g[0])]
            for v in (v if isinstance(v, list) else [v]):
                if not v or not ('1800' <= v[:4] <= '2040'):
                    continue
                if v not in found:
                    a, b = max(0, m.start() - 50), min(len(text), m.end() + 50)
                    found[v] = (m.start(), ' '.join(text[a:b].split()))
    return sorted(found.items(), key=lambda kv: kv[1][0])


def text_of(entry):
    src = ((entry or {}).get('extraction') or {}).get('source') or {}
    return '\n'.join(p.get('text') or '' for p in src.get('pages') or [])


def state_of(text):
    if len(text) <= STATE_CHARS:
        return text
    return text[:7500] + '\n[...]\n' + text[-2500:]


def org_lines(text):
    out = []
    for part in (text[:3000], text[-2500:]):
        for line in part.splitlines():
            line = line.strip(' |*#:-\t')
            if 4 <= len(line) <= 70 and LEGAL.search(line) and line[0].isupper() and len(line.split()) <= 8 \
                    and not re.search(r'\d{3}', line):
                out.append(line)
    return out


def norm(s):
    return ' '.join(''.join(c for c in s.lower() if c.isalnum() or c.isspace()).split())


def candidates(names):
    seen, out = set(), []
    for n in names:
        n = (n or '').strip()
        k = norm(n)
        if not k or k in seen:
            continue
        seen.add(k)
        out.append(n)
    return out[:14]


def main():
    sets = json.load(open(sys.argv[1]))
    out = open(sys.argv[2], 'w')
    n = 0
    for s in sets:
        rec = {d['id']: d for d in json.load(open(s['recording']))['documents']}
        rep = {r['id']: r for r in json.load(open(s['report']))['records']}
        gold = {d['id']: d for d in json.load(open(s['gold']))['documents']}
        for did, r in rep.items():
            if did not in gold or did not in rec or r.get('status') != 'completed':
                continue
            text = text_of(rec[did])
            if not text.strip():
                continue
            state = state_of(text)
            facts = ((r.get('model_proposal') or {}).get('facts') or {})
            model_names = [p.get('name') for p in facts.get('parties') or []]
            final = r.get('proposal') or {}
            names = candidates(list(final.get('parties') or []) + model_names + org_lines(text))
            dates = find_dates(text)[:30]  # the whole text, as the 9B's retrieval reads it; context in each option
            party_opts = {n_: None for n_ in names}
            party_opts['none'] = 'None of these names'
            date_opts = {d: f'printed as: "{ctx}"' for d, (_, ctx) in dates}
            date_opts['none'] = 'None of these dates'
            req = {
                'state': state,
                'questions': {
                    'date': {'type': 'choice', 'criteria': date_opts, 'instructions':
                             'Which date is this document filed under? Choose the date the document itself '
                             'carries: the date it was issued, sent or dated, or, for an agreement, the date it '
                             'takes effect. Do not choose the dates of other documents it refers to, deadlines, '
                             'periods, signature dates of attachments, or dates in footnotes.'},
                    'sender': {'type': 'choice', 'criteria': party_opts, 'instructions':
                               'Which party issued, sent or wrote this document? Choose the organization '
                               '(company, firm, agency) rather than the person who signed for it, unless that '
                               'person acts for themselves.'},
                    'recipient': {'type': 'choice', 'criteria': party_opts, 'instructions':
                                  'Which party is this document addressed to, issued to, or about? Choose the '
                                  'organization rather than an employee of it, unless an individual is the '
                                  'party.'},
                    'relation': {'type': 'choice', 'instructions':
                                 'A filing clerk names this document "<type> <word> <party>". Which word fits?',
                                 'criteria': {
                                     'between': 'An agreement, contract or amendment, or a notice made between '
                                                'two or more parties; both sides are named.',
                                     'from': 'An invoice, receipt, statement, memo, report, form or notice '
                                             'issued by an organization; the issuer is named.',
                                     'to': 'A letter or other correspondence; the party it is addressed to '
                                           'is named.',
                                     'for': 'A notice, filing, minutes, certificate or record about a '
                                            'particular person or entity; that subject is named.'}},
                },
            }
            meta = {'set': s['set'], 'id': did, 'gold': gold[did]['gold'],
                    'qwen': {'document_date': final.get('document_date'), 'parties': final.get('parties') or [],
                             'party_relation': final.get('party_relation'),
                             'token_min': (r.get('token_confidence') or {}).get('min'),
                             'readiness': r.get('readiness'),
                             'scores': {k: (r.get('scores') or {}).get(k) for k in
                                        ('date_correct', 'parties_correct', 'relation_correct')}},
                    'n_dates': len(dates), 'n_names': len(names), 'state_chars': len(state)}
            out.write(json.dumps({'meta': meta, 'request': req}) + '\n')
            n += 1
    print(n, 'requests')


if __name__ == '__main__':
    main()
