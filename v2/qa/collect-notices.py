"""Copy authentic dependency notices (Cargo's lock/metadata, no invented grants)."""
import pathlib,json,subprocess,re,os
v2=pathlib.Path(__file__).resolve().parents[1]
metadata=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--locked','--manifest-path',str(v2/'Cargo.toml')],text=True))
checks={}
for block in (v2/'Cargo.lock').read_text().split('[[package]]')[1:]:
 fields=dict(re.findall(r'^(name|version|checksum) = "([^"]+)"',block,re.M));checks[(fields['name'],fields['version'])]=fields.get('checksum')
report=[]
for p in metadata['packages']:
 if not p.get('source','') or not p['source'].startswith('registry+'):continue
 root=pathlib.Path(p['manifest_path']).parent;files=[]
 for f in root.rglob('*'):
  if f.is_file() and not any(x in f.relative_to(root).parts for x in ['tests','test','benches','examples','.git']) and f.name.upper().startswith(('LICENSE','COPYING','NOTICE')):
   rel=f.relative_to(root);dest=v2/'notices/rust'/(p['name']+'-'+p['version'])/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(f.read_bytes());files.append(str(dest.relative_to(v2)))
 report.append({'name':p['name'],'version':p['version'],'licenseExpression':p.get('license'),'source':p['source'],'checksum':checks[(p['name'],p['version'])],'noticeFiles':files,'missingText':not files})
(v2/'evidence/rust-notices.json').write_text(json.dumps(report,indent=2)+'\n')
print(len(report),'locked Rust package notices;',sum(r['missingText'] for r in report),'missing texts explicitly recorded')
