"""Audit executable ELF sections after a test-only edit changed debug identity."""
import hashlib,json,struct
from pathlib import Path
root=Path('../socialrouting-run/provenance')
def sections(path):
 b=path.read_bytes();assert b[:6]==b'\x7fELF\x02\x01'
 off=struct.unpack_from('<Q',b,40)[0];size,n,strings=struct.unpack_from('<HHH',b,58)
 headers=[struct.unpack_from('<IIQQQQIIQQ',b,off+i*size) for i in range(n)]
 st=headers[strings];names=b[st[4]:st[4]+st[5]];out={}
 for h in headers:
  name=names[h[0]:].split(b'\0',1)[0].decode()
  if h[2]&2 and name!='.note.gnu.build-id':
   data=b[h[4]:h[4]+h[5]] if h[1]!=8 else b''
   out[name]=dict(type=h[1],flags=h[2],address=h[3],size=h[5],alignment=h[8],sha256=hashlib.sha256(data).hexdigest())
 return out
a=sections(root/'evaluated-glassmind');b=sections(root/'final-glassmind');diff=[k for k in a.keys()|b.keys() if a.get(k)!=b.get(k)]
r=dict(equivalent=not diff,ignored='.note.gnu.build-id and nonallocated debug/symbol/section metadata',changed_allocated_sections=diff,evaluated=a,final=b)
Path('tools/conversation/social-routing/reports/binary-equivalence.json').write_text(json.dumps(r,indent=2)+'\n');print('equivalent',not diff,'sections',len(a),'differences',diff);assert not diff
