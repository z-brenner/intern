import os,json,sys,collections
rows=json.load(open(sys.argv[1]))
modes=list(rows[0]['sc'])
def filed(m,t): return [r for r in rows if r['sc'][m]>=t]
def tally(fs): c=collections.Counter(r['sev'] for r in fs); return c['right'],c['minor'],c['wrong']
print('today @0.7:',tally(filed('min (today)',0.7)))
print('\nBest right names filed for at most K really-wrong (threshold chosen per variant, in-sample):')
print('variant'.ljust(40),' '.join(f'K={k}'.rjust(12) for k in range(0,6)))
for m in modes:
    ts=sorted({r['sc'][m] for r in rows},reverse=True)
    best={}
    for t in ts:
        rgt,mn,wr=tally(filed(m,t))
        if wr<=5 and (wr not in best or rgt>best[wr][0]): best[wr]=(rgt,mn,round(t,3))
    cells=[]; run=None
    for k in range(6):
        cand=[best[w] for w in best if w<=k]
        b=max(cand) if cand else None
        cells.append(f'{b[0]}r/{b[1]}m@{b[2]}' if b else '-')
    print(m.ljust(40),' '.join(c.rjust(12) for c in cells))
# AUC of right vs wrong
def auc(m):
    R=[r['sc'][m] for r in rows if r['sev']=='right'];W=[r['sc'][m] for r in rows if r['sev']=='wrong']
    return sum((a>b)+0.5*(a==b) for a in R for b in W)/len(R)/len(W)
print('\nAUC right vs really wrong:'); [print(' ',m.ljust(40),round(auc(m),3)) for m in modes]
