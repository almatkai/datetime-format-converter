"""Build an ignored, closed-inventory DEVELOPMENT ZIP from the isolated binary."""
import pathlib,json,hashlib,zipfile,os,subprocess,shutil
v2=pathlib.Path(__file__).resolve().parents[1];binary=v2/'.qa/isolated/target/release/datetime-format-converter'
assert binary.is_file(),'Run isolated HTTPS SDK build first'
manifest=json.loads((v2/'extension.json').read_text());assert manifest['artifacts']=={}
files={'bin/datetime-format-converter':binary.read_bytes(),'LICENSE':(v2/'notices/OWN-LICENSE').read_bytes(),'NOTICE':(v2/'NOTICE').read_bytes()}
for folder in ['resources','notices']:
 for f in (v2/folder).rglob('*'):
  if f.is_file():assert not f.is_symlink();files[f.relative_to(v2).as_posix()]=f.read_bytes()
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest['files']={p:sha(b) for p,b in sorted(files.items())};files['extension.json']=(json.dumps(manifest,indent=2)+'\n').encode()
dest=v2/'dist';dest.mkdir(exist_ok=True);archive=dest/'datetime-format-converter-2.0.0-aarch64-apple-darwin-dev.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED)as z:
 for name,data in sorted(files.items()):
  i=zipfile.ZipInfo(name,(1980,1,1,0,0,0));i.create_system=3;i.external_attr=((0o100755 if name.startswith('bin/') else 0o100644)<<16);i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,data)
extract=v2/'.qa/extracted-development';extract.mkdir(exist_ok=True)
with zipfile.ZipFile(archive)as z:
 assert sorted(z.namelist())==sorted(files)
 for name in z.namelist():
  assert name in files and not name.startswith('/') and '..'not in pathlib.PurePosixPath(name).parts
  data=z.read(name);assert sha(data)==sha(files[name]);p=extract/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data);p.chmod(0o755 if name.startswith('bin/') else 0o644)
actual=extract/'bin/datetime-format-converter'
report={'developmentOnly':True,'artifacts':{},'archive':str(archive.relative_to(v2)),'archiveSha256':sha(archive.read_bytes()),'executableSha256':sha(actual.read_bytes()),'files':manifest['files'],'archiveMembers':sorted(files),'architecture':subprocess.check_output(['file',str(actual)],text=True).strip(),'sourceCommit':'6633da332d70a4a34a2b317309ef61f2c0e4d74d','sdkCommit':'e955911f0bf5b558f9d3eb5183df0adc95c22af2'}
(v2/'evidence/development-package.json').write_text(json.dumps(report,indent=2)+'\n')
env=dict(os.environ,QA_BINARY=str(actual),QA_REPORT='extracted-process-checks.json');subprocess.run(['python3',str(v2/'qa/process-tests.py')],env=env,check=True)
print('Closed-inventory native ARM64 development archive verified; extracted production executable passed real SDK pipes')
