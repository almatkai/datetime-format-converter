"""Hash EVERY Git-visible file except this inventory; no staging or mutation outside v2."""
import pathlib,subprocess,hashlib,json
v2=pathlib.Path(__file__).resolve().parents[1];repo=v2.parent
files=subprocess.check_output(['git','-C',str(repo),'ls-files','--cached','--others','--exclude-standard','-z']).decode().split('\0')
files=sorted(set(f for f in files if f and f!='v2/INDEX.json'))
rows={f:hashlib.sha256((repo/f).read_bytes()).hexdigest() for f in files if (repo/f).is_file()}
(v2/'INDEX.json').write_text(json.dumps({'scope':'all Git-visible files; excludes v2/INDEX.json itself','sha256':rows},indent=2)+'\n')
expected=json.loads((v2/'evidence/v1-hashes.json').read_text())
for f,h in expected.items():
 if f!='README.md':assert hashlib.sha256((repo/f).read_bytes()).hexdigest()==h,f
old=subprocess.check_output(['git','-C',str(repo),'show','6633da332d70a4a34a2b317309ef61f2c0e4d74d:README.md'])
assert (repo/'README.md').read_bytes()==old+b'\n[Standalone native Rust V2 (feature-branch review)](v2/README.md).\n'
assert not subprocess.check_output(['git','-C',str(repo),'diff','--cached','--name-only'])
assert subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip()=='6633da332d70a4a34a2b317309ef61f2c0e4d74d'
print(len(rows),'Git-visible files inventoried; index empty, HEAD unchanged, V1 byte boundaries passed')
