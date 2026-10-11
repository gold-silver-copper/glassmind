#!/usr/bin/env python3
"""Compile the public typing adapter against optimized workspace libraries.
Source defaults to current coding.rs; --baseline uses git HEAD, never private data.
"""
import argparse,os,pathlib,subprocess
p=argparse.ArgumentParser();p.add_argument('run');p.add_argument('--baseline',action='store_true');p.add_argument('--revision',default='6a346c3080a5dd94d7a6bc7dae32af7f6d969798');a=p.parse_args()
r=pathlib.Path(a.run).resolve();r.mkdir(parents=True,exist_ok=True)
current=pathlib.Path('crates/cli/src/coding.rs').read_text()
source=subprocess.check_output(['git','show',a.revision+':crates/cli/src/coding.rs'],text=True) if a.baseline else current
runtime=source.split('#[cfg(test)]')[0].replace('../models/coding.tsv',str(pathlib.Path('crates/cli/models/coding.tsv').resolve()))
adapter=current[current.index('    // Optional public development adapter'):]
source=runtime+'\n#[cfg(test)]\nmod adapter {\nuse super::*;\n'+adapter
(r/'typing.rs').write_text(source)
deps=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release/deps'
cmd=['rustc','--edition=2024','--test','-O',str(r/'typing.rs'),'-L','dependency='+str(deps),'-o',str(r/'typing')]
for name in ['glassmind_english','glassmind_english_rust','glassmind_meaning','glassmind_models','glassmind_rules','glassmind_world_rust','glassmind_grammar','serde_json']:
 libs=list(deps.glob('lib'+name+'-*.rlib'));assert len(libs)==1,(name,libs)
 cmd+=['--extern',name+'='+str(libs[0])]
subprocess.run(cmd,check=True)
subprocess.run([str(r/'typing'),'--test-threads','4'],check=True)
