"""Finite isolated Codex teacher using the existing ChatGPT login, never API keys."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
GLOBAL=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/span-expansion')
assert len(sys.argv)==3 and sys.argv[2] in ('01','02','03','04')
BATCH=sys.argv[2]
ROOT=GLOBAL/f'batch-{BATCH}'
HERE=Path(__file__).resolve().parent.parent/'span-references'
T=ROOT/'teacher';WORK=T/'work';AUTH=Path('/home/gsc/.codex/auth.json')
UNIT=f'glassmind-span-expansion-{BATCH}'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()


def environment():
    env={k:v for k,v in os.environ.items() if not any(term in k.upper() for term in ['OPENAI','ANTHROPIC','CLAUDE','GEMINI','API_KEY','GH_TOKEN','GITHUB_TOKEN','AWS_'])}
    env.update(CODEX_HOME=str(T/'codex'),TMPDIR=str(T/'tmp'))
    return env


def isolated():
    cmd=['bwrap','--die-with-parent','--ro-bind','/','/','--dev','/dev','--proc','/proc']
    for p in ['/home/gsc/code','/home/gsc/.cache/glassmind','/home/gsc/.codex','/home/gsc/.claude','/home/gsc/.ssh','/home/gsc/.config/gh']:
        if Path(p).exists():cmd+=['--tmpfs',p]
    if Path('/home/gsc/.claude.json').exists():cmd+=['--ro-bind','/dev/null','/home/gsc/.claude.json']
    for p in [WORK,T/'codex',T/'tmp']:cmd+=['--bind',str(p),str(p)]
    cmd+=['--ro-bind',str(AUTH),str(AUTH),'--bind',str(T/'tmp'),'/tmp']
    for name in ['batch.json','schema.json','validate.py','teacher-brief.txt']:
        p=WORK/name;cmd+=['--ro-bind',str(p),str(p)]
    return cmd+['--chdir',str(WORK),'--']


def prepare():
    ROOT.mkdir(exist_ok=False)
    (ROOT/'source').mkdir()
    (ROOT/'source/batch.json').write_bytes((GLOBAL/f'source/batch-{BATCH}/batch.json').read_bytes())
    T.mkdir(exist_ok=False)
    for p in [WORK,T/'codex',T/'tmp']:p.mkdir()
    (T/'codex/auth.json').symlink_to(AUTH)
    copies={'batch.json':ROOT/'source/batch.json',**{n:HERE/n for n in ['schema.json','validate.py','teacher-brief.txt']}}
    for name,p in copies.items():
        dst=WORK/name;dst.write_bytes(p.read_bytes());dst.chmod(0o444)
    hidden=[GLOBAL/'parent/expectations.json',GLOBAL/'source/native-outputs.jsonl',GLOBAL.parent.parent/'groundedsteps-source/train.jsonl',Path('/home/gsc/code/glassmind'),Path('/home/gsc/code/rustbench')]
    code='from pathlib import Path\nimport json,os\n'+repr(None)+'\n'
    code+='hidden='+repr([str(p) for p in hidden])+'\nassert all(not Path(p).exists() for p in hidden)\n'
    code+='assert Path("batch.json").is_file()\nassert not any("API_KEY" in k or k.startswith(("OPENAI","ANTHROPIC","CLAUDE","GEMINI")) for k in os.environ)\n'
    code+='print(json.dumps({"masked_paths":len(hidden),"selected_batch_visible":True,"key_environment_removed":True}))\n'
    p=subprocess.run(isolated()+['python','-c',code],env=environment(),capture_output=True,text=True,timeout=20,check=True)
    (T/'isolation-check.json').write_text(p.stdout)
    login=subprocess.run(isolated()+['codex','--no-daemon','login','status'],env=environment(),capture_output=True,text=True,timeout=20,check=True)
    status=login.stdout+login.stderr;assert status.strip()=='Logged in using ChatGPT';(T/'login-status.txt').write_text(status)
    proof=dict(model='gpt-6-astra',service_tier='fast',forced_login_method='chatgpt',reasoning='high',seconds_cap=1200,memory_max=8*1024**3,memory_swap_max=0,annotation_bytes_cap=2*1024**2,teacher_directory_bytes_cap=256*1024**2,official_configuration='https://learn.chatgpt.com/docs/config-file/config-reference',inputs={name:sha(WORK/name) for name in copies},script_sha256=sha(Path(__file__)))
    (T/'launch.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(dict(prepared=True,login='ChatGPT',masked_paths=len(hidden),model=proof['model'],service_tier='fast')))


def run():
    proof=json.loads((T/'launch.json').read_text());assert sha(Path(__file__))==proof['script_sha256']
    for name,digest in proof['inputs'].items():assert sha(WORK/name)==digest
    assert not (WORK/'annotations.json').exists()
    cli=['codex','--no-daemon','exec','--ignore-user-config','--skip-git-repo-check','--ephemeral','--model','gpt-6-astra','-c','service_tier="fast"','-c','forced_login_method="chatgpt"','-c','model_reasoning_effort="high"','-c','log_dir="'+str(T/'tmp')+'"','-c','sqlite_home="'+str(T/'tmp')+'"','--dangerously-bypass-approvals-and-sandbox','--json','-o',str(WORK/'summary.txt'),'-']
    command=['systemd-run','--user','--scope','--quiet','--collect','--unit='+UNIT,'-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','1200',*isolated(),*cli]
    started=time.monotonic();failed=None;peak=0
    def footprint():
        total=0
        for path in T.rglob('*'):
            if path.is_symlink():continue
            try:
                stat=path.stat()
                if path.is_file():total+=max(stat.st_size,512*stat.st_blocks)
            except FileNotFoundError:pass
        return total
    def stop(signum=None,frame=None):
        subprocess.run(['systemctl','--user','stop',UNIT+'.scope'],capture_output=True,timeout=30)
        if signum:raise RuntimeError('teacher supervisor interrupted')
    signal.signal(signal.SIGTERM,stop);signal.signal(signal.SIGINT,stop)
    with (WORK/'teacher-brief.txt').open() as brief,(T/'events.jsonl').open('x') as events,(T/'stderr.log').open('x') as errors:
        process=subprocess.Popen(command,env=environment(),stdin=brief,stdout=events,stderr=errors,start_new_session=True)
        (T/'process.json').write_text(json.dumps(dict(pid=process.pid,supervisor_pid=os.getpid(),scope=UNIT+'.scope',command=command,started_unix=time.time()))+'\n')
        try:
            while process.poll() is None:
                output=WORK/'annotations.json'
                peak=max(peak,footprint())
                if peak>256*1024**2 or (output.exists() and output.stat().st_size>2*1024**2) or time.monotonic()-started>=1210:
                    failed='annotation size, teacher-directory footprint or process time cap';stop();break
                time.sleep(2)
            code=process.wait(timeout=30)
        finally:
            if process.poll() is None:stop();process.wait(timeout=30)
    peak=max(peak,footprint())
    if peak>256*1024**2 or ((WORK/'annotations.json').exists() and (WORK/'annotations.json').stat().st_size>2*1024**2):failed='terminal output or teacher-directory footprint cap'
    report=dict(sampled_peak_teacher_bytes=peak,returncode=code,seconds=time.monotonic()-started,limit_failure=failed,worker_exited_cleanly=code==0 and failed is None,annotations_exist=(WORK/'annotations.json').is_file(),semantic_approval=False)
    (T/'completion.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
    if code or failed:raise SystemExit(1)


if __name__=='__main__':
    assert sys.argv[1] in ('prepare','run')
    (prepare if sys.argv[1]=='prepare' else run)()
