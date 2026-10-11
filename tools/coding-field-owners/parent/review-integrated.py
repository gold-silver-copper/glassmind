from pathlib import Path
import hashlib,os,shutil,sys
sys.path.insert(0, '/home/gsc/code/glassmind/tools/coding-field-owners-review')
import probe
out=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review/independent')
original=probe.HERE
out.mkdir(parents=True,exist_ok=True)
shutil.copy2(original/'frozen.json',out/'frozen.json')
probe.HERE=out
probe.RUN=out/'runtime'
os.environ['CARGO_TARGET_DIR']=str(probe.RUN/'target')
probe.BINARIES['accepted']=out.parent/'glassmind-integrated'
probe.HASHES['accepted']='8e52835c22bbb335a10fa4f3c9b175f1e7db07b0f5b895803adbcb35a8cc431d'
probe.run('accepted',True,'parent')
probe.run('accepted',False,'parent')
import supplement
shutil.copy2(original/'supplemental/frozen.json',probe.HERE/'frozen.json')
probe.run('accepted',False,'parent')
