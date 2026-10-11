"""Author the public, preimplementation comparison contract. No external labels."""
import json
from pathlib import Path
root = Path(__file__).parent
cases = []
def yes(q, dim, order, kind, op, answer, left, right):
    cases.append(dict(question=q, dimension=dim, ordering=order, kind=kind,
                      operation=op, answer=answer, witness_left=left, witness_right=right))
def rel(q,d,o,p,t,l,r): yes(q,d,o,'predicate',p,t,l,r)
def sel(q,d,o,p,s,l,r): yes(q,d,o,'selection',p,s,l,r)
rel('Is 2 kg greater than 1500 g?', 'mass','greater','greater_than',True,'2','3/2')
sel('Which is longer, 3 feet or 1 metre?', 'length','less','longer','right','3','1250/381')
rel('Is 1/3 hour equal to 20 minutes?', 'duration','equal','equal_to',True,'1/3','1/3')
rel('Is 2 kg less than 1500 g?', 'mass','greater','less_than',False,'2','3/2')
rel('Is 1 metre at least 100 cm?', 'length','equal','at_least',True,'1','1')
rel('Is 1 litre at most 999 ml?', 'volume','greater','at_most',False,'1','999/1000')
rel('Is 36 km/h equal to 10 m/s?', 'speed','equal','equal_to',True,'36','36')
rel('Is 0 celsius equal to 32 fahrenheit?', 'temperature','equal','equal_to',True,'0','0')
rel('Is -40 fahrenheit equal to -40 celsius?', 'temperature','equal','equal_to',True,'-40','-40')
rel('Is 273.15 kelvin equal to 0 degrees celsius?', 'temperature','equal','equal_to',True,'5463/20','5463/20')
rel('Is -2 kg greater than -1500 g?', 'mass','less','greater_than',False,'-2','-3/2')
rel('Is -1/3 hour less than -19 minutes?', 'duration','less','less_than',True,'-1/3','-19/60')
rel('  iS\t+1.25\tUS   GALLONS  At LEAST  1 us gallon ?  ', 'volume','greater','at_least',True,'5/4','1')
rel('Is .5 m equal to 50 cm', 'length','equal','equal_to',True,'1/2','1/2')
rel('Is 1/-3 hour equal to -20 minutes?', 'duration','equal','equal_to',True,'-1/3','-1/3')
rel('Is 1 kg equal to 2 kg?', 'mass','less','equal_to',False,'1','2')
rel('Is 100 cm less than 2 m?', 'length','less','less_than',True,'100','200')
rel('Is 2 m greater than 100 cm?', 'length','greater','greater_than',True,'2','1')
rel('Is 2 m at most 100 cm?', 'length','greater','at_most',False,'2','1')
rel('Is 100 cm at least 2 m?', 'length','less','at_least',False,'100','200')
sel('Which is shorter, 3 feet or 1 metre?', 'length','less','shorter','left','3','1250/381')
sel('Which is heavier, 2 kg or 1500 g?', 'mass','greater','heavier','left','2','3/2')
sel('Which is lighter, 2 kg or 1500 g?', 'mass','greater','lighter','right','2','3/2')
sel('Which is faster, 10 m/s or 35 km/h?', 'speed','greater','faster','left','10','175/18')
sel('Which is slower, 10 m/s or 35 km/h?', 'speed','greater','slower','right','10','175/18')
sel('Which is hotter, 10 celsius or 40 fahrenheit?', 'temperature','greater','hotter','left','10','40/9')
sel('Which is colder, 10 celsius or 40 fahrenheit?', 'temperature','greater','colder','right','10','40/9')
sel('Which is hotter, -300 celsius or -400 celsius?', 'temperature','greater','hotter','left','-300','-400')
sel('WHICH IS LONGER, 1 M OR 100 CM?', 'length','equal','longer','equal','1','1')
sel('Which is lighter, 1 pound or 16 ounces?', 'mass','equal','lighter','equal','1','1')
sel('Which is faster, 36 km/h or 10 m/s?', 'speed','equal','faster','equal','36','36')
sel('Which is colder, 32 degrees fahrenheit or 0 degrees celsius?', 'temperature','equal','colder','equal','32','32')
for q, error in [
 ('Is 1 kg greater than 2 m?', 'dimension_mismatch'),
 ('Which is longer, 1 kg or 2 kg?', 'selection_dimension_mismatch'),
 ('Which is heavier, 1 m or 2 m?', 'selection_dimension_mismatch'),
 ('Which is faster, 1 hour or 2 hours?', 'selection_dimension_mismatch'),
 ('Which is hotter, 1 litre or 2 litres?', 'selection_dimension_mismatch'),
 ('Is 1 gallon equal to 1 us gallon?', 'invalid_unit'),
 ('Is 1 stone greater than 1 kg?', 'invalid_unit'),
 ('Is 1 kg g greater than 1 kg?', 'invalid_unit'),
 ('Is 1/0 hour equal to 20 minutes?', 'invalid_number'),
 ('Is NaN kg greater than 1 kg?', 'invalid_number'),
 ('Is inf kg less than 2 kg?', 'invalid_number'),
 ('Is 1e3 g equal to 1 kg?', 'invalid_number'),
 ('Is 1..2 m equal to 1 m?', 'invalid_number'),
 ('Is 1/2/3 m equal to 1 m?', 'invalid_number'),
 ('Is 170141183460469231731687303715884105728 m equal to 1 m?', 'invalid_number'),
 ('Is 170141183460469231731687303715884105727 km equal to 1 m?', 'overflow'),
 ('Is 1 kg greater than 2 kg? Remember my name is Ada.', 'invalid_unit'),
 ('Is 1 kg greater than 2 kg and I own a dog', 'invalid_unit'),
 ('Is 1 kg greater than 2 kg??', 'invalid_unit'),
 ('Is 1 kg greater than 2?', 'grammar'),
 ('Is greater than 2 kg?', 'grammar'),
 ('Is 1 kg greater than?', 'grammar'),
 ('Is 1 greater than 2?', 'grammar'),
 ('Is 1 + 2 kg greater than 2 kg?', 'invalid_unit'),
 ('Is two kg greater than 1 kg?', 'invalid_number'),
 ('Which is longer, it or 1 metre?', 'grammar'),
 ('Which is longer 1 m or 2 m?', 'grammar'),
 ('Which is more, 1 m or 2 m?', 'grammar'),
 ('Is 1 kg >= 1 kg?', 'grammar'),
 ('Is 1 kg greater than 1 kg?\x00', 'invalid_input'),
 ('x'*4097, 'invalid_input'),
 ('é'*2049, 'invalid_input'),
]: cases.append(dict(question=q, refusal=error))
(root/'authored.json').write_text(json.dumps(cases, indent=2, ensure_ascii=False)+'\n')
