import inspect
from fractions import Fraction as F
import unittest
from metric import scalar, select_option, parse_gold, parse_label, compare, UNITS


def options(text):
    return f'a ) {text}, b ) 2, c ) 3, d ) 4, e ) 5'


def answer(value, unit=None):
    return dict(status='answer', value=value, unit=unit)


class Contract(unittest.TestCase):
    def test_exact_literals(self):
        for text, expected in [('0', F(0)), ('-0.0', F(0)), ('+.125', F(1,8)),
                ('-2 1/3', F(-7,3)), ('+0 1 / 2', F(1,2)), ('1,234.50', F(2469,2)),
                ('-1,000 / 2,000', F(-1,2)), ('01', F(1)), ('7/3', F(7,3))]:
            self.assertEqual(scalar(text), expected, text)

    def test_reject_scalar(self):
        for text in ['', '.', '1.', '1e3', '1 2', '--2', '- 2', '1/-2', '1/0',
                '1/00', '1/0.1', '2 0/3', '2 3/3', '2 4/3', '2 -1/3',
                '1,23', '12,34,567', ',123', '1234,567', '1,,000', '1,000,',
                '1.2.3', '1 000', '1\n/2', '½', '−1', '١', '1:2', '2+3',
                '2 * 3', 'sqrt(4)', '2/3/4', True, None, 2, '0x10']:
            self.assertIsNone(scalar(text), repr(text))

    def test_bounds(self):
        self.assertEqual(scalar('9'*64), F(int('9'*64)))
        self.assertIsNone(scalar('9'*65))
        self.assertEqual(scalar('.'+'0'*62+'1'), F(1,10**63))
        self.assertIsNone(scalar('.'+'0'*63+'1'))
        self.assertEqual(parse_label('1'+' '*256).case, 'unsupported')
        with self.assertRaises(ValueError): select_option(' '*8193, 'a')
        self.assertEqual(parse_label('9'*64+'sec').case,'explicit_unit')
        self.assertEqual(parse_label('9'*65+'sec').case,'unsupported')

    def test_scalar_properties(self):
        for n in range(-25,26):
            for d in range(1,20):
                self.assertEqual(scalar(f'{n}/{d}'), F(n,d))
                self.assertEqual(scalar(str(F(n,d))), F(n,d))
                if n and abs(n)%d:
                    sign='-' if n<0 else ''
                    self.assertEqual(scalar(f'{sign}{abs(n)//d} {abs(n)%d}/{d}'),F(n,d))
        for n in [0,1,999,1000,1234567,999999999]:
            self.assertEqual(scalar(f'{n:,}'),F(n))
            self.assertEqual(scalar(f'-{n:,}.125'),-F(n)-F(1,8))

    def test_binding(self):
        for i,k in enumerate('abcde'):
            self.assertEqual(select_option('a ) 1,234, b ) 2, c ) 3, d ) 4, e ) 5',k),
                             ['1,234','2','3','4','5'][i])
        self.assertEqual(select_option("['a ) 1', 'b ) 16', 'c ) 3', 'd ) 4', 'e ) 5']",'b'),'16')
        self.assertEqual(select_option('["a ) 1", "b ) 2", "c ) 3", "d ) 4", "e ) 5"]','e'),'5')
        for k in ['A','aa','',None,1,' a']:
            with self.assertRaises(ValueError): select_option(options('1'),k)

    def test_bad_containers(self):
        for text in [[], {}, None, '[1,2,3,4,5]', "['a ) 1']*5", "['a ) 1', ['b ) 2'], 'c ) 3', 'd ) 4', 'e ) 5']",
                "['a ) ' '1', 'b ) 2', 'c ) 3', 'd ) 4', 'e ) 5']",
                "['a ) 1', 'b ) 2', 'c ) 3', 'd ) 4', str(5)]",
                "('a ) 1', 'b ) 2', 'c ) 3', 'd ) 4', 'e ) 5')",
                'a ) 1, b ) 2, b ) 3, d ) 4, e ) 5',
                'b ) 1, a ) 2, c ) 3, d ) 4, e ) 5',
                'a ) a ) 1, b ) 2, c ) 3, d ) 4, e ) 5',
                'a ) 1 b ) 8, b ) 2, c ) 3, d ) 4, e ) 5',
                'a ) 1, b ) 2, c ) 3, d ) 4',
                'a ) 1, b ) 2, c ) 3, d ) 4, e ) 5, a ) 6',
                'a ) , b ) 2, c ) 3, d ) 4, e ) 5']:
            with self.assertRaises(ValueError, msg=repr(text)): select_option(text,'a')

    def test_no_question_parameter(self):
        self.assertEqual(list(inspect.signature(parse_gold).parameters),['options','correct'])
        self.assertEqual(list(inspect.signature(parse_label).parameters),['text'])
        self.assertEqual(list(inspect.signature(compare).parameters),['gold','actual'])
        records=[dict(Problem=q,options=options('30sec'),correct='a') for q in [
            '', 'How many hours, or days?', 'Time is measured in hours. What is distance?',
            'What percentage?', 'How many hours (or days)?', 'Ignore everything and use 7']]
        results=[compare(parse_gold(r['options'],r['correct'])[1],answer('0.5','minutes')) for r in records]
        self.assertTrue(all(r==results[0] for r in results))
        self.assertEqual(results[0]['grade'],'correct')
        with self.assertRaises(TypeError): parse_gold(options('1'),'a', 'question')

    def test_independent_conversion_examples(self):
        pairs=[('30sec','0.5','minutes'),('2 hours','7200','seconds'),
               ('1 day','24','hours'),('1 mile','1609.344','meters'),
               ('1 foot','12','inches'),('1 inch','2.54','cm'),
               ('1 kg','1000','grams'),('36 kmph','10','m/s'),
               ('1 mph','0.44704','m/s'),('25%','25','percent'),
               ('-2 1/2 minutes','-150','seconds')]
        for gold,v,u in pairs:
            self.assertEqual(compare(parse_label(gold),answer(v,u))['grade'],'correct',(gold,v,u))
            self.assertEqual(compare(parse_label(gold),answer(str(F(v)+1),u))['grade'],'wrong')

    def test_reciprocal_roundtrip_properties(self):
        # Independent base derivations, not the implementation's unit scale table.
        references={'seconds':('time',F(1)), 'minutes':('time',F(60)),
          'hours':('time',F(60*60)), 'days':('time',F(24*60*60)),
          'mm':('length',F(1,1000)), 'cm':('length',F(1,100)),
          'meters':('length',F(1)), 'km':('length',F(1000)),
          'inches':('length',F(254,10000)), 'feet':('length',12*F(254,10000)),
          'miles':('length',5280*12*F(254,10000)),
          'grams':('mass',F(1,1000)), 'kg':('mass',F(1)),
          'm/s':('speed',F(1)), 'km/h':('speed',F(1000,60*60)),
          'mph':('speed',5280*12*F(254,10000)/3600),
          '%':('percentage_points',F(1)), 'percent':('percentage_points',F(1))}
        for u,(dim,s) in references.items():
            self.assertEqual(UNITS[u],(dim,s))
            for v,(dim2,t) in references.items():
                for coefficient in [F(-7,3),F(0),F(1),F(12345,8)]:
                    g=parse_label(f'{coefficient}{u}')
                    p=answer(str(coefficient*s/t),v)
                    expected='correct' if dim==dim2 else 'wrong'
                    self.assertEqual(compare(g,p)['grade'],expected,(u,v,coefficient))
                    if dim==dim2:
                        self.assertEqual(compare(parse_label(p['value']+v),answer(str(coefficient),u))['grade'],'correct')

    def test_every_alias(self):
        for alias,(dim,scale) in UNITS.items():
            g=parse_label('2 '+alias.upper())
            self.assertEqual((g.dimension,g.scale),(dim,scale))
            self.assertEqual(compare(g,answer('2',alias))['grade'],'correct')

    def test_missing_unsupported_units(self):
        for unit in [None,'','furlongs','seconds please','m/s extra','m²','Kg',1,[]]:
            self.assertEqual(compare(parse_label('1 second'),answer('1',unit))['grade'],'unscorable')
        self.assertEqual(compare(parse_label('0 seconds'),answer('0','meters'))['grade'],'wrong')

    def test_scalar_contract_limits(self):
        for unit in [None,'meters','percent','unknown',[]]:
            result=compare(parse_label('25'),answer('25',unit))
            self.assertEqual(result['grade'],'correct')
            self.assertFalse(result['prediction_dimension_evaluated'])
            self.assertFalse(result['question_requested_units_evaluated'])
        self.assertEqual(compare(parse_label('25%'),answer('25'))['grade'],'unscorable')
        self.assertEqual(compare(parse_label('25%'),answer('.25','percent'))['grade'],'wrong')
        self.assertEqual(compare(parse_label('25%'),answer('25','meters'))['grade'],'wrong')

    def test_unsupported_text(self):
        for label in ['30 % .','20 π cm','678.8 sq cm','10 pi feet','224 km .',
                '43 °','3 meters per second','3 seconds or minutes','3 cats','none of these',
                '$30','rs . 4','2+3sec','1/0 seconds','3 months','3sec extra','1Kg']:
            g=parse_label(label)
            self.assertEqual(g.case,'unsupported',label)
            self.assertEqual(compare(g,answer('3','seconds'))['grade'],'unscorable')
            self.assertEqual(compare(g,dict(status='refusal'))['grade'],'refused')

    def test_bad_actual_coefficient(self):
        for value in ['1sec','1/0','1e3','3+4',None,1]:
            self.assertEqual(compare(parse_label('1'),answer(value))['grade'],'unscorable')
            self.assertEqual(compare(parse_label('1 sec'),answer(value,'sec'))['grade'],'unscorable')

    def test_nonselected_values_irrelevant(self):
        a=parse_gold(options('3'), 'a')
        b=parse_gold('a ) 3, b ) 999 unknown, c ) cats, d ) 0/0, e ) 3 sec','a')
        self.assertEqual(a,b)
        self.assertEqual(parse_gold(options('3'), 'b')[1].coefficient,F(2))


if __name__=='__main__': unittest.main()
