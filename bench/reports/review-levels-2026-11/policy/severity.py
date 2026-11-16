"""Severity of a proposed name against its gold: 'right', 'minor', 'wrong' (really wrong).
Applied by script; the rules are in classify() and were checked by reading names by hand (see REPORT)."""
import re, unicodedata
DATE=re.compile(r'^(\d{4}-\d{2}-\d{2})\b')
SUFFIX={'inc','llc','ltd','limited','corp','corporation','co','company','plc','lp','llp','the','of','and','for','gmbh','sa','ag','pty','nv','bv'}
GENERIC_TYPES={'document','file','record','untitled','scan','page'}
# the head nouns that say what kind of record something is; two types sharing one are the same kind
KIND_WORDS={'letter','notice','memo','memorandum','invoice','minutes','agenda','agreement','contract','form','statement',
 'report','order','email','receipt','bill','certificate','policy','resolution','consent','application','lease','amendment',
 'addendum','schedule','summary','table','list','template','bulletin','newsletter','press','release','announcement','warning',
 'determination','decision','opinion','complaint','petition','motion','response','reply','comments','comment','recommendation',
 'recommendations','guidance','circular','advisory','alert','declaration','affidavit','plan','proposal','quote','quotation',
 'estimate','purchase','voucher','remittance','transcript','presentation','register','questionnaire','survey','checklist',
 'license','licence','permit','deed','waiver','authorization','authorisation','request','submission','filing','rule','citation','violation'}
SAME_KIND=[{'memo','memorandum'},{'agreement','contract'},{'invoice','bill'},{'comment','comments'},{'recommendation','recommendations'},
 {'license','licence'},{'authorization','authorisation'},{'notice','announcement'},{'quote','quotation','estimate'}]
def norm(s):
    s=unicodedata.normalize('NFKD',s or '').encode('ascii','ignore').decode().lower()
    s=s.replace('&',' and ')
    return re.sub(r'[^a-z0-9]+',' ',s).strip()
def toks(s, drop=SUFFIX):
    return [t for t in norm(s).split() if t not in drop]
def canon_kind(t):
    for grp in SAME_KIND:
        if t in grp: return sorted(grp)[0]
    return t
def kinds(s):
    return {canon_kind(t) for t in norm(s).split() if t in KIND_WORDS}
PREP={'of','for','to','on','regarding','re','in','from','by','with','under','concerning','about','at'}
def head(t):
    """the noun a type is: the last word before its first preposition ("Notice of Appeal" -> notice,
    "Invoice Dispute Letter" -> letter, "Request for Corrected Invoice" -> request)."""
    ws=norm(t).split()
    cut=[]
    for w in ws:
        if w in PREP and cut: break
        cut.append(w)
    h=(cut or ws or [''])[-1]
    if h.endswith('s') and h not in ('minutes','comments','recommendations','press') and len(h)>3: h=h[:-1]
    return canon_kind(h)
ROLE_WORDS={'all','regional','resident','directors','director','officers','officer','in','charge','chair','chairs','board','ceo','ceos',
 'secretary','home','deans','academic','members','staff','employees','managers','heads','and','of','the','to','chief','executives','executive'}
def is_class_or_office(p):
    """an addressee given as a class or an office, not a name: "Regional Directors, Officers-in-Charge, and Resident Officers", "CEO"."""
    ws=norm(p).split()
    return bool(ws) and all(w in ROLE_WORDS for w in ws)
PERSON_WHY=re.compile(r"\b(signs?|signed|signer|signatory|addressed|addressees?|sender)\b",re.I)
NOT_PERSON_WHY=re.compile(r"not the sender|pre-filled",re.I)
def near_spelling(a,b):
    import difflib
    x,y=norm(a).replace(' ',''),norm(b).replace(' ','')
    return len(x)>=5 and len(y)>=5 and difflib.SequenceMatcher(None,x,y).ratio()>=0.8
def party_match(a,b):
    """loose: same after dropping suffixes/articles, or one's tokens contained in the other's (>=1 distinctive token), or acronym."""
    ta,tb=toks(a),toks(b)
    if not ta or not tb: return False
    if ta==tb: return True
    sa,sb=set(ta),set(tb)
    if sa<=sb or sb<=sa: return True
    ac=lambda t:''.join(w[0] for w in t)
    na,nb=norm(a).replace(' ',''),norm(b).replace(' ','')
    if len(ta)>=2 and ac(ta)==nb: return True
    if len(tb)>=2 and ac(tb)==na: return True
    inter=sa&sb
    return len(inter)>=2 and len(inter)/min(len(sa),len(sb))>=0.67
def gold_names(g):
    names=list(g.get('parties') or [])
    for s in g.get('acceptable_party_sets') or []: names+=s.get('parties') or []
    names+=[r['name'] for r in g.get('party_roles') or [] if r.get('name')]
    return names
def classify(r):
    """returns (severity, [why...]). r is a bench record with _gold attached."""
    g=r.get('_gold') or {}; sc=r.get('scores') or {}; why=[]
    name=r.get('filename') or ''
    if r.get('status')!='completed' or not name.strip():
        return 'wrong',['nothing: not read or no name']
    if sc.get('filename_correct') is True:
        return 'right',[]
    stem=re.sub(r'\.[A-Za-z0-9]{2,5}$','',name)
    m=DATE.match(stem)
    gdates=[d for d in [g.get('document_date')]+list(g.get('acceptable_dates') or []) if d]
    # date
    if m:
        if m.group(1) not in gdates:
            why.append('date: not the document\'s own' + (' (gold: undated)' if not gdates else '') +
                       (' (a trap date)' if any(f.get('date')==m.group(1) for f in g.get('forbidden_dates') or []) else ''))
    else:
        why.append('nodate: name has no date (cannot be filed: DATE_REQUIRED)')
    # parties actually shown in the name
    prop=r.get('proposal') or {}
    shown=[p for p in prop.get('parties') or [] if norm(p) and norm(p) in norm(stem)]
    good=gold_names(g); bad=[f['name'] for f in g.get('forbidden_parties') or [] if f.get('name')]
    forb=[f for f in g.get('forbidden_parties') or [] if f.get('name')]
    for p in shown:
        hit=[f for f in forb if party_match(p,f['name'])]
        # repaired per the checker: a forbidden party is judged by its forbidden entry first; a name the gold
        # lists among parties / acceptable sets (not merely party_roles) still passes
        strong=list(g.get('parties') or [])
        for s_ in g.get('acceptable_party_sets') or []: strong+=s_.get('parties') or []
        if not hit and any(party_match(p,x) for x in good): continue
        if hit and any(norm(p)==norm(x) for x in strong): continue
        if hit:
            w=hit[0].get('why') or ''
            if PERSON_WHY.search(w) and not NOT_PERSON_WHY.search(w): why.append(f'person: "{p}" is the individual who signs or is addressed ({w})')
            else: why.append(f'party: not a party "{p}" ({w})')
        elif any(near_spelling(p,x) for x in good): why.append(f'spelling: "{p}" misspells a party')
        elif is_class_or_office(p): why.append(f'class: "{p}" is a class or office of addressee')
        else: why.append(f'party: not a gold party "{p}"')
    # type
    ptype=prop.get('document_type') or ''
    gtypes=[t for t in [g.get('document_type')]+list(g.get('acceptable_types') or []) if t]
    if sc.get('type_correct') is False and ptype:
        gh={head(t) for t in gtypes}
        if set(norm(ptype).split())<=GENERIC_TYPES: why.append('generic: type is "Document"')   # says nothing, misleads nobody
        elif head(ptype) in gh: why.append(f'qualifier: "{ptype}" for "{gtypes[0]}"')
        else: why.append(f'type: "{ptype}" misdescribes a {gtypes[0] if gtypes else "?"}')
    elif not ptype and 'Document' in stem.split(): why.append('generic: type is "Document"')
    hard=[w for w in why if w.split(':')[0] in ('date','party','type','nodate')]
    if hard: return 'wrong',why
    return 'minor',why
