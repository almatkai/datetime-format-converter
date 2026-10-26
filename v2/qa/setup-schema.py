import pathlib,subprocess,os
v2=pathlib.Path(__file__).resolve().parents[1];d=v2/'.qa/schema';d.mkdir(parents=True,exist_ok=True)
for src,dst in [('schema-package.json','package.json'),('schema-package-lock.json','package-lock.json')]:
 (d/dst).write_bytes((v2/'qa'/src).read_bytes())
subprocess.run(['npm','ci','--ignore-scripts','--no-audit','--no-fund','--cache',str(v2/'.qa/npm-cache')],cwd=d,check=True)
