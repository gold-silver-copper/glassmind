#!/usr/bin/env python3
"""Supplemental post-implementation safety cases; not part of frozen gains."""
import evaluate
base=evaluate.CASES[0]
cases=[]
for label,extra,replace in [
 ('duplicate-owner','mod other { pub struct Parcel { pub count: u32 } }\n',None),
 ('import-alias','use Parcel as Alias;\n',('Parcel { count: 3 }.count','Alias { count: 3 }.count')),
 ('deref','struct Wrapper(Parcel); impl std::ops::Deref for Wrapper { type Target = Parcel; fn deref(&self) -> &Parcel { &self.0 } }\n',('Parcel { count: 3 }.count','Wrapper(Parcel { count: 3 }).count')),
 ('inferred-local','fn read_local() -> u32 { let x = Parcel { count: 3 }; x.count }\n',('Parcel { count: 3 }.count','read_local()')),
]:
 c=dict(base);source=base['source']
 if replace: source=source.replace(*replace)
 source+=extra
 c.update(id=label,source=source.replace('OWNER_BEHAVIOR_direct',f'OWNER_BEHAVIOR_{label}'),supported=False)
 c['expected']=c['source'];cases.append(c)
evaluate.CASES=cases
evaluate.main()
