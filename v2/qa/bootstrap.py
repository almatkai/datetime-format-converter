"""Install exact V1 QA dependencies into ignored clone-local output only."""
import pathlib,subprocess,json,os
v2=pathlib.Path(__file__).resolve().parents[1];repo=v2.parent;pin='6633da332d70a4a34a2b317309ef61f2c0e4d74d';dest=v2/'.qa/v1';dest.mkdir(parents=True,exist_ok=True)
for f in ['package.json','package-lock.json','tsconfig.json','raycast-env.d.ts','src/index.tsx','src/insert-natural-language-date.tsx','test/smoke.test.ts']:
 p=dest/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(subprocess.check_output(['git','-C',str(repo),'show',pin+':'+f]))
env=dict(os.environ,HOME=str(v2/'.qa/home'),TMPDIR=str(v2/'.qa/tmp'),npm_config_cache=str(v2/'.qa/npm-cache'))
for name in ['home','tmp']:(v2/'.qa'/name).mkdir(exist_ok=True)
subprocess.run(['npm','ci','--ignore-scripts','--no-audit','--no-fund'],cwd=dest,env=env,check=True)
print('Exact pinned V1 QA dependencies prepared under v2/.qa/v1')
