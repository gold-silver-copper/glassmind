"""Post-freeze independent English derivations; no inference or label parsing."""
import json
from fractions import Fraction as F
from pathlib import Path
HERE=Path(__file__).resolve().parent
# Read English in full/changes.json; derive exact arithmetic independently.
derivations={
265: ('The town starts at 8000; each year removes 30% of that year\'s population. Two years leave 70% twice.', '8000 * (7/10)^2', F(8000)*F(7,10)**2, None),
1482: ('The current 3744 is the population after two annual 20% gains. Divide by 1.2 twice to recover the explicitly earlier population.', '3744 / (6/5)^2', F(3744)/F(6,5)**2, None),
1751: ('The present population is 2000, and the query is one year after at 20% annual growth.', '2000 * 6/5', F(2000)*F(6,5), None),
2854: ('The increase is 50% of the explicitly supplied 70. Add 35 to 70.', '70 + 70/2', F(70)+F(70,2), None),
3805: ('The present 15000 gains 10% per year for two years; the annual base changes after the first year.', '15000 * (11/10)^2', F(15000)*F(11,10)**2, None),
4013: ('The second 20% increase explicitly uses the new price. Normalizing the positive original price to 100, stages are 100 ->120 ->144; original-base increase is44%.', '(6/5 * 6/5 - 1) * 100', (F(6,5)*F(6,5)-1)*100, 'percent'),
4071: ('The first25% gain gives125 per original100. The second30% gain uses125, adding37.5. Final162.5 is62.5% above original.', '(5/4 * 13/10 - 1) * 100', (F(5,4)*F(13,10)-1)*100, 'percent'),
}
changes=json.loads((HERE/'full/changes.json').read_text());assert set(derivations)=={r['index'] for r in changes}
rows=[]
for row in changes:
 explanation,formula,value,unit=derivations[row['index']];a=row['series']['candidate']['actual']
 ok=F(a['value'])==value and a['unit']==unit
 rows.append(dict(index=row['index'],question=row['question'],reasoning=explanation,formula=formula,independent_value=str(value),independent_unit=unit,actual=a,mathematically_justified=ok,label_grade=row['series']['candidate']['grade'],discrepancies=[] if ok else ['Independent English derivation differs']))
assert all(r['mathematically_justified'] for r in rows)
(HERE/'semantic-audit.json').write_text(json.dumps(dict(total=len(rows),justified=len(rows),discrepancies=0,assumptions=['Standard annual population compounding, no migration or rounding implied.','Original price is positive when defining its percentage change.','Population results are integer counts; existing scalar output convention uses null unit.'],rows=rows),indent=2)+'\n')
print('7/7 new acceptances independently derived; no discrepancies')
