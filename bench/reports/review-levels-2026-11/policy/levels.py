"""Apply the three levels to replay reports. Usage: levels.py dev | heldout
Kept as run on 2026-11-15. The paths are the scratch directory of the session that ran it: the replay reports
are intern-bench reports made from the live recordings named in README.md, and are not in the repository."""
import json, sys, collections, re
sys.path.insert(0,'.')
import severity as sev
S='/tmp/claude-0/-home-user-intern/6a9add71-b949-5750-bc98-54bb7979afa8/scratchpad'
ALWAYS={'DATE_MISSING','DATE_UNSUPPORTED','DATE_IS_PERIOD','DATE_IS_DEADLINE','DATE_IS_FORMULA','DESCRIPTION_INVALID','PARSER_WARNING','MODEL_REQUESTED_REVIEW'}
BAL=ALWAYS|{'DATE_AMBIGUOUS','DATE_IMPLAUSIBLE','PARTY_UNSUPPORTED','PARTY_MISSING'}
def prep(r):
    s,why=sev.classify(r)
    rs=[x for x in r['review_reasons'] if x!='LOW_CONFIDENCE']
    name=r.get('filename') or ''
    if any('TODAY()' in (f.get('why') or '') and f.get('date')==name[:10] for f in r['_gold'].get('forbidden_dates') or []):
        if 'DATE_IS_FORMULA' not in rs: rs.append('DATE_IS_FORMULA')
    tok=r['token_confidence']['min'] if r.get('token_confidence') else None
    flags={w.split(':')[0] for w in why}
    return dict(id=r['id'],set=r['_set'],name=name,gold=(r.get('gold_filenames') or ['(none)'])[0],sev=s,why=why,flags=flags,reasons=set(rs),tok=tok,
                has_date=bool(sev.DATE.match(name)),doc_type=(r.get('proposal') or {}).get('document_type'),desc=r.get('description') or '',
                strict='wrong' if (s=='minor' and flags&{'person','class'}) else s, ready=r['readiness']=='ready')
def held(d, level):
    h=set()
    if not d['has_date']: h.add('NO_DATE')
    if level=='strict':
        h|=d['reasons']
        if d['tok'] is None or d['tok']<0.8: h.add('TOKEN<0.8')   # engine: no token probability -> gate skipped; none absent here (checked below)
        return h
    h|=d['reasons']&ALWAYS
    h|=d['reasons']&{'DATE_AMBIGUOUS','DATE_IMPLAUSIBLE','PARTY_MISSING'}
    if not d['doc_type']: h.add('TYPE_IS_DOCUMENT')
    if {'TYPE_UNSUPPORTED','TYPE_INFERRED'}<=d['reasons'] and 'TYPE_MISSING' not in d['reasons']: h.add('TITLE_STOOD_IN')
    if 'PARTY_UNSUPPORTED' in d['reasons']:
        if level=='balanced' or d['tok'] is None or d['tok']<0.6: h.add('PARTY_UNSUPPORTED')
    return h
def load(pairs):
    out=[]
    for setname,report,gold in pairs:
        g={x['id']:x for x in json.load(open(gold))['documents']}
        for r in json.load(open(report))['records']:
            if r['status']!='completed': continue
            r=dict(r); r['_set']=setname; r['_gold']=g.get(r['id'],{}).get('gold',{})
            out.append(prep(r))
    return out
def table(ds,label):
    print(f"\n## {label}: {len(ds)} documents; right/minor/really wrong overall {collections.Counter(d['sev'] for d in ds)}")
    for lv in ('strict','balanced','relaxed'):
        f=[d for d in ds if not held(d,lv)]
        c=collections.Counter(d['sev'] for d in f); cs=collections.Counter(d['strict'] for d in f)
        n=len(f) or 1
        print(f"- {lv:8s}: filed {len(f)} of {len(ds)} ({len(f)/len(ds):.0%}); right {c['right']} / minor {c['minor']} / really wrong {c['wrong']} ({c['wrong']/n:.1%} of filed); strict scale {cs['right']}/{cs['minor']}/{cs['wrong']}")
if __name__=='__main__':
    which=sys.argv[1]
    if which=='dev':
        import load as L
        rows=[r for r in L.records(models=('9b',)) if r['status']=='completed']
        ds=[prep(r) for r in rows]
        table(ds,'development, all'); print('no token prob:',sum(1 for d in ds if d['tok'] is None))
    else:
        ds=load([('real7',f'{S}/replay/final19-real7/real7-9b.json',f'{S}/gold/real7.json'),('real8',f'{S}/replay/a19-real8/real8-9b.json',f'{S}/gold/real8.json')])
        table(ds,'held-out pooled (real7 + real8)')
        for s in ('real7','real8'): table([d for d in ds if d['set']==s],s)
        print('no token prob:',sum(1 for d in ds if d['tok'] is None))
        json.dump([{**d,'reasons':sorted(d['reasons']),'flags':sorted(d['flags']),'held':{lv:sorted(held(d,lv)) for lv in ('strict','balanced','relaxed')}} for d in ds],open('rows-heldout.json','w'),indent=1)
        for lv,prev in (('balanced','strict'),('relaxed','balanced')):
            print(f"\n### newly filed at {lv} and not exactly right")
            for d in ds:
                if not held(d,lv) and held(d,prev) and d['sev']!='right':
                    print(f"- [{d['sev']}] {d['id']} tok={d['tok']:.2f} reasons={sorted(d['reasons'])}\n    name: {d['name']}\n    gold: {d['gold']}\n    why: {d['why']}\n    desc: {d['desc'][:200]}")
        print("\n### filed at strict and not exactly right")
        for d in ds:
            if not held(d,'strict') and d['sev']!='right': print(f"- [{d['sev']}] {d['id']} {d['name']} | gold {d['gold']} | {d['why']}")
