"""Public authored synthetic development fixtures; no benchmark answer data."""
import copy
import json
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch
import contextlib
import io
import experiment as ex


def response(subjects, target='Q1', prop='P2', rank='NormalRank'):
    bindings=[]
    for q in sorted(subjects):
        uri=lambda value:dict(type='uri',value=value)
        bindings.append(dict(subject=uri('http://www.wikidata.org/entity/'+q),
            property=uri('http://www.wikidata.org/entity/'+prop),
            target=uri('http://www.wikidata.org/entity/'+target),
            statement=uri('http://www.wikidata.org/entity/statement/'+q+'-synthetic'),
            rank=uri('http://wikiba.se/ontology#'+rank)))
    return json.dumps(dict(head=dict(vars=['subject','property','target','statement','rank']),results=dict(bindings=bindings))).encode()


class ExperimentTests(unittest.TestCase):
    pair=dict(subject='Q1',relation='R2')

    def test_empty_truncated_missing_distinct(self):
        empty=ex.parse_response(response([]),self.pair)
        self.assertEqual(empty['status'],'complete_endpoint_snapshot')
        full=ex.parse_response(response(['Q'+str(n) for n in range(101)]),self.pair)
        self.assertEqual(full['status'],'truncated')
        self.assertEqual(len(full['subjects']),101)
        unordered=json.loads(response(['Q30','Q100']))
        unordered['results']['bindings'].reverse()
        self.assertEqual(len(ex.parse_response(json.dumps(unordered).encode(),self.pair)['subjects']),2)
        for raw in (b'',b'{}',b'{"error":"timeout"}',response(['Q3','Q3'])):
            with self.assertRaises((ValueError,KeyError)):
                ex.parse_response(raw,self.pair)

    def test_independent_edges_and_rank(self):
        for rank in ('NormalRank','PreferredRank'):
            self.assertEqual(ex.parse_response(response(['Q3'],rank=rank),self.pair)['subjects'],['Q3'])
        for kwargs in (dict(target='Q9'),dict(prop='P3'),dict(rank='DeprecatedRank')):
            with self.assertRaises(ValueError):ex.parse_response(response(['Q3'],**kwargs),self.pair)
        lower=response(['Q3']).replace(b'/statement/Q3-',b'/statement/q3-')
        self.assertEqual(ex.parse_response(lower,self.pair)['subjects'],['Q3'])
        raw=json.loads(response(['Q3']))
        raw['results']['bindings'][0]['statement']['value']='http://www.wikidata.org/entity/statement/Q9-other'
        with self.assertRaises(ValueError):ex.parse_response(json.dumps(raw).encode(),self.pair)
        q=ex.query(self.pair)
        self.assertIn('SELECT DISTINCT ?subject',q)
        self.assertIn('wikibase:NormalRank wikibase:PreferredRank',q)
        self.assertNotIn('wdt:',q)
        self.assertIn('ORDER BY ?subject\nLIMIT 101',q)

    def test_exact_bytes_preserved_and_aliasless(self):
        raw=b'{ "entities" : { "Q1" : {"aliases": ["\\u00e9", "one"], "claims": {"P2" : [ "Q8" ]}}, "Q2":{"aliases":[],"claims":{}} } }\n'
        added={'Q1':{'P2':{'Q1','Q8'},'P3':{'Q4'}},'Q9':{'P2':{'Q1'}},'Q2':{'P3':{'Q4'}}}
        merged=ex.merge_bytes(raw,added)
        self.assertIn(b'"aliases": ["\\u00e9", "one"]',merged)
        self.assertIn(b'"P2" : [ "Q8" , "Q1"]',merged)
        self.assertEqual(json.loads(merged)['entities']['Q9']['aliases'],[])
        self.assertEqual(json.loads(merged)['entities']['Q1']['claims']['P2'],['Q8','Q1'])
        self.assertEqual(ex.merge_bytes(raw,{}),raw)
        self.assertEqual(ex.merge_bytes(merged,added),merged)
        # Removing ONLY inserted bytes reconstructs the exact source sequence.
        it=iter(merged)
        self.assertTrue(all(any(c==b for c in it) for b in raw))
        self.assertEqual(ex.merge_bytes(raw,dict(reversed(list(added.items())))),merged)

    def test_provenance_tampering(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'response.json';p.write_bytes(response(['Q3']))
            record=ex.identity(p);self.assertEqual(ex.checked(p,record),p.read_bytes())
            p.write_bytes(response(['Q4']))
            with self.assertRaisesRegex(ValueError,'provenance mismatch'):ex.checked(p,record)

    def test_selection_ignores_answers_questions_and_exclusions(self):
        rows=[b'Q1\tR2\tPRIVATE-IGNORED\tignored']*20
        base={'entities':{'Q1':{}}}
        chosen=ex.selected_pairs(b'\n'.join(rows),base)
        self.assertEqual(chosen[0]['count'],16)
        rows[0]=b'Q9\tR9\tunused\tunused';rows[10]=rows[0];rows[18]=rows[0];rows[19]=rows[0]
        rows[1]=b'Q1\tR2\tDIFFERENT\tDIFFERENT'
        self.assertEqual(ex.selected_pairs(b'\n'.join(rows),base),chosen)
        self.assertEqual(ex.selected_pairs(b'Q1\tP2\tx\tx\n'*20,base),[])

    def test_rebuild_uses_raw_witnesses_and_keeps_missing_status(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);base=root/'base';run=root/'run'
            (base/'scope1000').mkdir(parents=True);run.mkdir()
            (base/'scope1000/index.json').write_bytes(b'{"entities":{"Q1":{"aliases":["one"],"claims":{}}}}')
            pairs=[self.pair,dict(subject='Q2',relation='R3')]
            raw=response(['Q'+str(n) for n in range(1000,1101)])
            rp=run/'response.json';rp.write_bytes(raw)
            qp=run/'query.rq';qp.write_text(ex.query(self.pair))
            qp2=run/'missing.rq';qp2.write_text(ex.query(pairs[1]))
            parsed=ex.parse_response(raw,self.pair)
            record=dict(pair_index=0,success=True,http_status=200,query=ex.identity(qp),query_text=qp.read_text(),response=ex.identity(rp))
            missing=dict(pair_index=1,success=False,query=ex.identity(qp2),query_text=qp2.read_text())
            ledger=dict(selection_sha256=ex.SELECTION_SHA,endpoint=ex.ENDPOINT,requests=[record,missing],queries=[parsed,dict(status='missing')],stop_reason='synthetic timeout')
            ex.write(run/'fetch.json',ledger)
            with patch.object(ex,'RUN',run),patch.object(ex,'BASE',base),patch.object(ex,'verify_base'),patch.object(ex,'selection',return_value=dict(pairs=pairs)),contextlib.redirect_stdout(io.StringIO()):
                ex.rebuild();first=(run/'candidate-index.json').read_bytes();ex.rebuild()
                self.assertEqual((run/'candidate-index.json').read_bytes(),first)
                report=json.loads((run/'merge.json').read_bytes())
                self.assertEqual(report['statuses'],dict(truncated=1,missing=1,not_requested=0))
                self.assertEqual(report['independent_edges'],101)
                self.assertTrue(all(e['aliases']==[] for q,e in json.loads(first)['entities'].items() if q!='Q1'))
                rp.write_bytes(response(['Q2000']))
                with self.assertRaisesRegex(ValueError,'provenance mismatch'):ex.rebuild()

    def test_missing_rebuild_never_creates_edges(self):
        # Real blocked run replay is exercised separately; no synthetic success
        # is mixed into its acquisition ledger.
        for raw in (b'{"results":{"bindings":[]}}',b'Forbidden'):
            with self.assertRaises((ValueError,KeyError)):ex.parse_response(raw,self.pair)
        base=b'{"entities":{"Q1":{"aliases":["one"],"claims":{}}}}'
        self.assertEqual(ex.merge_bytes(base,{}),base)


if __name__=='__main__':unittest.main()
