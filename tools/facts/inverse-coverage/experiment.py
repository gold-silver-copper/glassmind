#!/usr/bin/env python3
"""Bounded prospective WDQS acquisition and hash-bound offline graph merge.

No evaluation inputs, labels, annotations beyond TRAIN subject/relation selection,
or runtime changes. Fetch is one-shot; replay cannot issue network requests.
"""
import argparse
from collections import Counter
from datetime import datetime, timezone
import gzip
import hashlib
import json
from pathlib import Path
import re
import signal
import time
import urllib.error
import urllib.parse
import urllib.request

BASE = Path('/home/gsc/.cache/glassmind/continuous-20261010/trainindex-run/tmp/trainindex')
DATA = Path('/home/gsc/.cache/glassmind/data/simplequestions')
RUN = Path('/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp/inverse-coverage')
OWN = Path(__file__).resolve().parent
ENDPOINT = 'https://query.wikidata.org/sparql'
UA = 'GlassmindInverseCoverage/0.1 (https://github.com/gold-silver-copper/glassmind; bounded TRAIN graph research)'
MAX_BYTES = 20 * 1024 * 1024
SELECTION_SHA = 'e83ebf84d3574b2fcea42254bfb65fc4077a5d6250e9317b65f35c8933af0a45'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def identity(p):
    raw = p.read_bytes()
    return dict(path=str(p.resolve()), bytes=len(raw), sha256=sha(raw))


def write(p, obj):
    p.write_text(json.dumps(obj, indent=2, sort_keys=True)+'\n')


def checked(p, record):
    raw = p.read_bytes()
    if sha(raw) != record['sha256'] or len(raw) != record['bytes']:
        raise ValueError(f'provenance mismatch: {p}')
    return raw


def utc():
    return datetime.now(timezone.utc).isoformat()


def selection():
    raw = (OWN/'selection.json').read_bytes()
    if sha(raw) != SELECTION_SHA or (RUN/'selection.json').read_bytes() != raw:
        raise ValueError('frozen selection changed')
    return json.loads(raw)


def selected_pairs(train_bytes, base):
    rows = train_bytes.splitlines()
    counts = Counter()
    for i, line in enumerate(rows[:len(rows)*9//10]):
        if i % 10 == 0:
            continue
        subject, relation, _ = line.split(b'\t', 2)
        subject, relation = subject.decode(), relation.decode()
        if subject in base['entities'] and re.fullmatch(r'Q[0-9]+', subject) and re.fullmatch(r'R[0-9]+', relation):
            counts[subject, relation] += 1
    pairs = [dict(subject=s, relation=r, count=n, tie_sha256=sha((s+'\t'+r).encode())) for (s,r),n in counts.items()]
    return sorted(pairs, key=lambda p:(-p['count'], p['tie_sha256']))[:100]


def verify_base():
    """Check all reused inputs against prior ledgers, without DEV access."""
    prior = json.loads((OWN.parent/'trainindex/reports/evidence.json').read_bytes())
    records = {r['path']:r for r in prior['artifacts']}
    checked_files = {}
    def verify(p, record=None):
        if record is None:
            record = records[str(p)]
        checked(p, record)
        checked_files[str(p)] = identity(p)
    for name in ('selection.json','fetch.json','rebuild.json','label-cache.json',
                 'scope1000/index.json','scope1000/labels.json',
                 'bundle1000/index.json','bundle1000/labels.json',
                 'bundle1000/manifest.json','bundle1000/relation.json'):
        verify(BASE/name)
    rebuild = json.loads((BASE/'rebuild.json').read_bytes())
    fetch = json.loads((BASE/'fetch.json').read_bytes())
    sources = rebuild['sources'] + fetch['reused_sources'] + fetch['requests'] + json.loads((BASE/'label-cache.json').read_bytes())
    for record in sources:
        if 'path' in record:
            verify(Path(record['path']), record)
    sel = selection()
    raw = checked(DATA/'train.tsv', dict(sha256=sel['train_sha256'], bytes=sel['train_bytes']))
    verify(DATA/'train.tsv', dict(sha256=sel['train_sha256'], bytes=sel['train_bytes']))
    base = json.loads((BASE/'scope1000/index.json').read_bytes())
    if selected_pairs(raw, base) != sel['pairs']:
        raise ValueError('selection recomputation differs')
    verify(DATA/'relation.json', dict(sha256='db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6', bytes=35773124))
    report = dict(source='b569ac0eb0bab5a8b9d39cda904c816e77c08e63', files=list(checked_files.values()), selection=identity(OWN/'selection.json'), base_entities=len(base['entities']))
    write(RUN/'base-verification.json', report)
    print(f'verified {len(checked_files)} reused artifacts; selection matches TRAIN')


def query(pair):
    target, relation = pair['subject'], pair['relation']
    if not re.fullmatch(r'Q[0-9]+', target) or not re.fullmatch(r'R[0-9]+', relation):
        raise ValueError('invalid pair')
    prop = 'P'+relation[1:]
    return f'''PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX p: <http://www.wikidata.org/prop/>
PREFIX ps: <http://www.wikidata.org/prop/statement/>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX schema: <http://schema.org/>
SELECT ?subject (SAMPLE(?st) AS ?statement) (SAMPLE(?r) AS ?rank)
       (SAMPLE(?rev) AS ?revision) (wd:{prop} AS ?property) (wd:{target} AS ?target)
WHERE {{
  {{ SELECT DISTINCT ?subject WHERE {{
    ?subject p:{prop} ?s . ?s ps:{prop} wd:{target}; wikibase:rank ?rank0 .
    VALUES ?rank0 {{ wikibase:NormalRank wikibase:PreferredRank }}
    FILTER(REGEX(STR(?subject), "^http://www.wikidata.org/entity/Q[0-9]+$"))
  }} ORDER BY ?subject LIMIT 101 }}
  ?subject p:{prop} ?st . ?st ps:{prop} wd:{target}; wikibase:rank ?r .
  VALUES ?r {{ wikibase:NormalRank wikibase:PreferredRank }}
  BIND(IRI(REPLACE(STR(?subject), "/entity/", "/wiki/Special:EntityData/")) AS ?data)
  OPTIONAL {{ ?data schema:version ?rev . }}
}}
GROUP BY ?subject
ORDER BY ?subject
LIMIT 101
'''


def parse_response(raw, pair):
    doc = json.loads(raw)
    if not isinstance(doc, dict) or 'error' in doc or 'subject' not in doc.get('head', {}).get('vars', []):
        raise ValueError('not a subject SPARQL result')
    rows = doc['results']['bindings']
    if not isinstance(rows, list) or len(rows) > 101:
        raise ValueError('invalid result count')
    subjects, revisions = [], {}
    def uri(row, key):
        cell = row[key]
        if cell.get('type') != 'uri':
            raise ValueError(f'non-URI {key}')
        return cell['value']
    for row in rows:
        subject = uri(row, 'subject')
        if not re.fullmatch(r'http://www.wikidata.org/entity/Q[0-9]+', subject):
            raise ValueError('non-item subject')
        qid = subject.rsplit('/',1)[1]
        if uri(row,'target') != 'http://www.wikidata.org/entity/'+pair['subject'] or uri(row,'property') != 'http://www.wikidata.org/entity/P'+pair['relation'][1:]:
            raise ValueError('independent edge does not match requested pair')
        if uri(row,'rank') not in ('http://wikiba.se/ontology#NormalRank','http://wikiba.se/ontology#PreferredRank'):
            raise ValueError('deprecated or unknown rank')
        if not uri(row,'statement').lower().startswith('http://www.wikidata.org/entity/statement/'+qid.lower()+'-'):
            raise ValueError('statement belongs to another subject')
        revision = row.get('revision',{}).get('value')
        if revision is not None and not re.fullmatch('[0-9]+', revision):
            raise ValueError('invalid revision')
        subjects.append(qid)
        revisions[qid] = revision
    if len(subjects) != len(set(subjects)):
        raise ValueError('subjects must be distinct')
    # SPARQL ORDER BY on IRIs follows endpoint term ordering, which need not
    # match Python lexical QID order (e.g. inline WDQS IDs). Keep raw order.
    return dict(status='truncated' if len(subjects)==101 else 'complete_endpoint_snapshot', subjects=subjects, revisions=revisions)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise urllib.error.HTTPError(req.full_url, code, 'redirect refused: primary endpoint only', headers, fp)


def fetch(resume_local_validation=False):
    sel = selection()
    path = RUN/'fetch.json'
    if path.exists() and not resume_local_validation:
        raise ValueError('one-shot network ledger already exists; offline replay only')
    ledger = dict(selection_sha256=SELECTION_SHA, endpoint=ENDPOINT, user_agent=UA, license='CC0-1.0', started_utc=utc(), requests=[], queries=[dict(subject=p['subject'],relation=p['relation'],status='not_requested') for p in sel['pairs']], new_decompressed_bytes=0, stop_reason=None)
    offset=0
    elapsed_before=0
    if resume_local_validation:
        ledger=json.loads(path.read_bytes())
        if ledger['stop_reason'] not in ('first failure: statement belongs to another subject', 'first failure: subjects must be distinct and ORDER BY subject'):
            raise ValueError('resume permitted only for documented local parser errors')
        write(RUN/f"fetch-before-local-parser-fix-{len(ledger['requests'])}.json",ledger)
        elapsed_before=time.time()-datetime.fromisoformat(ledger['started_utc']).timestamp()
        if elapsed_before>=600:raise ValueError('original network budget expired')
        for record in ledger['requests']:
            pair=sel['pairs'][record['pair_index']]
            raw=checked(Path(record['response']['path']),record['response'])
            parsed=parse_response(raw,pair)
            ledger['queries'][record['pair_index']].update(parsed)
            if not record.get('success'):
                record['original_validation_error']=record.pop('error')
                record['success']=True
        offset=len(ledger['requests']);ledger['stop_reason']=None
        ledger['local_parser_correction_utc']=utc()
    write(path,ledger)
    (RUN/'raw').mkdir(exist_ok=True)
    (RUN/'queries').mkdir(exist_ok=True)
    start = time.monotonic()-elapsed_before; last = time.monotonic()-5
    def deadline(signum, frame):
        raise TimeoutError('network phase 600-second deadline')
    previous = signal.signal(signal.SIGALRM, deadline)
    signal.setitimer(signal.ITIMER_REAL, 600-elapsed_before)
    opener = urllib.request.build_opener(NoRedirect())
    try:
        for i,pair in list(enumerate(sel['pairs']))[offset:]:
            delay = max(0, 5-(time.monotonic()-last))
            if time.monotonic()-start+delay >= 599 or len(ledger['requests'])>=110 or ledger['new_decompressed_bytes']>=MAX_BYTES:
                ledger['stop_reason']='budget';break
            time.sleep(delay)
            q = query(pair); qp=RUN/'queries'/f'{i:03}.rq';qp.write_text(q)
            url=ENDPOINT+'?'+urllib.parse.urlencode(dict(query=q,format='json'))
            record=dict(pair_index=i,subject=pair['subject'],relation=pair['relation'],query=identity(qp),query_text=q,url=url,started_utc=utc(),start_elapsed_seconds=time.monotonic()-start)
            ledger['requests'].append(record); ledger['queries'][i]['status']='missing';write(path,ledger)
            last=time.monotonic()
            try:
                req=urllib.request.Request(url,headers={'User-Agent':UA,'Accept-Encoding':'gzip','Accept':'application/sparql-results+json'})
                try:
                    response=opener.open(req,timeout=min(45,600-(time.monotonic()-start)))
                except urllib.error.HTTPError as error:
                    response=error
                with response:
                    record['http_status']=response.code
                    record['response_headers']={k:v for k,v in response.headers.items() if k.lower() in ('date','content-type','content-encoding','retry-after','server','x-cache','x-served-by')}
                    stream=gzip.GzipFile(fileobj=response) if response.headers.get('Content-Encoding')=='gzip' else response
                    remaining=MAX_BYTES-ledger['new_decompressed_bytes']
                    raw=stream.read(remaining)
                ledger['new_decompressed_bytes']+=len(raw)
                rp=RUN/'raw'/(sha(raw)+'.body');rp.write_bytes(raw);record['response']=identity(rp)
                if len(raw)>=remaining:
                    raise ValueError('decompressed byte budget exhausted; response may be partial')
                if record['http_status'] != 200:
                    raise ValueError(f"HTTP {record['http_status']}; no retry or alternate endpoint")
                result=parse_response(raw,pair)
                ledger['queries'][i].update(result)
                record['success']=True
            except (OSError,ValueError,KeyError,TypeError,EOFError) as error:
                record['error']=str(error); record['success']=False
                ledger['stop_reason']='first failure: '+str(error)
            record['finished_utc']=utc();write(path,ledger)
            print(f"request {i+1}: {ledger['queries'][i]['status']} HTTP {record.get('http_status')}",flush=True)
            if ledger['stop_reason']:
                break  # Stopping entirely also honors any Retry-After; never retries.
    except TimeoutError as error:
        ledger['stop_reason']=str(error)
    finally:
        signal.setitimer(signal.ITIMER_REAL,0);signal.signal(signal.SIGALRM,previous)
        ledger.update(finished_utc=utc(),elapsed_seconds=time.monotonic()-start)
        write(path,ledger)
    print(json.dumps({k:ledger[k] for k in ('stop_reason','new_decompressed_bytes','elapsed_seconds')}))


def merge_bytes(base_raw, additions):
    """Insert only new JSON tokens; all original bytes remain, in original order.

    json raw_decode gives spans, avoiding reserialization of aliases/claims.
    Existing property arrays are extended; untouched entities remain byte-identical.
    """
    text=base_raw.decode(); decoder=json.JSONDecoder()
    def members(start):
        pos=start+1; result={}
        while True:
            while text[pos].isspace() or text[pos]==',':pos+=1
            if text[pos]=='}':return result,pos
            key,end=decoder.raw_decode(text,pos);pos=end
            while text[pos].isspace() or text[pos]==':':pos+=1
            value,end=decoder.raw_decode(text,pos)
            result[key]=(pos,end,value);pos=end
    root,_=members(text.index('{')); entities_start=root['entities'][0]
    entities,entities_end=members(entities_start); edits=[]; new_entities={}
    for subject,props in sorted(additions.items()):
        if not re.fullmatch(r'Q[0-9]+',subject):raise ValueError('invalid subject')
        for prop,values in props.items():
            if not re.fullmatch(r'P[0-9]+',prop) or any(not re.fullmatch(r'Q[0-9]+',v) for v in values):raise ValueError('invalid edge')
        if subject not in entities:
            new_entities[subject]=dict(aliases=[],claims={p:sorted(set(v)) for p,v in sorted(props.items())})
            continue
        fields,_=members(entities[subject][0]);claims,claims_end=members(fields['claims'][0]);new_props={}
        for prop,values in sorted(props.items()):
            if prop not in claims:
                new_props[prop]=sorted(set(values));continue
            start,end,old=claims[prop]; extra=sorted(set(values)-set(old))
            if extra:edits.append((end-1, (', ' if old else '')+', '.join(json.dumps(v) for v in extra)))
        if new_props:
            edits.append((claims_end, (', ' if claims else '')+json.dumps(new_props,sort_keys=True)[1:-1]))
    if new_entities:
        edits.append((entities_end, (', ' if entities else '')+json.dumps(new_entities,sort_keys=True)[1:-1]))
    for pos,insert in sorted(edits,reverse=True):text=text[:pos]+insert+text[pos:]
    result=text.encode();new=json.loads(result)['entities'];old=json.loads(base_raw)['entities']
    for q,e in old.items():
        if new[q]['aliases']!=e['aliases']:raise ValueError('alias mutation')
        for p,v in e['claims'].items():
            if new[q]['claims'][p][:len(v)]!=v:raise ValueError('base claim mutation')
    return result


def rebuild():
    verify_base(); sel=selection();ledger=json.loads((RUN/'fetch.json').read_bytes())
    if ledger['selection_sha256']!=SELECTION_SHA or ledger['endpoint']!=ENDPOINT:raise ValueError('fetch binding differs')
    additions={};provenance=[];statuses=Counter()
    seen=set()
    for record in ledger['requests']:
        i=record['pair_index']
        if i in seen:raise ValueError('duplicate pair record')
        seen.add(i);pair=sel['pairs'][i]
        qraw=checked(Path(record['query']['path']),record['query'])
        if qraw.decode()!=query(pair) or record['query_text']!=qraw.decode():raise ValueError('query mismatch')
        raw=checked(Path(record['response']['path']),record['response']) if 'response' in record else None
        if not record.get('success'):
            statuses['missing']+=1;continue
        if record.get('http_status')!=200 or raw is None:raise ValueError('unwitnessed response')
        parsed=parse_response(raw,pair)
        if any(ledger['queries'][i][k]!=v for k,v in parsed.items()):raise ValueError('response summary mismatch')
        statuses[parsed['status']]+=1
        for subject in parsed['subjects']:
            prop='P'+pair['relation'][1:];target=pair['subject']
            additions.setdefault(subject,{}).setdefault(prop,set()).add(target)
            provenance.append(dict(subject=subject,property=prop,target=target,pair_index=i,response_sha256=record['response']['sha256'],revision=parsed['revisions'][subject],query_status=parsed['status']))
    statuses['not_requested']=len(sel['pairs'])-len(seen)
    base_raw=(BASE/'scope1000/index.json').read_bytes()
    candidate=merge_bytes(base_raw,additions)
    (RUN/'candidate-index.json').write_bytes(candidate)
    report=dict(statuses=dict(statuses),independent_edges=len(provenance),new_nodes=len(set(additions)-set(json.loads(base_raw)['entities'])),index=identity(RUN/'candidate-index.json'),base=identity(BASE/'scope1000/index.json'),provenance=provenance,decision='candidate awaiting frozen DEV evaluation; acquisition status reported separately',fetch=identity(RUN/'fetch.json'))
    write(RUN/'merge.json',report);print(json.dumps({k:v for k,v in report.items() if k!='provenance'}))


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('command',choices=('verify','fetch','resume-local-validation','rebuild'))
    command=parser.parse_args().command
    {'verify':verify_base,'fetch':fetch,'resume-local-validation':lambda:fetch(True),'rebuild':rebuild}[command]()


if __name__=='__main__':main()
