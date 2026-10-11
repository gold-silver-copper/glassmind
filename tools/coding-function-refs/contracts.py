#!/usr/bin/env python3
"""Supplemental preview, rollback and unchanged frontend contracts."""
import argparse, json, os, pathlib, shutil, subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--inputs',required=True);p.add_argument('--out',required=True);a=p.parse_args()
binary=str(pathlib.Path(a.binary).resolve()); inputs=pathlib.Path(a.inputs).resolve();out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=True)
def run(cmd,root,log):
 r=subprocess.run(cmd,cwd=root,text=True,capture_output=True,timeout=650);(out/log).write_text(r.stdout+r.stderr);return r

def snap(root):return {str(f.relative_to(root)):f.read_bytes() for f in root.rglob('*') if f.is_file() and f.name!='Cargo.lock'}
provenance=json.loads((inputs/'provenance.json').read_text());results={}
root=out/'preview';shutil.copytree(inputs/'workspace',root);before=snap(root)
r=run([binary,'ask','-s',str(root),'--json',provenance['canonical_request']],root,'preview.json')
j=json.loads(r.stdout);changes=j['response']['changes'];results['preview_exact_nine']=len(changes)==9
results['preview_unchanged']=snap(root)==before
for c in changes:
 line=before[c['file']].decode().splitlines()[c['line']-1];assert line[c['column']:c['column']+len(c['old'])]==c['old']=='parse_line';assert c['new']=='parse_entry'
for row in provenance['requests']:
 name=row['id'].split('/')[-1].split('#')[0];root=out/name;shutil.copytree(inputs/'workspace',root);before=snap(root)
 r=run([binary,'do',str(root),'--explain',row['text']],root,name+'.log')
 results[name+'_unchanged']=snap(root)==before
 results[name+'_route']=('Command(Repair(Test))' in r.stdout) if name=='chatty' else ('UnknownShape' in r.stdout)
root=out/'rollback';(root/'src').mkdir(parents=True)
(root/'Cargo.toml').write_text("[package]\nname='identity-proof'\nversion='0.0.0'\nedition='2024'\n[workspace]\n")
(root/'src/lib.rs').write_text('pub fn parse_line()->u32{7}\n#[test] fn identity(){ assert!(std::any::type_name_of_val(&parse_line).ends_with("::parse_line")); }\n')
before=snap(root)
scope=['systemd-run','--user','--scope','--quiet','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
r=run(scope+['cargo','test','-j1','--','--test-threads','4'],root,'rollback-native-before.log');results['rollback_baseline_pass']=r.returncode==0
r=run([binary,'ask','-s',str(root),'--json',provenance['canonical_request']],root,'rollback-preview.json');results['rollback_has_proposal']=json.loads(r.stdout)['response']['type']=='edit'
r=run([binary,'ask','-s',str(root),'--json','--apply',provenance['canonical_request']],root,'rollback-apply.json');results['rollback_refused']=r.returncode!=0 and json.loads(r.stdout)['response']['type']=='refuse'
results['rollback_exact_originals']=snap(root)==before
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2));assert all(results.values())
