"""Builds the main result chart and the readability numbers from a loop state.

    python3 tools/report.py --state ~/.cache/glassmind/loop/state --out docs/results

Writes `chart.html` (two panels sharing the tokens axis: share of turns
answered right for training, validation and held-out; code size), and
`summary.json` (the timeline, rules by provenance, how often rules are used,
and size against coverage). Reads only the loop's history and baseline.
"""

import argparse
import html
import json
from collections import Counter
from pathlib import Path


def timeline(history):
    """One point per history entry: the accepted state after it."""
    points, current = [], None
    for e in history:
        if e["accepted"]:
            current = e["scores"]
        if current is None:
            continue
        s = current
        pct = lambda r, t: round(100.0 * r / t, 2) if t else None
        points.append({
            "round": e["round"],
            "kind": e["kind"],
            "group": e["group"],
            "accepted": e["accepted"],
            "tokens_m": round(e["tokens_cumulative"] / 1e6, 2),
            "train": pct(s["train_right"], s["train_turns"]),
            "valid": pct(s["valid_right"], s["valid_turns"]),
            "heldout": pct(s["heldout_right"], s["heldout_turns"]),
            "train_turns": s["train_turns"],
            "size": s["size"],
            "rules": s.get("rules", 0),
        })
    return points


def readability(baseline, rules_tsv):
    """Rules by provenance, and how many right answers use each rule."""
    rounds = {}
    for line in rules_tsv.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2 and parts[1].isdigit():
            rounds[parts[0]] = int(parts[1])
    uses = Counter()
    for o in baseline.get("train", []):
        for r in set(o.get("rules", [])):
            uses[r] += 1
    hand = 2**32 - 1
    prov = Counter("seed" if r == 0 else "by hand" if r == hand else "loop" for r in rounds.values())
    buckets = Counter()
    for rid in rounds:
        n = uses.get(rid, 0)
        b = "0" if n == 0 else "1" if n == 1 else "2-9" if n < 10 else "10-99" if n < 100 else "100+"
        buckets[b] += 1
    return {
        "rules": len(rounds),
        "by_provenance": dict(prov),
        "uses_per_rule": {k: buckets.get(k, 0) for k in ["0", "1", "2-9", "10-99", "100+"]},
        "most_used": uses.most_common(10),
    }


PAGE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>glassmind loop results</title>
<style>
:root{--surface:#fcfcfb;--ink:#0b0b0b;--ink2:#52514e;--grid:#e6e5e1;--s1:#2a78d6;--s2:#eb6834;--s3:#1baf7a;--size:#52514e;--mark:#b9b8b2}
@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--surface:#1a1a19;--ink:#ffffff;--ink2:#c3c2b7;--grid:#383835;--s1:#3987e5;--s2:#d95926;--s3:#199e70;--size:#c3c2b7;--mark:#5c5b56}}
:root[data-theme="dark"]{--surface:#1a1a19;--ink:#ffffff;--ink2:#c3c2b7;--grid:#383835;--s1:#3987e5;--s2:#d95926;--s3:#199e70;--size:#c3c2b7;--mark:#5c5b56}
body{margin:0;background:var(--surface);color:var(--ink);font:14px/1.45 system-ui,sans-serif}
main{max-width:980px;margin:0 auto;padding:16px}
h1{font-size:20px;margin:8px 0 4px}h2{font-size:15px;margin:20px 0 6px}
p{color:var(--ink2);margin:4px 0 10px}
.legend{display:flex;gap:16px;flex-wrap:wrap;color:var(--ink2);font-size:13px;margin:6px 0}
.legend i{display:inline-block;width:14px;height:3px;border-radius:2px;margin-right:6px;vertical-align:middle}
svg{width:100%;height:auto;display:block;overflow:visible}
.axis text{fill:var(--ink2);font-size:11px}.grid line{stroke:var(--grid)}
.tip{position:fixed;pointer-events:none;background:var(--surface);color:var(--ink);border:1px solid var(--grid);border-radius:6px;padding:6px 8px;font-size:12px;box-shadow:0 2px 8px rgba(0,0,0,.15);display:none;max-width:320px}
details{margin-top:16px}table{border-collapse:collapse;font-size:12px;width:100%}td,th{padding:3px 6px;border-bottom:1px solid var(--grid);text-align:right}td:first-child,th:first-child{text-align:left}
.wrap{overflow-x:auto}
</style></head><body><main>
<h1>glassmind: does spending more on the loop keep improving it?</h1>
<p>Share of turns answered exactly right, against tokens spent by the loop's agents. Training and validation phrasings come from the training generators; held-out phrasings come from a separate generator, about repositories never seen in training. All requests were written by Claude Opus 5.5; none by a person.</p>
<div class="legend"><span><i style="background:var(--s1)"></i>training</span><span><i style="background:var(--s2)"></i>held-out</span><span><i style="background:var(--s3)"></i>validation (training styles, unseen repos)</span><span><i style="background:var(--mark)"></i>training data added / changed by hand</span></div>
<svg id="acc" viewBox="0 0 940 330" role="img" aria-label="Share of turns right over tokens spent"></svg>
<h2>Code size (syntax tokens of the mind crates)</h2>
<svg id="size" viewBox="0 0 940 200" role="img" aria-label="Code size over tokens spent"></svg>
<details><summary>Table view</summary><div class="wrap"><table id="tbl"></table></div></details>
</main><div class="tip" id="tip"></div>
<script>
const P = __POINTS__;
const W=940, L=56, R=24, xmax=Math.max(...P.map(p=>p.tokens_m))||1;
const x=t=>L+(W-L-R)*t/xmax;
const tip=document.getElementById('tip');
function step(vals, y){let d='',last=null;P.forEach((p,i)=>{const v=vals(p);if(v==null)return;const px=x(p.tokens_m),py=y(v);d+= last==null?`M${px},${py}`:` L${px},${last} L${px},${py}`;last=py;});return d;}
function panel(id,H,top,bottom,ymax,ticks,fmt,series){
  const svg=document.getElementById(id); const y=v=>top+(H-top-bottom)*(1-v/ymax);
  let s='<g class="grid">'+ticks.map(t=>`<line x1="${L}" x2="${W-R}" y1="${y(t)}" y2="${y(t)}"/>`).join('')+'</g>';
  s+='<g class="axis">'+ticks.map(t=>`<text x="${L-6}" y="${y(t)+4}" text-anchor="end">${fmt(t)}</text>`).join('');
  for(let i=0;i<=5;i++){const t=xmax*i/5;s+=`<text x="${x(t)}" y="${H-bottom+16}" text-anchor="middle">${t.toFixed(0)}M</text>`;}
  s+=`<text x="${(L+W-R)/2}" y="${H-4}" text-anchor="middle">agent tokens spent (millions)</text></g>`;
  P.forEach(p=>{if(p.kind==='data'||p.kind==='manual'){s+=`<line x1="${x(p.tokens_m)}" x2="${x(p.tokens_m)}" y1="${top}" y2="${H-bottom}" stroke="var(--mark)" stroke-dasharray="3 3"/>`;}});
  const used=[];
  series.forEach(([key,col,label])=>{s+=`<path d="${step(p=>p[key],y)}" fill="none" stroke="var(${col})" stroke-width="2"/>`;
    const lastP=[...P].reverse().find(p=>p[key]!=null); if(!label||!lastP) return;
    let ly=y(lastP[key])-6; while(used.some(u=>Math.abs(u-ly)<13)) ly+=13; used.push(ly);
    s+=`<text x="${x(lastP.tokens_m)-4}" y="${ly}" text-anchor="end" font-size="11" fill="var(--ink)">${label} ${fmt(lastP[key])}</text>`;});
  s+=`<line id="${id}-x" y1="${top}" y2="${H-bottom}" stroke="var(--ink2)" stroke-width="1" visibility="hidden"/>`;
  s+=`<rect x="${L}" y="${top}" width="${W-L-R}" height="${H-top-bottom}" fill="transparent" id="${id}-hit"/>`;
  svg.innerHTML=s;
  const hit=document.getElementById(id+'-hit'), cross=document.getElementById(id+'-x');
  hit.addEventListener('mousemove',ev=>{const r=svg.getBoundingClientRect();const t=((ev.clientX-r.left)*W/r.width-L)/(W-L-R)*xmax;
    let best=P[0];P.forEach(p=>{if(p.tokens_m<=t+1e-9)best=p;});cross.setAttribute('x1',x(best.tokens_m));cross.setAttribute('x2',x(best.tokens_m));cross.setAttribute('visibility','visible');
    tip.style.display='block';tip.style.left=(ev.clientX+12)+'px';tip.style.top=(ev.clientY+12)+'px';
    tip.innerHTML=`<b>round ${best.round}</b> · ${best.tokens_m}M tokens<br>training ${best.train}% · held-out ${best.heldout}% · validation ${best.valid}%<br>size ${best.size.toLocaleString()}<br><span style="color:var(--ink2)">${best.kind}: ${best.group.replace(/[<>&]/g,'')}</span>`;});
  hit.addEventListener('mouseleave',()=>{tip.style.display='none';cross.setAttribute('visibility','hidden');});
}
panel('acc',330,12,40,100,[0,25,50,75,100],v=>v+'%',[['train','--s1','training'],['heldout','--s2','held-out'],['valid','--s3','validation']]);
const smax=Math.ceil(Math.max(...P.map(p=>p.size))/10000)*10000;
panel('size',200,12,40,smax,[0,smax/2,smax],v=>(v/1000).toFixed(0)+'k',[['size','--size','']]);
document.getElementById('tbl').innerHTML='<tr><th>round</th><th>kind</th><th>tokens (M)</th><th>training %</th><th>validation %</th><th>held-out %</th><th>size</th></tr>'+
  P.filter((p,i)=>i===P.length-1||P[i+1].round!==p.round||p.kind==='data'||p.kind==='manual').map(p=>`<tr><td>${p.round}</td><td>${p.kind}</td><td>${p.tokens_m}</td><td>${p.train}</td><td>${p.valid}</td><td>${p.heldout}</td><td>${p.size}</td></tr>`).join('');
</script></body></html>
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--state", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--rules", help="output of `glassmind rules` for the final version")
    a = ap.parse_args()
    state = Path(a.state).expanduser()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    history = [json.loads(l) for l in open(state / "history.jsonl") if l.strip()]
    points = timeline(history)
    baseline = json.load(open(state / "baseline.json"))
    rules_tsv = Path(a.rules).read_text() if a.rules else ""
    summary = {
        "points": points,
        "readability": readability(baseline, rules_tsv),
        "first": points[0] if points else None,
        "last": points[-1] if points else None,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=1))
    (out / "chart.html").write_text(PAGE.replace("__POINTS__", json.dumps(points)))
    print(json.dumps({k: summary[k] for k in ("readability", "last")}, indent=1))


if __name__ == "__main__":
    main()
