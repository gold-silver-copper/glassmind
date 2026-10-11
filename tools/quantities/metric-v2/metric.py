"""Offline v2 only. No solver imports, inference, or answer-dependent normalization."""
import ast
from fractions import Fraction
import re

UNITS = {}
def units(names, dimension, scale):
    for name in names.split('|'):
        UNITS[name] = (dimension, Fraction(scale))
units('s|sec|secs|second|seconds','time',1)
units('min|mins|minute|minutes','time',60)
units('hr|hrs|hour|hours','time',3600)
units('day|days','time',86400)
units('mm|millimeter|millimeters|millimetre|millimetres','length','1/1000')
units('cm|centimeter|centimeters|centimetre|centimetres','length','1/100')
units('meter|meters|metre|metres','length',1)
units('km|kilometer|kilometers|kilometre|kilometres','length',1000)
units('inch|inches','length','127/5000')
units('ft|foot|feet','length','381/1250')
units('mile|miles','length','201168/125')
units('g|gram|grams','mass',1)
units('kg|kilogram|kilograms','mass',1000)
units('l|liter|liters|litre|litres','volume',1)
units('degree|degrees|°','angle',1)
units('kmph|km/h|km/hr','speed','5/18')
units('mps|m/s','speed',1)
UNIT_RE = '|'.join(re.escape(x).replace('/',r'\s*/\s*') for x in sorted(UNITS,key=len,reverse=True))
INT = r'(?:\d{1,3}(?:,\d{3})+|\d+)'
DEC = rf'(?:{INT}(?:\.\d+)?|\.\d+)'
NUM = rf'[+-]?(?:{INT}\s+\d+\s*/\s*\d+|\d+\s*/\s*\d+|{DEC})'


def scalar(text):
    text = text.strip()
    if len(text)>64 or not re.fullmatch(NUM,text): return None
    s = text.replace(',','')
    mixed = re.fullmatch(r'([+-]?)(\d+)\s+(\d+)\s*/\s*(\d+)',s)
    try:
        if mixed:
            sign,w,n,d=mixed.groups(); w,n,d=map(int,(w,n,d))
            if not 0<=n<d: return None
            return (-1 if sign=='-' else 1)*(Fraction(w)+Fraction(n,d))
        return Fraction(re.sub(r'\s*/\s*','/',s))
    except (ValueError,ZeroDivisionError): return None


def select_option(options, correct):
    if not isinstance(options,str) or len(options)>8192 or correct not in 'abcde' or len(correct)!=1:
        raise ValueError('container_or_key')
    text=options.strip()
    if text.startswith('['):
        try: parts=ast.literal_eval(text)
        except (ValueError,SyntaxError,RecursionError): raise ValueError('list_container')
        if not isinstance(parts,list) or len(parts)!=5 or not all(isinstance(p,str) for p in parts):
            raise ValueError('list_shape')
    else:
        # Delimit only complete option labels, never thousands commas.
        markers=list(re.finditer(r'(?:^|,\s*)([a-e])\s*\)\s*',text))
        if len(markers)!=5 or markers[0].start()!=0: raise ValueError('option_labels')
        parts=[text[m.start(1):markers[i+1].start() if i+1<5 else len(text)] for i,m in enumerate(markers)]
    bound={}
    for expected,part in zip('abcde',parts):
        m=re.fullmatch(r'([a-e])\s*\)\s*(.*?)\s*',part,re.S)
        if not m or m[1]!=expected or re.search(r'\b[a-e]\s*\)',m[2]): raise ValueError('option_binding')
        bound[expected]=m[2]
    return bound[correct]


def request_unit(question):
    q=question.lower()
    found=[]
    for m in re.finditer(rf'\b(?:how many|in)\s+({UNIT_RE})(?![a-z0-9/])',q):
        if re.match(r'\s+(?:or|and|per|squared|cubed)\b|\s*(?:/|\^|[²³])',q[m.end():]): return None,'ambiguous_requested_unit'
        found.append(re.sub(r'\s*/\s*','/',m[1]))
    specs={UNITS[x] for x in found}
    if len(specs)!=1: return None,'ambiguous_requested_unit' if specs else 'missing_explicit_requested_unit'
    return found[0],None


def parse_gold(text,question):
    result=dict(value=None,reason=None,gold_unit=None,requested_unit=None,raw_value=None)
    def reject(reason): result['reason']=reason; return result
    if not isinstance(text,str) or len(text)>256 or len(question)>8192: return reject('bounds')
    text=text.strip().lower()
    bare=scalar(text)
    if bare is not None:
        result.update(value=str(bare),raw_value=str(bare),reason='bare_requested_unit_contract'); return result
    if text.endswith('%'):
        v=scalar(text[:-1]); q=question.lower()
        if v is None: return reject('malformed_percent')
        if not re.search(r'\b(?:what percent|what percentage|percentage|gain percent|loss percent|profit percent|percent increase|percent decrease)\b',q): return reject('missing_explicit_percent_request')
        u,err=request_unit(question)
        if u: return reject('dimension_mismatch')
        result.update(value=str(v),raw_value=str(v),gold_unit='percent',requested_unit='percent',reason='percentage_points'); return result
    m=re.fullmatch(rf'({NUM})\s*({UNIT_RE})',text)
    if not m:
        if re.search(r'\b(?:none|neither|cannot|cannot be|all of)\b',text): return reject('non_scalar_choice_text')
        if re.fullmatch(r'[+\-\d., /]+',text): return reject('malformed_numeric')
        if re.match(NUM,text): return reject('unsupported_suffix_or_expression')
        return reject('unsupported_text_or_expression')
    v=scalar(m[1]); unit=re.sub(r'\s*/\s*','/',m[2])
    if v is None: return reject('malformed_numeric')
    result.update(raw_value=str(v),gold_unit=unit)
    requested,error=request_unit(question)
    if error: return reject(error)
    result['requested_unit']=requested
    dim,scale=UNITS[unit]; rdim,rscale=UNITS[requested]
    if dim!=rdim: return reject('dimension_mismatch')
    result.update(value=str(v*scale/rscale),reason='explicit_unit_conversion')
    return result
