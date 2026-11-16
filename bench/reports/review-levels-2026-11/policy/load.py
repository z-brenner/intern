"""Load the final19 replay reports: one record per (document, model), from the newest recording
in which that document was actually read (live18 over traced over dev).
Development sets only: internbench, real1..real6. Never touches real7/real8/heldout/live19."""
import json, os
S='/tmp/claude-0/-home-user-intern/6a9add71-b949-5750-bc98-54bb7979afa8/scratchpad'
REPO='/home/user/intern'
PREF={'9b':['final19-live18','final19-traced','final19-dev'],'4b':['final19-dev']}
SETS=['internbench','real1','real2','real3','real4','real5','real6']
def gold_path(s):
    return REPO+'/bench/gold.json' if s=='internbench' else f'{S}/gold/{s}.json'
def load_gold(s):
    g=json.load(open(gold_path(s)))
    return {d['id']:d for d in g['documents']}
def records(base=None, models=('9b','4b')):
    """base: optional dict mapping source name (e.g. 'final19-live18') -> directory of replay reports."""
    rows=[]
    for m in models:
        for s in SETS:
            gold=load_gold(s); best={}; order=[]
            for src in PREF[m]:
                d=(base or {}).get(src, f'{S}/replay/{src}')
                p=f'{d}/{s}-{m}.json'
                if not os.path.exists(p): continue
                for r in json.load(open(p))['records']:
                    if r['id'] not in best: order.append(r['id'])
                    cur=best.get(r['id'])
                    if cur is None or (cur['status']!='completed' and r['status']=='completed'):
                        r=dict(r); r['_src']=src; best[r['id']]=r
            for i in order:
                r=best[i]; r['_set']=s; r['_model']=m
                r['_gold']=gold.get(i,{}).get('gold',{}); r['_goldmeta']=gold.get(i,{})
                rows.append(r)
    return rows
if __name__=='__main__':
    import collections
    rows=records()
    c=collections.Counter((r['_model'],r['_set'],r['_src'],r['status']) for r in rows)
    for k,v in sorted(c.items()): print(k,v)
    print(len(rows), collections.Counter((r['_model'],r['status']) for r in rows))
