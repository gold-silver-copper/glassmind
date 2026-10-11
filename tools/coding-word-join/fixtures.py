"""Authored before candidate edits; no benchmark answers in runtime proposals."""
import json
from pathlib import Path


def source(body,tests,target='transform',sig='text: &str',output='String'):
    return f'pub fn {target}({sig}) -> {output} {{ {body} }}\n#[cfg(test)] mod tests {{ use super::*; #[test] fn contract() {{ {tests} }} }}\n'


def checks(pairs,target='transform'):
    return ' '.join(f'assert_eq!({target}({json.dumps(a,ensure_ascii=False)}), {json.dumps(b,ensure_ascii=False)});' for a,b in pairs)


rows=[]
for name,pairs in [
    ('normalize', [('  cerise\tbleu\nvert ', 'cerise bleu vert'), ('東京\u2003été\u00a0🙂!', '東京 été 🙂!'), ('x x\ty','x x y'), (' \t\n',''), (' solo ','solo')]),
    ('reverse', [('  cerise\tbleu\nvert ', 'vert bleu cerise'), ('東京\u2003été\u00a0🙂!', '🙂! été 東京'), ('x x\ty','y x x'), (' \t\n',''), (' solo ','solo')]),
    ('chars', [('ab🚀', '🚀ba'), ('été 東京', '京東 été'), ('a\tb\n', '\nb\ta'), ('','')]),
    ('compact', [(' north\tsouth\nwest ', 'northsouthwest'), ('東京 été', '東京été'), ('','')]),
    ('compact-reverse', [(' north\tsouth\nwest ', 'westsouthnorth'), ('東京 été', 'été東京'), ('','')]),
]:
    rows.append(dict(id=name,code=source('todo!()',checks(pairs)),text='implement transform',baseline=name not in ('normalize','reverse'),candidate=True))
# The previous family finds one survivor, but two distinct programs in the new
# family agree on these weak tests: accepting either would conceal ambiguity.
rows.append(dict(id='new-ambiguity',code=source('todo!()',checks([('a b','b a'),('','')])),text='implement transform',baseline=True,candidate=False))
rows.append(dict(id='old-ambiguity',code=source('todo!()',checks([('a','a')])),text='implement transform',baseline=False,candidate=False))
rows.append(dict(id='already-implemented',code=source('text.to_string()',checks([('a b','a b')])),text='implement transform',baseline=False,candidate=False))
rows.append(dict(id='unexercised',code=source('todo!()','assert_eq!(2+2,4);'),text='implement transform',baseline=False,candidate=False))
rows.append(dict(id='two-inputs',code=source('todo!()','assert_eq!(transform("red","blue"),"red blue");',sig='text: &str, other: &str'),text='implement transform',baseline=False,candidate=False))
Path(__file__).with_name('native.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
