"""Question-independent exact-label comparison v3. Pure, bounded, no inference."""
import ast
import io
import tokenize
from dataclasses import dataclass
from fractions import Fraction
import re

WS = ' \t\r\n'
INT = r'(?:[0-9]{1,3}(?:,[0-9]{3})+|[0-9]+)'
MIXED = re.compile(r'([+-]?)(' + INT + r')[ \t]+(' + INT + r')[ \t]*/[ \t]*(' + INT + r')')
RATIONAL = re.compile(r'([+-]?)(' + INT + r')[ \t]*/[ \t]*(' + INT + r')')
NUMBER = re.compile(r'[+-]?(?:' + INT + r'(?:\.[0-9]+)?|\.[0-9]+)')
LABEL = re.compile(r'([a-e])[ \t]*\)[ \t]*(.*)', re.DOTALL)
EMBEDDED = re.compile(r'(?<![A-Za-z])[a-e][ \t]*\)')
UNITS = {}
def _unit(dimension, scale, aliases):
    for alias in aliases.split('|'):
        assert alias not in UNITS
        UNITS[alias] = (dimension, Fraction(scale))
_unit('time', '1', 's|sec|secs|second|seconds')
_unit('time', '60', 'min|mins|minute|minutes')
_unit('time', '3600', 'h|hr|hrs|hour|hours')
_unit('time', '86400', 'day|days')
_unit('length', '1/1000', 'mm|millimeter|millimeters|millimetre|millimetres')
_unit('length', '1/100', 'cm|centimeter|centimeters|centimetre|centimetres')
_unit('length', '1', 'm|meter|meters|metre|metres')
_unit('length', '1000', 'km|kilometer|kilometers|kilometre|kilometres')
_unit('length', '127/5000', 'in|inch|inches')
_unit('length', '381/1250', 'ft|foot|feet')
_unit('length', '201168/125', 'mi|mile|miles')
_unit('mass', '1/1000', 'g|gm|gms|gram|grams')
_unit('mass', '1', 'kg|kgs|kilogram|kilograms')
_unit('speed', '1', 'm/s|mps')
_unit('speed', '5/18', 'km/h|km/hr|kmph|kph')
_unit('speed', '1397/3125', 'mph')
_unit('percentage_points', '1', '%|percent|percentage points')


def scalar(text):
    if not isinstance(text, str):
        return None
    text = text.strip(WS)
    if not text or len(text) > 64:
        return None
    m = MIXED.fullmatch(text)
    if m:
        sign, w, n, d = m.groups()
        w, n, d = (int(x.replace(',', '')) for x in (w, n, d))
        if not 0 < n < d:
            return None
        return (-1 if sign == '-' else 1) * (Fraction(w) + Fraction(n, d))
    m = RATIONAL.fullmatch(text)
    if m:
        sign, n, d = m.groups()
        n, d = (int(x.replace(',', '')) for x in (n, d))
        return None if d == 0 else Fraction((-1 if sign == '-' else 1) * n, d)
    if NUMBER.fullmatch(text):
        return Fraction(text.replace(',', ''))
    return None


def select_option(options, correct):
    if not isinstance(correct, str) or correct not in tuple('abcde'):
        raise ValueError('invalid_correct_key')
    if not isinstance(options, str) or len(options) > 8192:
        raise ValueError('invalid_container')
    text = options.strip(WS)
    if text.startswith('['):
        try:
            tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
            meaningful = [t for t in tokens if t.type not in (tokenize.NEWLINE, tokenize.NL, tokenize.ENDMARKER)]
            if sum(t.type == tokenize.STRING for t in meaningful) != 5 or any(
                    t.type != tokenize.STRING and not (t.type == tokenize.OP and t.string in ('[', ']', ','))
                    for t in meaningful):
                raise ValueError('invalid_list_tokens')
            tree = ast.parse(text, mode='eval')
        except (SyntaxError, ValueError, RecursionError, tokenize.TokenError):
            raise ValueError('invalid_list_syntax') from None
        if not isinstance(tree.body, ast.List) or len(tree.body.elts) != 5:
            raise ValueError('invalid_list_shape')
        if any(not isinstance(n, ast.Constant) or type(n.value) is not str
               or len(n.value) > 1024 for n in tree.body.elts):
            raise ValueError('invalid_list_element')
        entries = [n.value for n in tree.body.elts]
    else:
        entries = re.split(r',[ \t\r\n]*(?=[a-e][ \t]*\))', text)
    if len(entries) != 5:
        raise ValueError('option_count')
    values = []
    for key, entry in zip('abcde', entries):
        m = LABEL.fullmatch(entry.strip(WS))
        if not m or m[1] != key:
            raise ValueError('option_order_or_binding')
        value = m[2].strip(WS)
        if not value or EMBEDDED.search(value):
            raise ValueError('empty_or_embedded_label')
        values.append(value)
    return values['abcde'.index(correct)]


@dataclass(frozen=True)
class Gold:
    case: str
    reason: str
    coefficient: Fraction | None = None
    dimension: str | None = None
    scale: Fraction | None = None
    unit: str | None = None

    def record(self):
        return dict(case=self.case, reason=self.reason,
                    coefficient=None if self.coefficient is None else str(self.coefficient),
                    dimension=self.dimension, scale=None if self.scale is None else str(self.scale),
                    unit=self.unit, canonical=None if self.coefficient is None else str(
                        self.coefficient * (self.scale or 1)))


def parse_label(text):
    if not isinstance(text, str) or len(text) > 256:
        return Gold('unsupported', 'label_type_or_bound')
    text = text.strip(WS)
    value = scalar(text)
    if value is not None:
        return Gold('bare_scalar', 'exact_scalar', value)
    if not text.isascii():
        return Gold('unsupported', 'unsupported_or_malformed_label')
    lower = text.lower()
    # Suffix recognition is full-label recognition, never arbitrary stripping.
    for alias in sorted(UNITS, key=lambda x: (-len(x), x)):
        if lower.endswith(alias):
            value = scalar(text[:-len(alias)])
            if value is not None:
                dimension, scale = UNITS[alias]
                return Gold('explicit_unit', 'exact_explicit_unit', value, dimension, scale, alias)
    return Gold('unsupported', 'unsupported_or_malformed_label')


def parse_gold(options, correct):
    try:
        selected = select_option(options, correct)
    except ValueError as e:
        return None, Gold('unsupported', 'option_binding:' + str(e))
    return selected, parse_label(selected)


def compare(gold, actual):
    flags = dict(question_requested_units_evaluated=False,
                 prediction_dimension_evaluated=False)
    def result(grade, reason, canonical=None):
        return dict(grade=grade, reason=reason, predicted_canonical=canonical, **flags)
    if actual['status'] == 'refusal':
        return result('refused', 'frozen_refusal')
    if actual['status'] != 'answer':
        raise ValueError('invalid_answer_status')
    if gold.case == 'unsupported':
        return result('unscorable', 'unsupported_gold')
    value = scalar(actual.get('value'))
    if value is None:
        return result('unscorable', 'unsupported_prediction_coefficient')
    if gold.case == 'bare_scalar':
        return result('correct' if value == gold.coefficient else 'wrong',
                      'scalar_coefficient_only', str(value))
    unit = actual.get('unit')
    if unit is None or unit == '':
        return result('unscorable', 'missing_prediction_unit')
    if not isinstance(unit, str) or not unit.isascii() or unit.strip(WS).lower() not in UNITS:
        return result('unscorable', 'unsupported_prediction_unit')
    dimension, scale = UNITS[unit.strip(WS).lower()]
    flags['prediction_dimension_evaluated'] = True
    canonical = value * scale
    if dimension != gold.dimension:
        return result('wrong', 'incompatible_dimension', str(canonical))
    return result('correct' if canonical == gold.coefficient * gold.scale else 'wrong',
                  'exact_physical_label', str(canonical))
