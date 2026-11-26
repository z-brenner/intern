"""Gate verifier experiment for ATLAS: can a small learned model decide which Qwen readings
to file without review better than the token-probability gate?

Usage: python3 verifier.py SETS.json OUT_DIR
SETS.json: [{"set": "real1", "report": "...json", "recording": "...json", "gold": "...json"}, ...]
Each report is an intern-bench replay of the recording at the shipped gate (0.7).

Labels come from the review-levels severity scale (right / minor / really wrong).
Evaluation is leave-one-set-out when there are 2+ sets, else 5-fold CV (plumbing check only).
"""
import json, os, re, sys, collections, math
import numpy as np

POLICY = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'review-levels-2026-11', 'policy')
sys.path.insert(0, POLICY)
import severity as sev  # noqa: E402
import levels as lv  # noqa: E402

REASONS = ['LOW_CONFIDENCE', 'TYPE_MISSING', 'TYPE_INFERRED', 'TYPE_UNSUPPORTED', 'PARTY_UNSUPPORTED',
           'PARTY_MISSING', 'DESCRIPTION_UNSUPPORTED', 'DESCRIPTION_INVALID', 'DATE_UNSUPPORTED',
           'DATE_MISSING', 'DATE_AMBIGUOUS', 'DATE_IMPLAUSIBLE', 'DATE_IS_PERIOD', 'DATE_IS_DEADLINE',
           'DATE_IS_FORMULA', 'PARSER_WARNING', 'MODEL_REQUESTED_REVIEW']
CLASSES = ['amendment', 'notice', 'issued', 'agreement', 'email', 'letter', 'form', 'record', 'unknown']
ROLES = ['client', 'contractor', 'employer', 'employee', 'buyer', 'seller', 'landlord', 'tenant', 'issuer',
         'recipient', 'vendor', 'customer', 'borrower', 'lender', 'licensor', 'licensee', 'sender',
         'addressee', 'other']
LEGAL = re.compile(r'\b(inc|llc|ltd|limited|corp|corporation|co|company|plc|lp|llp|gmbh|pty|bank|authority|'
                   r'department|office|agency|commission|council|university|college|board|bureau|ministry|'
                   r'court|county|city|state|association|society|trust|foundation|institute|group)\b\.?', re.I)
DATE_RX = re.compile(r'\b(?:\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}|\d{4}-\d{2}-\d{2}|'
                     r'(?:jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?\s+\d{1,2},?\s+\d{4}|'
                     r'\d{1,2}\s+(?:jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?,?\s+\d{4})\b', re.I)
EFFECTIVE = re.compile(r'effective|dated|as of|date of (?:notice|letter|issue)|invoice date|issued', re.I)
OTHER_DOC = re.compile(r'\bdated\b|agreement dated|master|under the|pursuant to', re.I)


def text_of(entry):
    ex = (entry or {}).get('extraction') or {}
    src = ex.get('source') or {}
    return '\n'.join(p.get('text') or '' for p in src.get('pages') or [])


def trace_feats(entry):
    """per-field token probabilities from a traced recording: walk the reply text token by token and
    assign each token to the top-level key it falls under (type, date, parties, subject); keep only
    tokens holding letters or digits (the grammar forces the punctuation)."""
    nan = np.nan
    out = {'tr_type_min': nan, 'tr_date_min': nan, 'tr_party_min': nan, 'tr_subject_min': nan,
           'tr_type_margin': nan, 'tr_date_margin': nan, 'tr_party_margin': nan, 'tr_party_mean': nan,
           'tr_date_mean': nan, 'tr_type_mean': nan, 'tr_role_min': nan}
    for x in (entry or {}).get('exchanges') or []:
        tr = (x.get('reply') or {}).get('token_trace')
        if not tr:
            continue
        text = ''
        per = collections.defaultdict(list)
        marg = collections.defaultdict(list)
        for t in tr:
            tok = t.get('token') or ''
            p = t.get('p')
            text += tok
            m = re.findall(r'"(type|date|parties|subject)"\s*:', text)
            key = m[-1] if m else None
            if key is None or p is None or not re.search(r'[A-Za-z0-9]', tok):
                continue
            # a party's role word sits after its name in the same array; count role tokens apart
            if key == 'parties' and re.search(r'"\s*,\s*"[^"]*$', text) and re.fullmatch(r'\s*[a-z]+', tok) and text.count('"') % 2 == 1:
                per['role'].append(p)
            per[key].append(p)
            top = sorted([a.get('p') or 0 for a in t.get('top') or []], reverse=True)
            if len(top) >= 2:
                marg[key].append(top[0] - top[1])
        nm = {'type': 'type', 'date': 'date', 'parties': 'party', 'subject': 'subject'}
        for k, v in nm.items():
            if per[k]:
                out[f'tr_{v}_min'] = min(per[k])
            if k in ('type', 'date', 'parties') and per[k]:
                out[f'tr_{v}_mean'] = float(np.mean(per[k]))
            if k in ('type', 'date', 'parties') and marg[k]:
                out[f'tr_{v}_margin'] = min(marg[k])
        if per['role']:
            out['tr_role_min'] = min(per['role'])
    return out


def party_feats(name, text):
    n = name or ''
    first = text[:400].lower()
    idx = text.lower().find(n.lower()) if n else -1
    before = text[max(0, idx - 40):idx].lower() if idx >= 0 else ''
    return {
        'is_org': 1.0 if LEGAL.search(n) else 0.0,
        'n_words': len(n.split()),
        'in_head': 1.0 if n and n.lower() in first else 0.0,
        'found': 1.0 if idx >= 0 else 0.0,
        'pos': idx / max(1, len(text)) if idx >= 0 else 1.0,
        'after_dear_to': 1.0 if re.search(r'(dear|to:|attn|attention)\s*$', before.strip() + ' ') or 'dear' in before or 'to:' in before else 0.0,
    }


def features(r, entry):
    f = {}
    tc = r.get('token_confidence') or {}
    f['tok_min'] = tc.get('min', np.nan)
    f['tok_mean'] = tc.get('mean', np.nan)
    f['tok_n'] = tc.get('tokens', np.nan)
    reasons = set(r.get('review_reasons') or [])
    for x in REASONS:
        f['r_' + x] = 1.0 if x in reasons else 0.0
    f['n_reasons'] = len(reasons - {'LOW_CONFIDENCE'})
    sc = r.get('scores') or {}
    for k in ('cited_support_rate', 'facts_proposed', 'facts_unsupported', 'miscited_ids', 'unknown_ids'):
        v = sc.get(k)
        f[k] = float(v) if isinstance(v, (int, float)) else np.nan
    rel = r.get('relation') or {}
    cls = rel.get('document_class') or 'unknown'
    for c in CLASSES:
        f['cls_' + c] = 1.0 if cls == c else 0.0
    parties = rel.get('parties') or []
    f['n_parties'] = len(parties)
    sup = collections.Counter((p.get('role_support') or 'none') for p in parties)
    for s in ('cited', 'context', 'unsupported', 'none'):
        f['role_support_' + s] = sup.get(s, 0)
    f['n_document_role'] = sum(1 for p in parties if p.get('document_role'))
    f['role_disagree'] = sum(1 for p in parties if p.get('document_role') and p.get('proposed_role') and p['document_role'] != p['proposed_role'])
    basis = (rel.get('basis') or '').lower()
    f['basis_reply_roles'] = 1.0 if 'reply' in basis else 0.0
    f['basis_default'] = 1.0 if ('default' in basis or 'first party' in basis) else 0.0
    prop = r.get('proposal') or {}
    rel_word = (prop.get('party_relation') or 'none')
    for w in ('between', 'from', 'to', 'for', 'with', 'none'):
        f['rel_' + w] = 1.0 if rel_word == w else 0.0
    mp = (r.get('model_proposal') or {}).get('facts') or {}
    mtype = (mp.get('document_type') or '')
    ptype = (prop.get('document_type') or '')
    f['type_changed'] = 1.0 if mtype and ptype and sev.norm(mtype) != sev.norm(ptype) else 0.0
    f['type_generic'] = 1.0 if (not ptype or set(sev.norm(ptype).split()) <= sev.GENERIC_TYPES) else 0.0
    f['type_words'] = len(ptype.split())
    text = text_of(entry)
    f['text_chars'] = len(text)
    f['pages'] = float(r.get('pages') or 0)
    tm = r.get('timings') or {}
    f['ocr_pages'] = float(tm.get('worker_ocr_pages') or 0)
    f['ocr_share'] = f['ocr_pages'] / max(1.0, f['pages'])
    f['context_tokens'] = float(tm.get('context_tokens') or 0)
    f['generated_tokens'] = float(tm.get('generated_tokens') or 0)
    f['type_in_head'] = 1.0 if ptype and sev.norm(ptype) in sev.norm(text[:600]) else 0.0
    dates = DATE_RX.findall(text)
    f['n_dates'] = len(dates)
    f['n_distinct_dates'] = len(set(d.lower() for d in dates))
    ev = prop.get('evidence') or {}
    dline = (ev.get('date') or '')
    f['date_line_effective_cue'] = 1.0 if EFFECTIVE.search(dline) else 0.0
    f['date_line_other_doc_cue'] = 1.0 if OTHER_DOC.search(dline) else 0.0
    f['date_line_words'] = len(dline.split())
    role = mp.get('date_role') or prop.get('date_role') or ''
    f['date_role_effective'] = 1.0 if 'effective' in role else 0.0
    f['date_role_signed'] = 1.0 if 'sign' in role or 'execut' in role else 0.0
    name = r.get('filename') or ''
    f['has_date'] = 1.0 if sev.DATE.match(name) else 0.0
    m = sev.DATE.match(name)
    if m:
        y = int(m.group(1)[:4])
        years = [int(x) for x in re.findall(r'\b(19\d{2}|20\d{2})\b', text)]
        f['date_year_is_max'] = 1.0 if years and y >= max(years) else 0.0
        f['date_year_is_min'] = 1.0 if years and y <= min(years) else 0.0
    else:
        f['date_year_is_max'] = f['date_year_is_min'] = np.nan
    shown = [p for p in prop.get('parties') or [] if sev.norm(p) and sev.norm(p) in sev.norm(name)]
    f['n_shown_parties'] = len(shown)
    pf = [party_feats(p, text) for p in shown] or [party_feats('', text)]
    for k in pf[0]:
        vals = [d[k] for d in pf]
        f['p_' + k + '_min'] = min(vals)
        f['p_' + k + '_max'] = max(vals)
    f['p_person_shown'] = 1.0 if any(d['is_org'] == 0 and 2 <= d['n_words'] <= 4 for d in pf if shown) else 0.0
    f.update(trace_feats(entry))
    return f


def load(sets):
    rows = []
    for s in sets:
        gold = {d['id']: d for d in json.load(open(s['gold']))['documents']}
        rec = {d['id']: d for d in json.load(open(s['recording']))['documents']} if s.get('recording') else {}
        for r in json.load(open(s['report']))['records']:
            if r.get('status') != 'completed':
                continue
            r = dict(r)
            r['_set'] = s['set']
            r['_gold'] = gold.get(r['id'], {}).get('gold', {})
            d = lv.prep(r)
            rows.append({'id': r['id'], 'set': s['set'], 'sev': d['sev'], 'why': d['why'], 'name': d['name'],
                         'gold': d['gold'], 'ready': r.get('readiness') == 'ready', 'd': d,
                         'f': features(r, rec.get(r['id']))})
    return rows


HARD_V1 = None  # V1: every reason the engine raises except LOW_CONFIDENCE stays a hold


def hard_hold(row, variant):
    d = row['d']
    if not d['has_date']:
        return True
    if variant == 'v1':  # replace only the token-probability gate
        return bool(d['reasons'] - {'LOW_CONFIDENCE'})
    # v2: replace the token gate and balanced's soft reasons; keep balanced's hard holds
    h = lv.held(d, 'balanced') - {'PARTY_UNSUPPORTED'}
    return bool(h)


def fit_predict(Xtr, ytr, Xte, seed=0):
    import lightgbm as lgb
    m = lgb.LGBMClassifier(n_estimators=200, learning_rate=0.05, num_leaves=7, min_child_samples=8,
                           subsample=0.8, subsample_freq=1, colsample_bytree=0.8, reg_lambda=1.0,
                           random_state=seed, verbose=-1)
    m.fit(Xtr, ytr)
    return m.predict_proba(Xte)[:, 1], m


def oof(X, y, groups, seed=0):
    """out-of-fold P(really wrong) inside the training data, by group (set) or 5 folds."""
    from sklearn.model_selection import GroupKFold, StratifiedKFold
    p = np.zeros(len(y))
    ug = sorted(set(groups))
    if len(ug) >= 3:
        splits = GroupKFold(n_splits=min(5, len(ug))).split(X, y, groups)
    else:
        splits = StratifiedKFold(n_splits=5, shuffle=True, random_state=seed).split(X, y)
    for a, b in splits:
        if len(set(y[a])) < 2:
            p[b] = y[a].mean()
            continue
        p[b], _ = fit_predict(X[a], y[a], X[b], seed)
    return p


def pick_threshold(p, y, eligible, max_wrong_share):
    """largest threshold t on P(wrong) whose filed set (eligible & p < t) keeps really-wrong share <= cap."""
    best = 0.0
    for t in sorted(set(np.round(p[eligible], 4))) + [1.01]:
        f = eligible & (p < t)
        if f.sum() == 0:
            continue
        if y[f].mean() <= max_wrong_share + 1e-9:
            best = t
    return best


def count(rows, mask):
    c = collections.Counter(r['sev'] for r, m in zip(rows, mask) if m)
    return {'filed': int(sum(mask)), 'right': c['right'], 'minor': c['minor'], 'wrong': c['wrong']}


def main():
    sets = json.load(open(sys.argv[1]))
    out = sys.argv[2]
    os.makedirs(out, exist_ok=True)
    rows = load(sets)
    keys = sorted(rows[0]['f'])
    X = np.array([[r['f'][k] for k in keys] for r in rows], dtype=float)
    y = np.array([1 if r['sev'] == 'wrong' else 0 for r in rows])
    groups = np.array([r['set'] for r in rows])
    print(f'{len(rows)} readings; {collections.Counter(r["sev"] for r in rows)}; sets {collections.Counter(groups)}')
    ug = sorted(set(groups))
    test_sets = {s_['set'] for s_ in sets if s_.get('role') == 'test'}
    if test_sets:
        tr_idx = np.where(~np.isin(groups, list(test_sets)))[0]
        folds = [(tr_idx, np.where(groups == g)[0], g) for g in sorted(test_sets)]
        folds.append((tr_idx, np.where(np.isin(groups, list(test_sets)))[0], 'pooled-test'))
    elif len(ug) >= 2:
        folds = [(np.where(groups != g)[0], np.where(groups == g)[0], g) for g in ug]
    else:
        from sklearn.model_selection import StratifiedKFold
        folds = [(a, b, f'fold{i}') for i, (a, b) in enumerate(StratifiedKFold(5, shuffle=True, random_state=0).split(X, y))]
    caps = {'v1': 0.0, 'v2': 0.05}
    results = collections.defaultdict(lambda: collections.Counter())
    per_fold = []
    pred_all = np.full(len(rows), np.nan)
    for tr, te, name in folds:
        if len(set(y[tr])) < 2:
            continue
        p_oof = oof(X[tr], y[tr], groups[tr])
        p_te, model = fit_predict(X[tr], y[tr], X[te])
        pred_all[te] = p_te
        rec = {'fold': name, 'n': len(te)}
        base_strict = np.array([rows[i]['ready'] for i in te])
        base_bal = np.array([not lv.held(rows[i]['d'], 'balanced') for i in te])
        rec['strict'] = count([rows[i] for i in te], base_strict)
        rec['balanced'] = count([rows[i] for i in te], base_bal)
        for v, cap in caps.items():
            elig_tr = np.array([not hard_hold(rows[i], v) for i in tr])
            t = pick_threshold(p_oof, y[tr], elig_tr, cap)
            elig_te = np.array([not hard_hold(rows[i], v) for i in te])
            filed = elig_te & (p_te < t)
            rec[v] = count([rows[i] for i in te], filed)
            rec[v + '_threshold'] = float(t)
        per_fold.append(rec)
        if name != 'pooled-test':
            for k in ('strict', 'balanced', 'v1', 'v2'):
                results[k].update(rec[k])
    # ranking quality vs token probability, among readings with a date and no hard reason (v1 eligible)
    from sklearn.metrics import roc_auc_score
    el = np.array([not hard_hold(r, 'v1') for r in rows]) & ~np.isnan(pred_all)
    auc = {}
    if el.sum() and len(set(y[el])) == 2:
        tok = np.array([r['f']['tok_min'] for r in rows])
        ok = el & ~np.isnan(tok)
        auc['verifier'] = float(roc_auc_score(y[el], pred_all[el]))
        auc['token_min (inverted)'] = float(roc_auc_score(y[ok], -tok[ok]))
        auc['n'] = int(el.sum()); auc['n_wrong'] = int(y[el].sum())
    summary = {'readings': len(rows), 'severity': dict(collections.Counter(r['sev'] for r in rows)),
               'totals': {k: dict(v) for k, v in results.items()}, 'auc_v1_eligible': auc, 'folds': per_fold}
    json.dump(summary, open(os.path.join(out, 'summary.json'), 'w'), indent=1)
    # importances on all data
    _, m = fit_predict(X, y, X[:1])
    imp = sorted(zip(m.booster_.feature_importance('gain'), keys), reverse=True)[:20]
    json.dump([[k, float(g)] for g, k in imp], open(os.path.join(out, 'importance.json'), 'w'), indent=1)
    print(json.dumps({k: v for k, v in summary.items() if k != 'folds'}, indent=1))
    for rec in per_fold:
        print(rec)
    print('top features:', [k for _, k in imp[:12]])
    with open(os.path.join(out, 'predictions.jsonl'), 'w') as fh:
        for r, p in zip(rows, pred_all):
            fh.write(json.dumps({'set': r['set'], 'id': r['id'], 'sev': r['sev'], 'p_wrong': None if np.isnan(p) else float(p),
                                 'tok_min': r['f']['tok_min'], 'ready': r['ready'], 'name': r['name'], 'gold': r['gold']}) + '\n')


if __name__ == '__main__':
    main()
