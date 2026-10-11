from pathlib import Path
from fractions import Fraction
from collections import Counter
import hashlib,json,re,sys
root=Path('/home/gsc/.cache/glassmind/continuous-20261010');out=Path(__file__).resolve().parent
sys.path.insert(0,str(root/'quantities-run/tmp/quantities-deps'));sys.path.insert(0,'/home/gsc/code/glassmind/tools/quantities/typed-label-metric')
import pyarrow.parquet as pq
from metric import parse_gold,compare
source=root/'quantities-run/tmp/mathqa/train.parquet';rows=pq.read_table(source,columns=['Problem','linear_formula','options','correct']).to_pylist()
number=re.compile(r'(?<![\w.])(?:-?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?|\.\d+)(?![\w.])')
step=re.compile(r'(add|subtract|multiply|divide)\((n\d+|#\d+|const_\d+(?:_\d+)?),(n\d+|#\d+|const_\d+(?:_\d+)?)\)')
counts=Counter();reasons=Counter();details=[];eligible=[]
for i,row in enumerate(rows):
 q=row['Problem'];f=row['linear_formula'];pieces=f.rstrip('|').split('|');names=re.findall(r'([a-z_]+)\(',f)
 if not names or set(names)-{'add','subtract','multiply','divide'} or len(pieces)>6:counts['outside_family']+=1;continue
 nums=[Fraction(m.group().replace(',','')) for m in number.finditer(q)];values=[];error=None
 try:
  if len(nums)>16 or len(q.encode())>4096:raise ValueError('input_bound')
  for j,piece in enumerate(pieces):
   m=step.fullmatch(piece)
   if not m:raise ValueError('unsupported_program_syntax_or_constant')
   def arg(s):
    if s.startswith('n'):
     n=int(s[1:])
     if n>=len(nums):raise ValueError('missing_number_slot')
     return nums[n]
    if s.startswith('#'):
     n=int(s[1:])
     if n>=len(values):raise ValueError('invalid_step_reference')
     return values[n]
    return Fraction(s[6:].replace('_','.'))
   op,a,b=m.groups();a,b=arg(a),arg(b)
   if op=='add':v=a+b
   elif op=='subtract':v=a-b
   elif op=='multiply':v=a*b
   else:v=a/b
   if abs(v.numerator)>=2**127 or v.denominator>=2**127:raise ValueError('rational_bound')
   values.append(v)
 except (ValueError,ZeroDivisionError) as e:error=str(e) or 'division_by_zero'
 if error:counts['execution_refusal']+=1;reasons[error]+=1;continue
 _,gold=parse_gold(row['options'],row['correct']);actual={'status':'answer','value':str(values[-1]),'unit':None};grade=compare(gold,actual)['grade'];counts[grade]+=1
 eligible.append({'index':i,'question_sha256':hashlib.sha256(q.encode()).hexdigest(),'template':f,'number_count':len(nums),'grade':grade})
 if len(details)<12 and grade!='correct':details.append({'index':i,'question':q,'formula':f,'numbers':list(map(str,nums)),'program_value':str(values[-1]),'selected_train_gold':gold.record(),'grade':grade})
report={'source':str(source),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'train_rows':len(rows),'split':'TRAIN only; annotations audited offline, never runtime input','number_binding':'bounded left-to-right numeric lexemes; fraction components remain separate; no number words or English entailment claim','program_family':'at most6 add/subtract/multiply/divide steps, backward refs, numeric constants, at most16 numeric inputs; const_pi unsupported','counts':dict(counts),'execution_refusal_reasons':dict(reasons),'metric':'unchanged exact-label-comparison-v3; dimensionless proposed program; unsupported/unit labels remain in denominator','disagreements_sample':details}
(out/'annotation-audit.json').write_text(json.dumps(report,indent=2)+'\n');(out/'eligible-programs.json').write_text(json.dumps(eligible,separators=(',',':'))+'\n');print(json.dumps({k:v for k,v in report.items() if k!='disagreements_sample'},indent=2))
