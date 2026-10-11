#!/usr/bin/env python3
"""Report raw observations by protocol, without relabelling frozen failures."""
import collections, hashlib, json, pathlib, sys
root=pathlib.Path(sys.argv[1]);rows=json.loads((root/'results.json').read_text());groups={}
for r in rows:
 key=r['expected'].split('_')[0]+'/'+r['mode']
 g=groups.setdefault(key,collections.Counter())
 g['rows']+=1;g['changed']+=r['source_before']!=r['source_after']
 g['exact_expected_repair']+=r['source_after']==r['source_before'].replace('7u8','7u32')
 g['native_before_failed']+=r['native_before'] is not None and r['native_before']['exit']!=0
 g['native_after_failed']+=r['native_after'] is not None and r['native_after']['exit']!=0
 g['frozen_mutation_expectation_failed']+=not r['mutation_ok']
 g['frozen_native_expectation_failed']+=not r['native_ok']
# All fixtures have a single authored Rust source; preserve the manifest too.
manifests=[]
for w in sorted(root.glob('fixture-*')):
 expected='[package]\nname="repair_probe_'+hashlib.sha256(str(w).encode()).hexdigest()[:16]+'"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
 manifests.append({'path':str(w),'manifest_unchanged':(w/'Cargo.toml').read_text()==expected,'only_authored_source':sorted(str(p.relative_to(w)) for p in (w/'src').rglob('*') if p.is_file())==['src/lib.rs']})
print(json.dumps({'groups':groups,'fixture_manifests':manifests},indent=2))
