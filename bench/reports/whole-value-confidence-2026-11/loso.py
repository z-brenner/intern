import os,json,sys,collections
rows=json.load(open(sys.argv[1]))
sets=sorted({r['set'] for r in rows})
def tally(fs): c=collections.Counter(r['sev'] for r in fs); return [c['right'],c['minor'],c['wrong']]
def at(rs,m,t): return [r for r in rs if r['sc'][m]>=t]
WV='whole value, formatting merged'
for m in [WV,'whole value','min, formatting merged','whole value, formatting+order merged']:
    tot=[0,0,0]; base=[0,0,0]; per=[]
    for h in sets:
        train=[r for r in rows if r['set']!=h]; test=[r for r in rows if r['set']==h]
        budget=tally(at(train,'min (today)',0.7))[2]
        best=None
        for t in sorted({r['sc'][m] for r in train}):
            rg,mn,wr=tally(at(train,m,t))
            if wr<=budget and (best is None or rg>best[0]): best=(rg,t)
        t=best[1]
        a=tally(at(test,m,t)); b=tally(at(test,'min (today)',0.7))
        per.append((h,round(t,3),a,b)); tot=[x+y for x,y in zip(tot,a)]; base=[x+y for x,y in zip(base,b)]
    print(f'{m}: leave-one-set-out total right/minor/wrong {tot}  vs today@0.7 {base}')
    if m==WV:
        for p in per: print('   ',p[0],'t=',p[1],'new',p[2],'today',p[3])
# which wrong names each files at matched K=2 in-sample
for m,t in [('min (today)',0.7),(WV,0.594)]:
    print(m,'wrong filed:',[(r['set'],r['id'],round(r['sc'][m],3)) for r in at(rows,m,t) if r['sev']=='wrong'])
