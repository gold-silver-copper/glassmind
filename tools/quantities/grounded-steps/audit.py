#!/usr/bin/env python3
"""Offline official-TRAIN label audit. No learned inference or runtime features."""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess

CONSTANTS = ['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
MAX = (1 << 127) - 1
MIN = -(1 << 127)
OPS = {'+':'add', '-':'subtract', '*':'multiply', '/':'divide'}
NUM = r'(?:[0-9]+(?:\.[0-9]+)?|\.[0-9]+)'
SCALAR = re.compile(r'-?(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]+)?|-?\.[0-9]+')
TOKEN = re.compile(NUM + r'|[()+*/-]')

class Reject(Exception):
    pass

def sha(b):
    return hashlib.sha256(b).hexdigest()

def dump(path, obj):
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=2, sort_keys=True)+'\n')

def jsonl(path, rows):
    with path.open('w') as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True)+'\n')

def checked(n):
    if not MIN <= n <= MAX:
        raise Reject('rational_overflow')
    return n

def rational(n, d=1):
    checked(n); checked(d)
    if d == 0:
        raise Reject('division_by_zero')
    if n == MIN or d == MIN:
        raise Reject('rational_overflow')
    return Fraction(n, d)

def scalar(text):
    if len(text) > 100:
        raise Reject('number_too_long')
    if SCALAR.fullmatch(text) is None:
        raise Reject('unsupported_scalar')
    text = text.replace(',', '')
    if '.' in text:
        whole, frac = text.split('.')
        return rational(int(whole+frac), checked(10**len(frac)))
    return rational(int(text))

def native_value(text):
    # Only executor-generated canonical rational strings, never expression source.
    if re.fullmatch(r'-?[0-9]+/[0-9]+', text):
        n, d = text.split('/')
        return rational(int(n), int(d))
    return scalar(text)

def calculate(op, a, b):
    if op == '-':
        b = rational(checked(-b.numerator), b.denominator)
        op = '+'
    if op == '+':
        common = math.gcd(a.denominator, b.denominator)
        left = b.denominator // common
        right = a.denominator // common
        n = checked(checked(a.numerator*left)+checked(b.numerator*right))
        return rational(n, checked(a.denominator*left))
    if op == '/':
        b = rational(b.denominator, b.numerator)
    x = math.gcd(abs(a.numerator), b.denominator)
    y = math.gcd(abs(b.numerator), a.denominator)
    return rational(checked((a.numerator//x)*(b.numerator//y)),
                    checked((a.denominator//y)*(b.denominator//x)))

class Parser:
    def __init__(self, text):
        if len(text) > 1024:
            raise Reject('expression_bound')
        self.tokens = []
        pos = 0
        while pos < len(text):
            if text[pos] in ' \t\r\n\v\f':
                pos += 1
                continue
            m = TOKEN.match(text, pos)
            if not m:
                raise Reject('unsupported_expression')
            self.tokens.append(m.group()); pos = m.end()
        if len(self.tokens) > 128:
            raise Reject('expression_bound')
        self.i = 0
    def peek(self):
        return self.tokens[self.i] if self.i < len(self.tokens) else None
    def atom(self, depth):
        if depth > 32:
            raise Reject('expression_bound')
        t = self.peek()
        self.i += 1
        if t == '(':
            node = self.expr(depth+1)
            if self.peek() != ')':
                raise Reject('unsupported_expression')
            self.i += 1
            return node
        if t is None or re.fullmatch(NUM, t) is None:
            raise Reject('unsupported_expression')
        return {'literal':t}
    def term(self, depth):
        n = self.atom(depth)
        while self.peek() in ('*','/'):
            op = self.peek(); self.i += 1
            n = {'op':op, 'left':n, 'right':self.atom(depth)}
        return n
    def expr(self, depth=0):
        n = self.term(depth)
        while self.peek() in ('+','-'):
            op = self.peek(); self.i += 1
            n = {'op':op, 'left':n, 'right':self.term(depth)}
        return n
    def parse(self):
        n = self.expr()
        if self.i != len(self.tokens) or 'op' not in n:
            raise Reject('unsupported_expression')
        return n

def shape(node):
    if 'literal' in node:
        return []
    return shape(node['left']) + shape(node['right']) + [OPS[node['op']]]

def evaluate(node):
    if 'literal' in node:
        node['value'] = scalar(node['literal'])
    else:
        node['value'] = calculate(node['op'], evaluate(node['left']), evaluate(node['right']))
    return node['value']

def flatten(trees):
    nodes, leaves = [], []
    def visit(node, ancestors, annotation):
        if 'literal' in node:
            leaf = {'literal':node['literal'], 'value':str(node['value']),
                    'available_before':len(nodes), 'annotation':annotation,
                    '_ancestors':ancestors}
            leaves.append(leaf)
            return {'leaf':len(leaves)-1}
        left = visit(node['left'], ancestors+[node], annotation)
        right = visit(node['right'], ancestors+[node], annotation)
        node['_index'] = len(nodes)
        nodes.append({'op':OPS[node['op']], 'left':left, 'right':right,
                      'value':str(node['value']), 'annotation':annotation})
        return {'kind':'step', 'index':node['_index']}
    for i, tree in enumerate(trees):
        visit(tree, [], i)
    for leaf in leaves:
        leaf['ancestors'] = [n['_index'] for n in leaf.pop('_ancestors')]
    return nodes, leaves

def annotations(answer):
    if answer.count('####') != 1:
        raise Reject('malformed_final_answer')
    rationale, final = answer.split('####')
    if '<<' in final or '>>' in final:
        raise Reject('malformed_annotation')
    blocks = list(re.finditer(r'<<([^<>]*)>>', rationale))
    remainder = re.sub(r'<<([^<>]*)>>', '', rationale)
    if '<<' in remainder or '>>' in remainder:
        raise Reject('malformed_annotation')
    if not blocks:
        raise Reject('missing_annotation')
    if len(blocks) > 64:
        raise Reject('annotation_bound')
    values = []
    for b in blocks:
        if b.group(1).count('=') != 1:
            raise Reject('malformed_annotation')
        expression, label = b.group(1).split('=')
        values.append({'expression':expression.strip(), 'label':label.strip(),
                       'answer_char_span':[b.start(), b.end()]})
    return values, final.strip()

def verify_spans(question, binding):
    if binding['status'] != 'bound':
        return
    b = question.encode()
    last = 0
    for n in binding['numbers']:
        assert last <= n['start'] < n['end'] <= len(b)
        assert b[n['start']:n['end']].decode() == n['literal']
        assert scalar(n['literal'].removeprefix('+')) == native_value(n['value'])
        last = n['end']

def convert(row, binding):
    q, answer = row['question'], row['answer']
    rec = {'annotation_present':'<<' in answer, 'annotation_parses':False,
           'arithmetic_valid':None, 'final_consistent':None,
           'exact_native_binder_accepted':binding['status']=='bound',
           'uniquely_grounded_typed_program':False, 'ambiguity':None,
           'input_count':len(binding['numbers']) if binding['status']=='bound' else None,
           'step_count':None, 'operators':[], 'reasons':[], 'accepted':False}
    def reason(r):
        if r not in rec['reasons']:
            rec['reasons'].append(r)
    if binding['status'] != 'bound':
        reason('native_binder_refusal')
        rec['native_binder_reason'] = binding['reason']
    else:
        verify_spans(q, binding)
    try:
        anns, final_text = annotations(answer)
        rec['annotations'] = anns
    except Reject as e:
        reason(str(e)); return rec
    trees = []
    for ann in anns:
        try:
            trees.append(Parser(ann['expression']).parse())
            scalar(ann['label'])
        except Reject as e:
            reason(str(e))
    if len(trees) != len(anns) or any(r in rec['reasons'] for r in
            ('unsupported_scalar','number_too_long','rational_overflow')):
        return rec
    # A scalar parse may fail after its tree was appended.
    if any(r != 'native_binder_refusal' for r in rec['reasons']):
        return rec
    rec['annotation_parses'] = True
    rec['operators'] = [op for tree in trees for op in shape(tree)]
    rec['step_count'] = len(rec['operators'])
    if rec['step_count'] > 128:
        reason('row_expression_bound'); return rec
    if rec['step_count'] > 6:
        reason('overlong_program')
    arithmetic_ok = True
    evaluable = True
    for tree, ann in zip(trees, anns):
        try:
            value = evaluate(tree)
            ann['computed'] = str(value)
            if value != scalar(ann['label']):
                reason('wrong_annotated_value'); arithmetic_ok = False
        except Reject as e:
            reason(str(e)); arithmetic_ok = False; evaluable = False
            ann['error'] = str(e)
    rec['arithmetic_valid'] = arithmetic_ok
    try:
        final = scalar(final_text)
        rec['final_answer'] = str(final)
        if 'value' in trees[-1]:
            rec['final_consistent'] = trees[-1]['value'] == final
            if not rec['final_consistent']:
                reason('wrong_final_value')
    except Reject as e:
        reason('malformed_final_answer'); rec['final_parse_error'] = str(e)
    if not evaluable:
        return rec
    nodes, leaves = flatten(trees)
    rec['expression_nodes'] = nodes
    if binding['status'] != 'bound':
        return rec
    inputs = [native_value(n['value']) for n in binding['numbers']]
    prior = [native_value(n['value']) for n in nodes]
    constants = [scalar(x) for x in CONSTANTS]
    product = 1
    rec['ambiguity'] = False
    for leaf in leaves:
        value = native_value(leaf['value'])
        alternatives = []
        for kind, inventory in [('input', inputs), ('step',prior[:leaf['available_before']]),
                                ('constant', constants)]:
            alternatives.extend({'kind':kind,'index':i} for i,v in enumerate(inventory) if v==value)
        leaf['alternatives'] = alternatives
        product *= len(alternatives)
        if len(alternatives) > 1:
            reason('ambiguous_leaf'); rec['ambiguity'] = True
        elif not alternatives:
            reason('unbound_leaf')
            future = [i for i,v in enumerate(prior) if i >= leaf['available_before'] and v==value]
            leaf['ineligible_equal_nodes'] = future
            if any(i in leaf['ancestors'] for i in future):
                reason('cyclic_reference')
            if any(i not in leaf['ancestors'] for i in future):
                reason('impossible_forward_reference')
    rec['leaves'] = leaves
    rec['candidate_program_count'] = str(product)
    # Retain complete candidate sets even for wrong labels/finals and overlong graphs.
    if rec['reasons']:
        return rec
    def argument(ref):
        if 'leaf' in ref:
            choices = leaves[ref['leaf']]['alternatives']
            assert len(choices) == 1
            return choices[0]
        return ref
    rec['program'] = {'steps':[{'op':n['op'], 'left':argument(n['left']),
                               'right':argument(n['right'])} for n in nodes]}
    rec['uniquely_grounded_typed_program'] = True
    return rec

def native(exe, requests, directory, label):
    inp, out, err = (directory/(label+s) for s in ('.requests.jsonl','.outputs.jsonl','.stderr'))
    jsonl(inp, requests)
    with inp.open('rb') as fi, out.open('wb') as fo, err.open('wb') as fe:
        run = subprocess.run([str(exe)], stdin=fi, stdout=fo, stderr=fe, timeout=120)
    if run.returncode:
        raise RuntimeError(f'Original native executor failed ({run.returncode}); see {err}')
    outputs = [json.loads(line) for line in out.read_text().splitlines()]
    assert len(outputs) == len(requests)
    return outputs

def execute_unique(exe, rows, bindings, records, directory):
    ids = [i for i,r in enumerate(records) if r['uniquely_grounded_typed_program']]
    requests = [{'question':rows[i]['question'],'program':records[i]['program']} for i in ids]
    outputs = native(exe, requests, directory, 'gold-provided-execution')
    jsonl(directory/'gold-provided-execution.row-ids.jsonl', [{'request_index':j,'row_id':i} for j,i in enumerate(ids)])
    for i, out in zip(ids, outputs):
        rec = records[i]
        rec['native_gold_execution'] = out
        good = (out['status']=='proposed' and native_value(out['value'])==native_value(rec['final_answer'])
                and out['numbers']==bindings[i]['numbers'] and out['program']==rec['program'])
        rec['accepted'] = good
        if not good:
            rec['reasons'].append('native_gold_execution_mismatch_or_refusal')

def verify_source(source):
    manifest = json.loads((source/'manifest.json').read_text())
    assert manifest['source']=='openai/grade-school-math'
    assert manifest['revision']=='3101c7d5072418e28b9008a6636bde82a006892c'
    assert manifest['official_split']=='train' and manifest['rows']==7473
    assert manifest['license']=='MIT' and manifest['test_downloaded'] is False
    verified = []
    for f in manifest['files']+manifest['reused_native_files']:
        assert Path(f['file']).name == f['file']
        b = (source/f['file']).read_bytes()
        assert sha(b)==f['sha256'] and len(b)==f['bytes'], f['file']
        verified.append({'file':f['file'],'bytes':len(b),'sha256':sha(b)})
    verified.append({'file':'manifest.json','bytes':(source/'manifest.json').stat().st_size,
                     'sha256':sha((source/'manifest.json').read_bytes())})
    assert next(f for f in verified if f['file']=='train.jsonl')['sha256']=='17f347dc51477c50d4efb83959dbb7c56297aba886e5544ee2aaed3024813465'
    assert next(f for f in verified if f['file']=='equation_executor')['sha256']=='40e6a6e975cf8474231ac0e3bcc2e3df5c1ce8196dead0b9159586aa5935ea6f'
    return {'metadata':manifest,'verified':verified}

def membership(row, i, raw):
    normalized = re.sub(r'[0-9]+','#',' '.join(row['question'].lower().split()))
    group = sha(normalized.encode())
    return {'row_id':i,'raw_line_sha256':sha(raw),'question_sha256':sha(row['question'].encode()),
            'answer_sha256':sha(row['answer'].encode()),'normalized_question':normalized,
            'group_sha256':group, 'subset':'TRAIN-CAL' if int(group,16)%5==0 else 'TRAIN-FIT'}

def summarize(records):
    flags = ['annotation_present','annotation_parses','arithmetic_valid','final_consistent',
             'exact_native_binder_accepted','uniquely_grounded_typed_program','ambiguity','accepted']
    def group(rs):
        return {'all_rows':len(rs), 'stages':{f:{'true':sum(r[f] is True for r in rs),
                      'false':sum(r[f] is False for r in rs),
                      'not_evaluated':sum(r[f] is None for r in rs)} for f in flags},
                'exclusion_reasons':dict(sorted(Counter(x for r in rs for x in r['reasons']).items())),
                'native_binder_reasons':dict(sorted(Counter(r['native_binder_reason'] for r in rs if 'native_binder_reason' in r).items()))}
    report = {'all':group(records)}
    for field in ['subset','step_count','input_count']:
        vals = sorted(set(str(r[field]) for r in records))
        report['by_'+field] = {v:group([r for r in records if str(r[field])==v]) for v in vals}
    report['by_operator_presence'] = {op:group([r for r in records if op in r['operators']]) for op in OPS.values()}
    report['operator_node_counts'] = dict(Counter(op for r in records for op in r['operators']))
    report['accepted_operator_node_counts'] = dict(Counter(op for r in records if r['accepted'] for op in r['operators']))
    report['gate'] = {'required_unique':1000,'unique_exact_programs':sum(r['accepted'] for r in records),
                      'coverage_pass':sum(r['accepted'] for r in records)>=1000,
                      'independent_contracts':'unavailable; pending coordinator',
                      'semantics':'unverified; converted TRAIN gold labels, not predictions'}
    return report

def contracts(args, source_info):
    cases = json.loads((Path(__file__).parent/'contracts.json').read_text())
    bindings = native(args.source/'equation_executor',[{'question':x['question'],'program':None} for x in cases],args.output,'question-only-binding')
    records = [convert(x,b) for x,b in zip(cases,bindings)]
    execute_unique(args.source/'equation_executor',cases,bindings,records,args.output)
    checks = []
    for case, rec, binding in zip(cases,records,bindings):
        errors = []
        if rec['accepted'] != case.get('accepted',False): errors.append('acceptance')
        if 'reason' in case and case['reason'] not in rec['reasons']: errors.append('reason')
        if 'choices' in case and not any(len(x['alternatives'])==case['choices'] for x in rec.get('leaves',[])): errors.append('alternatives')
        if 'first_start' in case and binding['numbers'][0]['start']!=case['first_start']: errors.append('UTF8 span')
        checks.append({'id':case['id'],'passed':not errors,'errors':errors,'record':rec})
    dump(args.output/'contracts-results.json',checks)
    dump(args.output/'source-verification.json',source_info)
    print(json.dumps({'contracts':len(checks),'passed':sum(x['passed'] for x in checks),'failed':[x['id'] for x in checks if not x['passed']]}))
    return all(x['passed'] for x in checks)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--mode',choices=['contracts','sample','full'],required=True)
    args = p.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    info = verify_source(args.source)
    if args.mode=='contracts':
        if not contracts(args,info): raise SystemExit(1)
        return
    raw_lines = (args.source/'train.jsonl').read_bytes().splitlines(keepends=True)
    assert len(raw_lines)==7473
    if args.mode=='sample': raw_lines=raw_lines[:200]
    rows = [json.loads(raw) for raw in raw_lines]
    members = [membership(r,i,raw) for i,(r,raw) in enumerate(zip(rows,raw_lines))]
    jsonl(args.output/'membership.jsonl',members)
    bindings = native(args.source/'equation_executor',[{'question':r['question'],'program':None} for r in rows],args.output,'question-only-binding')
    records = []
    for row,binding,member in zip(rows,bindings,members):
        rec = convert(row,binding)
        rec.update({k:member[k] for k in ['row_id','subset','group_sha256','question_sha256','answer_sha256']})
        records.append(rec)
    execute_unique(args.source/'equation_executor',rows,bindings,records,args.output)
    jsonl(args.output/'records.jsonl',records)
    report = summarize(records)
    report['normalized_groups']=len(set(m['group_sha256'] for m in members))
    report['source']=info
    dump(args.output/'report.json',report)
    assert verify_source(args.source)==info
    print(json.dumps({'mode':args.mode,'all':report['all'],'gate':report['gate']},sort_keys=True))

if __name__=='__main__':
    main()
