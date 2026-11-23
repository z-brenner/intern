"""Matched-volume comparison: within each variant's eligible pool, rank readings by an out-of-fold
verifier score vs by token probability, file the k safest, count really wrong among them.
Usage: curves.py SETS.json OUT_DIR"""
import json, sys, collections, os
import numpy as np
sys.path.insert(0, os.path.dirname(__file__))
import verifier as V
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler
from sklearn.pipeline import make_pipeline
from sklearn.impute import SimpleImputer
from sklearn.metrics import roc_auc_score

sets = json.load(open(sys.argv[1])); out = sys.argv[2]; os.makedirs(out, exist_ok=True)
rows = V.load(sets)
keys = sorted(rows[0]['f'])
X = np.array([[r['f'][k] for k in keys] for r in rows], float)
y = np.array([r['sev'] == 'wrong' for r in rows], int)
g = np.array([r['set'] for r in rows])
tok = np.array([r['f']['tok_min'] for r in rows]); tok = np.where(np.isnan(tok), 0, tok)
test_sets = {s['set'] for s in sets if s.get('role') == 'test'}

def models():
    return {
        'lgbm': lambda: __import__('lightgbm').LGBMClassifier(n_estimators=150, learning_rate=0.05, num_leaves=5, min_child_samples=10,
                                                         subsample=0.8, subsample_freq=1, colsample_bytree=0.7, verbose=-1, random_state=0),
        'logreg': lambda: make_pipeline(SimpleImputer(strategy='median'), StandardScaler(), LogisticRegression(C=0.1, max_iter=2000)),
    }

report = {}
for variant in ('v1', 'v2'):
    elig = np.array([not V.hard_hold(r, variant) for r in rows])
    for mname, mk in models().items():
        p = np.full(len(rows), np.nan)
        if test_sets:
            folds = [(~np.isin(g, list(test_sets)), np.isin(g, list(test_sets)))]
        else:
            folds = [(g != s, g == s) for s in sorted(set(g))]
        for tr, te in folds:
            tr = tr & elig; te = te & elig
            if te.sum() == 0 or len(set(y[tr])) < 2: continue
            Xtr = np.nan_to_num(X[tr], nan=-1) if mname == 'lgbm' else X[tr]
            Xte = np.nan_to_num(X[te], nan=-1) if mname == 'lgbm' else X[te]
            m = mk(); m.fit(Xtr, y[tr]); p[te] = m.predict_proba(Xte)[:, 1]
        ok = elig & ~np.isnan(p)
        sev = np.array([r['sev'] for r in rows])
        res = {'pool': int(ok.sum()), 'pool_wrong': int(y[ok].sum()), 'pool_right': int((sev[ok] == 'right').sum())}
        if len(set(y[ok])) == 2:
            res['auc_verifier'] = round(float(roc_auc_score(y[ok], p[ok])), 3)
            res['auc_token'] = round(float(roc_auc_score(y[ok], -tok[ok])), 3)
        idx = np.where(ok)[0]
        by_v = idx[np.argsort(p[idx], kind='stable')]
        by_t = idx[np.argsort(-tok[idx], kind='stable')]
        curve = []
        for k in sorted(set([10, 20, 30, 40, 50, 75, 100, 125, 150, 200, 250, len(idx)])):
            if k > len(idx): continue
            cv = collections.Counter(sev[by_v[:k]]); ct = collections.Counter(sev[by_t[:k]])
            curve.append({'filed': k, 'verifier_r/m/w': [cv['right'], cv['minor'], cv['wrong']], 'token_r/m/w': [ct['right'], ct['minor'], ct['wrong']]})
        res['curve'] = curve
        report[f'{variant}-{mname}'] = res
json.dump(report, open(os.path.join(out, 'curves.json'), 'w'), indent=1)
for k, v in report.items():
    print(k, {a: b for a, b in v.items() if a != 'curve'})
    for c in v['curve']: print('   ', c)
