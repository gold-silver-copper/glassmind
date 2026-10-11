"""Small offline tests of scientific selection/extraction boundaries."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('prepare', Path(__file__).with_name('prepare.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

class Preparation(unittest.TestCase):
    def test_selection_ignores_calibration_tail_and_non_subject_columns(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root/'train.tsv'
            rows = [f'Q{i+1}\tP1\tQ8\tquestion' for i in range(2000)]
            path.write_text('\n'.join(rows)+'\n')
            m.select(path,root)
            a = json.loads((root/'selection.json').read_text())
            expected = sorted((f'Q{i+1}' for i in range(1800) if i%10),key=lambda q:m.sha(q.encode()))[:1000]
            self.assertEqual([r['qid'] for r in a['entities']], expected)
            self.assertEqual(a['fit_rows'],1620)
            self.assertFalse(set(expected)&{f'Q{i+1}' for i in range(2000) if i%10==0 or i>=1800})
            other = root/'other'
            other.mkdir()
            rows = [('Q999999' if i%10==0 or i>=1800 else row.split('\t')[0])+'\tP999\tQ777\tchanged' for i,row in enumerate(rows)]
            path.write_text('\n'.join(rows)+'\n')
            m.select(path,other)
            self.assertEqual(a['entities'],json.loads((other/'selection.json').read_text())['entities'])

    def test_extraction_preserves_rank_and_qualifier_semantics(self):
        def claim(q,rank):
            return dict(rank=rank, qualifiers={'P580':[{}]}, mainsnak={'datavalue':{'type':'wikibase-entityid','value':{'id':q}}})
        entity = dict(labels={'en':{'value':'Name'}},aliases={'en':[{'value':'Alias'},{'value':'Name'}]},claims={'P1':[claim('Q1','normal'),claim('Q2','preferred'),claim('Q3','deprecated'),claim('P4','normal'),{'mainsnak':{'snaktype':'novalue'}}]})
        self.assertEqual(m.extract(entity),{'aliases':['Alias','Name'],'claims':{'P1':['Q1','Q2']}})

    def test_rebuild_freezes_cache_membership_and_checks_raw_hashes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); cache=root/'cache'; out=root/'out'
            cache.mkdir(); out.mkdir()
            raw=cache/'one.json'
            m.write(raw,{'entities':{'Q1':{'id':'Q1','labels':{'en':{'value':'One'}},'claims':{}}}})
            m.write(out/'selection.json',{'entities':[{'qid':'Q1'},{'qid':'Q2'}]})
            m.write(out/'fetch.json',{'reused_sources':[m.identity(raw)],'requests':[]})
            m.rebuild(cache,out)
            before=(out/'scope1000/index.json').read_bytes()
            m.write(cache/'two.json',{'entities':{'Q2':{'id':'Q2','labels':{'en':{'value':'Two'}},'claims':{}}}})
            m.rebuild(cache,out)
            self.assertEqual(before,(out/'scope1000/index.json').read_bytes())
            raw.write_text('{}')
            with self.assertRaises(AssertionError):
                m.rebuild(cache,out)

    def test_incomplete_raw_reply_is_not_an_entity_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'raw.json'
            path.write_text('{\"entities\":')
            self.assertEqual(m.cached([path], {'Q1'}), ({}, []))

    def test_cache_never_imports_unselected_entities(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'raw.json'
            m.write(path,{'entities':{'Q1':{'id':'Q1','claims':{}},'Q2':{'id':'Q2'},'Q3':{'id':'Q3','missing':''}}})
            entities,_=m.cached([path],{'Q1','Q3'})
            self.assertEqual(set(entities),{'Q1'})

if __name__ == '__main__':
    unittest.main()
