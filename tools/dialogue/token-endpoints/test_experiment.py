"""Authored independent contrasts for the frozen offline endpoint mechanism."""
import unittest
import experiment as e


class EndpointContrasts(unittest.TestCase):
    def test_unseen_names_and_lengths(self):
        for source, value, count, expected in [
            ('Visit Zora Vale today.', 'Zora', 1, 'Zora Vale'),
            ('Visit Zora Vale Ridge today.', 'Zora', 2, 'Zora Vale Ridge'),
            ('Visit Nova Vale Ridge today.', 'Nova Vale', 1, 'Nova Vale Ridge'),
        ]:
            _, start, end = e.latest(value, [source])
            key, ends = e.context(source, start, end)
            self.assertEqual(source[start:ends[count - 1]], expected)
            self.assertNotIn('Zora', repr(key))
        self.assertEqual(e.shape('Zora'), e.shape('Nova'))

    def test_unicode_and_punctuation_contiguous_bytes(self):
        source = 'À bientôt: Évora-Luz Verde, please.'
        _, start, end = e.latest('Évora-Luz', [source])
        _, ends = e.context(source, start, end)
        lo, hi = len(source[:start].encode()), len(source[:ends[0]].encode())
        self.assertEqual(source.encode()[lo:hi].decode(), 'Évora-Luz Verde')
        self.assertEqual(source[ends[0]], ',')
        self.assertNotEqual(lo, start)

    def test_latest_occurrence_no_older_fallback(self):
        history = ['Try Zora Vale.', 'Correct that: Zora.']
        ti, start, end = e.latest('Zora', history)
        self.assertEqual(ti, 1)
        self.assertIsNone(e.context(history[ti], start, end))
        self.assertEqual(e.latest('Zora', ['Zora Vale or Zora Ridge?']), (0, 13, 17))
        self.assertIsNone(e.latest('Zora', ['Zoraville']))
        self.assertIsNone(e.latest('Zora', ['zora Vale']))

    def test_complete_value_and_unknown_bounds(self):
        source = 'Zora Vale'
        self.assertIsNone(e.context(source, 0, len(source)))
        self.assertIsNone(e.context('x' * 4097, 0, 1))
        self.assertIsNone(e.context('a ' * 257, 0, 1))
        self.assertIsNone(e.context('Zora Vale', 1, 4))
        self.assertIsNone(e.gap('....'))

    def test_missing_and_conflicting_annotations_are_not_matches(self):
        sources = [['A', 'city']]
        absent = {'frames': []}
        conflicting = {'frames': [{'service': 'A', 'slots': [
            {'slot': 'city', 'start': 0, 'exclusive_end': 9},
            {'slot': 'city', 'start': 0, 'exclusive_end': 15},
        ]}]}
        self.assertNotEqual(e.endpoints(absent, 0, sources), [9])
        self.assertNotEqual(e.endpoints(conflicting, 0, sources), [9])
        conflicting['frames'][0]['slots'].pop()
        self.assertEqual(e.endpoints(conflicting, 0, sources), [9])
        self.assertEqual(e.endpoints(conflicting, 1, sources), [])
        self.assertEqual(e.endpoints(conflicting, 0, [['B', 'city']]), [])


if __name__ == '__main__':
    unittest.main()
