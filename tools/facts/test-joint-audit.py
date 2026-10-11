"""A shared answer ID must not hide a wrong subject or relation."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('joint', Path(__file__).with_name('joint-audit.py'))
joint = importlib.util.module_from_spec(spec)
spec.loader.exec_module(joint)


class Agreement(unittest.TestCase):
    def test_same_answer_different_claim(self):
        rows = [['Q1','P27','Q30','question'] for _ in range(4)]
        predictions = [dict(entity=e, relation=r, answers=a, refusal=f) for e,r,a,f in
                       [('Q1','P27',['Q30'],None), ('Q2','P27',['Q30'],None),
                        ('Q1','P19',['Q30'],None), (None,None,[],'unknown')]]
        result = joint.audit(rows,predictions)
        self.assertEqual(result['answer_only'],dict(right=3,refused=1,wrong=0))
        self.assertEqual(result['joint_entity_relation_answer'],dict(right=1,refused=1,wrong=2))
        self.assertEqual([x['row'] for x in result['answer_matches_with_wrong_link_or_relation']],[2,3])

    def test_missing_predictions_and_contradictory_refusals_fail(self):
        row = ['Q1','P27','Q30','question']
        for rows,preds in [([row],[]), ([],[]),
                           ([row],[dict(entity='Q1',relation='P27',answers=['Q30'],refusal='unknown')])]:
            with self.assertRaises(ValueError):
                joint.audit(rows,preds)


if __name__ == '__main__':
    unittest.main()
