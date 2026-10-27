import json,pathlib,subprocess,collections,sys
v2=pathlib.Path(__file__).resolve().parents[1]
rows=json.loads((v2/'evidence/source-vectors.json').read_text())
p=subprocess.run([str(v2/'target/debug/domain-oracle')],input=''.join(json.dumps(r)+'\n' for r in rows),text=True,capture_output=True)
if p.returncode:print(p.stderr);sys.exit(p.returncode)
actual=[json.loads(l) for l in p.stdout.splitlines()];assert len(rows)==len(actual),(len(rows),len(actual))
diffs=[dict(input={k:v for k,v in r.items() if k!='expected'},expected=r['expected'],actual=a) for r,a in zip(rows,actual) if r['expected']!=a]
(v2/'.qa/diffs.json').write_text(json.dumps(diffs,indent=2)+'\n')
print(len(rows)-len(diffs),'/',len(rows),'byte-exact vectors')
for d in diffs[:35]:print(d['input']['kind'],d['input']['tz'],repr(d['input']['text']), 'expected',d['expected'],'actual',d['actual'])
sys.exit(bool(diffs))
