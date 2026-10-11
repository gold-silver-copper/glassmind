import unittest
from fractions import Fraction as F
from metric import parse_gold, select_option, scalar

class Tests(unittest.TestCase):
    def test_exact(self):
        for s,v in [('0',F(0)),('-0',F(0)),('1,234.50',F(2469,2)),('.125',F(1,8)),('-2 3/4',F(-11,4)),('0 1 / 2',F(1,2)),('-3 / 8',F(-3,8))]:
            self.assertEqual(scalar(s),v,s)
    def test_malformed(self):
        for s in ['1 2','1,23','1/0','1/-2','1 3/2','1 1/0','1 + 2','3e2','--1','1..2','NaN','2:3','3.' ,'9'*65]:
            self.assertIsNone(scalar(s),s)
    def test_units(self):
        for text,q,v in [('90 minutes','How many hours are needed?',F(3,2)),('-2 1/2 hours','How many minutes are needed?',F(-150)),('0 kg','What is the mass in grams?',F(0)),('36 kmph','What is the speed in m/s?',F(10)),('12 inches','How many feet?',F(1)),('240 %','120 is what percent of 50?',F(240))]:
            self.assertEqual(parse_gold(text,q)['value'],str(v))
    def test_context_refusals(self):
        for text,q in [('3 hours','How many meters?'),('3 hours','How long?'),('3 meters','What is the speed in meters per second?'),('3 hours','How many hours or days?'),('3 hours','How many fortnights?'),('2 furlongs','How many meters?'),('25 %','What fraction?'),('3 hours extra','How many hours?'),('2 hours 3 minutes','How many minutes?'),('none of these','How many days?')]:
            self.assertIsNone(parse_gold(text,q)['value'],(text,q))
    def test_binding(self):
        a='a ) 1,000 , b ) 2 , c ) 3 , d ) 4 , e ) 5'
        self.assertEqual(select_option(a,'a'),'1,000')
        self.assertEqual(select_option("['a ) 8', 'b ) 16', 'c ) 24', 'd ) 32', 'e ) 48']",'b'),'16')
        for s in ['a ) 1, b ) 2', 'a ) 1, a ) 2, c ) 3, d ) 4, e ) 5',a+', b ) 99']:
            with self.assertRaises(ValueError): select_option(s,'a')
        with self.assertRaises(ValueError): select_option(a,'z')
    def test_hand_computed_train(self):
        # TRAIN50: rates x/10 + 2x/5 = x/2 boxes/min; 10x takes 20 min.
        self.assertEqual(parse_gold('20 minutes','How many minutes does it take?')['value'],'20')
        # TRAIN220: 15*25/20 = 75/4 days, exactly 18.75.
        self.assertEqual(parse_gold('18.75 days','In how many days can 20 men complete work?')['value'],'75/4')
        # TRAIN3: 120/50 *100 =240 percentage points.
        self.assertEqual(parse_gold('240 %','120 is what percent of 50?')['value'],'240')
        # TRAIN324: t+(400-20t)/70=8 => t=16/5 hours.
        self.assertEqual(parse_gold('3.2 hr','How many hours has he been driving?')['value'],'16/5')

if __name__=='__main__': unittest.main()
