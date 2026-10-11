#!/usr/bin/env python3
"""Append benchmark totals and render a chart against elapsed hours and model tokens.

Only aggregate scores enter the journal; never input examples or held-out answers.
Each point records its split, metric, dataset version and evaluated code commit.
Unmeasured tracks stay explicitly unmeasured, rather than becoming zeroes.
"""
import argparse
import html
import json
import subprocess
import time
from pathlib import Path

TRACKS = ('coding', 'stories', 'intents', 'dialogue', 'facts', 'numbers',
          'grammar', 'reading', 'chat', 'code_qa')
COLORS = ('#1565c0', '#c62828', '#2e7d32', '#6a1b9a', '#e65100',
          '#00838f', '#ad1457', '#4527a0', '#558b2f', '#795548')


def chart(rows, axis):
    width, height = 900, 280
    left, top, bottom = 50, 15, 35
    xmax = max((r[axis] for r in rows), default=1) or 1
    plot_height = height - top - bottom
    def xy(row):
        return (left + row[axis] / xmax * (width-left-20),
                top + (1-row['right']/row['total']) * plot_height)
    out = [f'<svg viewBox="0 0 {width} {height}" role="img" aria-label="Accuracy against {axis}">']
    for pct in (0, 25, 50, 75, 100):
        y = top + (1-pct/100)*plot_height
        out.append(f'<path d="M{left} {y}H880" stroke="#bbb" opacity=".4"/><text x="5" y="{y+4}">{pct}%</text>')
    for step in range(6):
        x = left + step/5*(width-left-20)
        value = xmax*step/5
        label = f'{value:.2f}' if axis == 'hours' else f'{value/1e6:.2f}M'
        out.append(f'<text x="{x}" y="270" text-anchor="middle">{label}</text>')
    for track, color in zip(TRACKS, COLORS):
        for split in ('dev', 'heldout'):
            # Dataset and metric changes start a new series.
            groups = {}
            for row in rows:
                if row['track'] == track and row['split'] == split:
                    groups.setdefault((row['dataset'], row['metric']), []).append(row)
            for points in groups.values():
                coords = ' '.join(f'{x:.2f},{y:.2f}' for x,y in map(xy, points))
                dash = ' stroke-dasharray="5 4"' if split == 'heldout' else ''
                out.append(f'<polyline points="{coords}" fill="none" stroke="{color}" stroke-width="2"{dash}/>')
                for row in points:
                    x,y = xy(row)
                    label = html.escape(f"{track} {split}: {row['right']}/{row['total']}, wrong {row.get('wrong', 'n/a')}; {row['dataset']}; {row['commit']}")
                    out.append(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="4" fill="{color}"><title>{label}</title></circle>')
    return ''.join(out) + '</svg>'


def render(journal, out):
    rows = [json.loads(line) for line in journal.read_text().splitlines() if line.strip()] if journal.exists() else []
    rows.sort(key=lambda r: r['unix_time'])
    start = min((r['unix_time'] for r in rows), default=time.time())
    for row in rows:
        row['hours'] = (row['unix_time']-start)/3600
    legend = ' '.join(f'<span style="color:{color}">{track}</span>' for track,color in zip(TRACKS,COLORS))
    table = []
    for track in TRACKS:
        for split in ('dev','heldout'):
            points = [r for r in rows if r['track']==track and r['split']==split]
            if points:
                r = points[-1]
                score = f"{r['right']}/{r['total']} ({100*r['right']/r['total']:.1f}%)"
                detail = f"wrong {r.get('wrong','n/a')}; refused {r.get('refused','n/a')}; {r['dataset']}; {r['metric']}"
            else:
                score, detail = 'unmeasured', ''
            table.append('<tr>'+''.join(f'<td>{html.escape(v)}</td>' for v in (track,split,score,detail))+'</tr>')
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_text('''<!doctype html><meta charset="utf-8"><title>glassmind benchmark tracks</title>
<style>body{font:15px system-ui;max-width:1050px;margin:24px auto;padding:0 16px;color:#222;background:#fafafa}svg{width:100%}svg text{font-size:12px;fill:#555}span{margin-right:16px}td,th{padding:6px 10px;border-bottom:1px solid #ddd;text-align:left}table{border-collapse:collapse;width:100%}</style>
<h1>glassmind benchmark tracks</h1><p>Development scores steer work; held-out totals only check it. Solid lines: dev. Dashed: held-out. Each dataset and metric has its own series. Hours start at the first recorded measurement; tokens count completed Codex worker turns and include cached input. Parent usage and interrupted calls are excluded, so token counts are a lower bound. Reading measures coverage; the grammar probe uses parse preference, not probabilities. Synthetic chat checks are not an independent benchmark.</p>
''' + '<p>'+legend+'</p><h2>Elapsed hours</h2>'+chart(rows,'hours')+'<h2>Model tokens</h2>'+chart(rows,'tokens')+'<table><tr><th>Track</th><th>Split</th><th>Right / covered</th><th>Measure</th></tr>'+''.join(table)+'</table>')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--journal',type=Path,default=Path('docs/results/tracks.jsonl'))
    parser.add_argument('--out',type=Path,default=Path('docs/results/tracks.html'))
    subs=parser.add_subparsers(dest='command',required=True)
    add=subs.add_parser('record')
    add.add_argument('--track',choices=TRACKS,required=True)
    add.add_argument('--split',choices=('dev','heldout'),required=True)
    add.add_argument('--dataset',required=True)
    add.add_argument('--metric',default='exact_accuracy')
    for name in ('right','total','tokens'):
        add.add_argument('--'+name,type=int,required=True)
    for name in ('wrong','refused'):
        add.add_argument('--'+name,type=int)
    add.add_argument('--unix-time',type=float,default=None)
    add.add_argument('--commit',default=None)
    subs.add_parser('render')
    args=parser.parse_args()
    if args.command=='record':
        row={k:v for k,v in vars(args).items() if k not in ('journal','out','command') and v is not None}
        if args.total<=0 or not 0<=args.right<=args.total or args.tokens<0:
            parser.error('invalid counts')
        if any(v is not None and v<0 for v in (args.wrong,args.refused)):
            parser.error('negative verdict count')
        if args.wrong is not None and args.refused is not None and args.right+args.wrong+args.refused!=args.total:
            parser.error('verdict counts must sum to total')
        row['unix_time']=args.unix_time if args.unix_time is not None else time.time()
        row['commit']=args.commit or subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
        args.journal.parent.mkdir(parents=True,exist_ok=True)
        with args.journal.open('a') as f:f.write(json.dumps(row,sort_keys=True)+'\n')
    render(args.journal,args.out)

if __name__=='__main__':
    main()
