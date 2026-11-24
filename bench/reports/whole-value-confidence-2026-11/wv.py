import os,json,sys,re,math,collections
sys.path.insert(0,os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','review-levels-2026-11','policy'))
sys.path.insert(0,os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','review-gate-2026-11','policy'))
import severity as sev
from trace import scalars,name_of
D=sys.argv[1]
sets=dict(l.split() for l in open(f'{D}/sets.txt'))
def valpart(t):  # the part of a token that is inside the string value: up to an unescaped quote
    i=t.find('"'); return t if i<0 else t[:i]
def score(tr, mode):
    text='';spans=[]
    for t in tr: s=len(text); text+=t['token']; spans.append((s,len(text)))
    vals={}
    for path,a,b,isstr in scalars(text):
        nm=name_of(path)
        if nm=='date' or (nm and nm.startswith('party') and '.' not in nm):
            vals[nm]=(a,b,text[a:b])
    out={}
    for nm,(a,b,v) in vals.items():
        toks=[(t,s,e) for t,(s,e) in zip(tr,spans) if s<b and a<e]
        lp=0.0; mn=1.0
        for j,(t,s,e) in enumerate(toks):
            p=t['p']; tok=t['token']; alts=[z for z in (t.get('top') or []) if z['token']!=tok]
            if mode.get('fmt'):
                # mass of alternatives that write the same value characters (a closing token's variants)
                mine=text[max(s,a):min(e,b)]
                if e>b:  # token runs past the value's end: alternatives that end the value the same way
                    p+=sum(z['p'] for z in alts if valpart(z['token'])==mine and '"' in z['token'])
            if mode.get('case'):
                k=lambda x: re.sub(r'\s+','',x).lower()
                p+=sum(z['p'] for z in alts if k(z['token']) and k(z['token'])==k(tok) and z['token']!=tok)
            if mode.get('order') and nm.startswith('party') and j==0 and alts:
                others=[w for n2,(_,_,w) in vals.items() if n2.startswith('party') and n2!=nm]
                p+=sum(z['p'] for z in alts[:1] if z['token'].strip() and any(o.startswith(z['token'].strip()) for o in others))
            p=min(1.0,max(p,1e-9))
            lp+=math.log(p); mn=min(mn,p)
        out[nm]=(mn, math.exp(lp), math.exp(lp/max(1,len(toks))))
    return out
MODES={'min (today)':('min',{}),'whole value':('prod',{}),'whole value, formatting merged':('prod',{'fmt':1,'case':1}),
       'whole value, formatting+order merged':('prod',{'fmt':1,'case':1,'order':1}),'per-token geo mean':('geo',{}),
       'min, formatting merged':('min',{'fmt':1,'case':1})}
rows=[]
for s,dirn in sets.items():
    gold={d['id']:d for d in json.load(open(f'{D}/gold-{s}.json'))['documents']}
    rec={d['id']:d for d in json.load(open(f'{D}/{dirn}/recording.json'))['documents']}
    for r in json.load(open(f'{D}/g0-{s}.json'))['records']:
        if r['readiness']!='ready': continue
        rr=dict(r); rr['_gold']=gold.get(r['id'],{}).get('gold',{}); rr['_goldmeta']=gold.get(r['id'],{})
        c=sev.classify(rr)[0]
        tr=rec[r['id']]['exchanges'][-1]['reply']['token_trace']
        sc={}
        for name,(agg,mode) in MODES.items():
            o=score(tr,mode)
            idx={'min':0,'prod':1,'geo':2}[agg]
            sc[name]=min([v[idx] for v in o.values()],default=1.0)
        rows.append({'set':s,'id':r['id'],'sev':c,'tc':(r.get('token_confidence') or {}).get('min'),'sc':sc,'name':r['filename']})
json.dump(rows,open(f'{D}/wv-rows.json','w'))
# sanity: our min equals engine's
bad=[r for r in rows if r['tc'] is not None and abs(r['tc']-r['sc']['min (today)'])>1e-3]
print('pool',len(rows),collections.Counter(r['sev'] for r in rows),'min mismatches',len(bad))
for b in bad[:5]: print('  mismatch',b['set'],b['id'],b['tc'],b['sc']['min (today)'])
