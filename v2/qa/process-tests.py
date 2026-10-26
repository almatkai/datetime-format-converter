"""Real JSONL pipes, actual SDK runner and production executable. No OS effects."""
import subprocess,pathlib,json,uuid,selectors,time,sys,os
v2=pathlib.Path(__file__).resolve().parents[1]
ID='com.tezbar.native.datetime-format-converter'
CAPS=['clipboard.read','clipboard.write','clipboard.pasteText','settings.open','window.visibility']
PRODUCERS=json.loads((v2/'.qa/host-producers.json').read_text())
class Host:
 def __init__(self,binary=None):
  self.root=v2/'.qa/process'/uuid.uuid4().hex;self.root.mkdir(parents=True)
  env={'PATH':os.environ['PATH'],'TZ':'UTC','HOME':str(self.root),'TMPDIR':str(self.root)}
  self.p=subprocess.Popen([str(binary or os.environ.get('QA_BINARY') or v2/'target/debug/datetime-format-converter')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,text=True,bufsize=1)
  self.selector=selectors.DefaultSelector();self.selector.register(self.p.stdout,selectors.EVENT_READ);self.frames=[];self.buffer=b'';self.calls=[];self.readreply='2024-11-01T01:56:27Z';self.ack=None
  hello=self.read();assert hello['type']=='hello' and hello['id']==ID and hello['protocol']==1 and hello['sdkVersion']=='0.1.1',hello
  r=self.request('initialize',dict(protocol=1,extensionId=ID,platform='macOS',packageDir=str(v2),dataDir=str(self.root/'data'),tempDir=str(self.root/'tmp'),grantedCapabilities=CAPS,migrationPaths={}))
  assert r['result']['ready'] is True
 def send(self,v):self.p.stdin.write(json.dumps(v)+'\n');self.p.stdin.flush()
 def read(self):
  while b'\n' not in self.buffer:
   if not self.selector.select(5):raise AssertionError('JSONL timeout')
   chunk=os.read(self.p.stdout.fileno(),65536);assert chunk,'EOF '+self.p.stderr.read();self.buffer+=chunk
  line,self.buffer=self.buffer.split(b'\n',1);v=json.loads(line);self.frames.append(v);return v
 def begin(self,m,p):
  id='h:'+str(uuid.uuid4());self.send(dict(type='request',id=id,method=m,params=p));return id
 def reply(self,f,value=None,error=None):
  r=dict(type='response',id=f['id']);r['error' if error else 'result']=error if error else value;self.send(r)
 def wait(self,id):
  while True:
   f=self.read()
   if f['type']=='request':
    self.calls.append(f)
    if f['method']=='host.clipboard.read':self.reply(f,self.readreply)
    else:
     assert f['method'] in CAPS_HOST,f
     self.reply(f,PRODUCERS[f['method']] if self.ack is None else self.ack)
   elif f['type']=='response' and f['id']==id:return f
 def request(self,m,p):return self.wait(self.begin(m,p))
 def open(self,s='s',cmd='index',args=None,prefs=None,search=None):
  p=dict(sessionId=s,commandName=cmd,argumentValues=args or {},preferences={'autoPasteClipboard':False,**(prefs or {})})
  if search is not None:p['searchText']=search
  return self.request('session.open',p)
 def call(self,s,op,payload=None):return self.request('session.call',dict(sessionId=s,operation='extension:'+op,payload=payload or {}))
 def action(self,view,index=0,fields=None):return {'actionId':view['actions'][index]['id'],'formValues':fields if fields is not None else self.fields(view)}
 def fields(self,view):return dict(__viewToken=view['root']['props']['__draftGuard'],__draftDirty=False,__searchText=view['root']['props']['searchText'])
 def dispose(self,s='s'):assert 'result'in self.request('session.dispose',{'sessionId':s})
 def finish(self):
  self.p.stdin.close();self.p.wait(timeout=5);assert self.p.returncode==0,self.p.stderr.read();self.selector.close()
CAPS_HOST=['host.clipboard.write','host.clipboard.pasteText','host.settings.open','host.window.visibility']
checks=[]
def check(name,fn):fn();checks.append(name)
def workflow():
 h=Host();v=h.open(search='2024-11-01T01:56:27Z')['result'];assert v['mode']=='view' and len(v['root']['children'])==14
 old=h.action(v);b=h.call('s','search-text-changed',{'searchText':'2024-02-29'})['result'];a=h.call('s','search-text-changed',{'searchText':'2024-11-01T01:56:27Z'})['result'];assert 'error'in h.call('s','invoke-action',old)
 for fields in [None,{},dict(h.fields(a),__draftDirty=True),dict(h.fields(a),__searchText='B')]:
  p=h.action(a);p['formValues']=fields;assert 'error'in h.call('s','invoke-action',p)
 r=h.call('s','invoke-action',h.action(a));assert r['result']['message']=='Copied to clipboard' and r['result']['mode']=='no-view';assert h.calls[-2]['params']=={'text':'2024-11-01'};assert h.calls[-1]['params']=={'visible':False}
 assert 'error'in h.call('s','invoke-action',h.action(a))
 h.dispose();v=h.open(search='2024-02-29')['result'];r=h.call('s','invoke-action',h.action(v,1));assert r['result']['mode']=='no-view';assert h.calls[-1]['method']=='host.clipboard.pasteText';assert h.calls[-1]['params']=={'text':'2024-02-29'}
 h.dispose();h.finish()
check('search A-B-A, current draft, copy/paste, consumed actions',workflow)
def prefs():
 h=Host();v=h.open(prefs={k:False for k in ['isoDate','isoDateTime','formatWithMillis','formatWithTimezone','isoFormat','utcIsoFormat','unixTimestamp','unixMillis','localizedShortDate','localizedShortDateTime','localizedFullDateTime','localizedLongDateTime','localizedTime','localizedSecondsTime']},search='now')['result'];assert len(v['actions'])==1
 h.call('s','invoke-action',h.action(v));assert h.calls[-1]['params']=={'surface':'settings-extensions'}
 v=h.call('s','search-text-changed',{'searchText':'bad date'})['result'];assert not v['actions'];h.dispose();h.finish()
check('preferences defaults/order, empty result settings, invalid parse editing',prefs)
def natural():
 h=Host();r=h.open(cmd='insert-natural-language-date',args={'prompt':'July 4, 2025 at 3pm','format':'YYYY-MM-DD HH:mm'})['result'];assert r['mode']=='no-view';assert h.calls[-1]['params']=={'text':'2025-07-04 15:00'};assert len(h.calls)==1;h.dispose()
 for i,args in enumerate([{'prompt':'not a date'},{'prompt':'today','format':'YYYY[x]'},{'prompt':'not a date','format':'bad'}]):
  r=h.open(str(i),'insert-natural-language-date',args)['result'];assert r['mode']=='no-view';h.dispose(str(i))
 assert len(h.calls)==1;h.finish()
check('natural command automatically pastes; parse/format failure order no effects',natural)
def load():
 h=Host();h.readreply=None;v=h.open(prefs={'autoPasteClipboard':True})['result'];assert v['mode']=='view' and len(v['actions'])==1;h.readreply='2024-02-29';v=h.call('s','invoke-action',h.action(v))['result'];assert len(v['root']['children'])==14;v=h.call('s','search-text-changed',{'searchText':'nonsense'})['result'];assert v['root']['props']['searchText']=='nonsense';v=h.call('s','search-text-changed',{'searchText':'2024-01-01'})['result'];assert len(v['root']['children'])==14;h.dispose();h.finish()
check('clipboard read exact STRING and load/parse recovery',load)
def bad_acks():
 h=Host()
 for i,ack in enumerate([True,None,{}, {'ok':False},{'ok':True,'error':'contradiction'},{'ok':True,'message':None},{'ok':True,'extra':True}]):
  v=h.open(str(i),search='2024-11-01T01:56:27Z')['result'];p=h.action(v,1);h.ack=ack
  # None normally means producer default; send it manually for this case.
  id=h.begin('session.call',dict(sessionId=str(i),operation='extension:invoke-action',payload=p));f=h.read();assert f['method']=='host.clipboard.pasteText';prepared=f['params']['text'];h.reply(f,ack);r=h.wait(id)['result'];assert r['mode']=='view' and 'unknown'in r['message'];assert 'May Repeat'in r['actions'][0]['title']
  assert 'error'in h.call(str(i),'invoke-action',p)
  h.ack=None;r2=h.call(str(i),'invoke-action',h.action(r))['result'];assert r2['mode']=='no-view' and h.calls[-1]['params']['text']==prepared;h.dispose(str(i))
 h.finish()
check('closed method-specific acknowledgements, explicit exact-byte retry',bad_acks)
def cancel_effect():
 h=Host();v=h.open(search='2024-11-01T01:56:27Z')['result'];old=h.action(v,1);id=h.begin('session.call',dict(sessionId='s',operation='extension:invoke-action',payload=old));effect=h.read();assert effect['type']=='request';prepared=effect['params'];h.send({'type':'cancel','requestId':id,'reason':'synthetic cancellation'});r=h.wait(id);assert 'error'in r;h.reply(effect,PRODUCERS['host.clipboard.pasteText']);v=h.call('s','refresh-session')['result'];assert 'unknown'in v['message'];r=h.call('s','invoke-action',h.action(v));assert r['result']['mode']=='no-view' and h.calls[-1]['params']==prepared;h.dispose();h.finish()
check('cancel and late ACK preserve exact prepared bytes without auto-replay',cancel_effect)
def canceled_opens():
 h=Host()
 for i in range(140):
  s='c'+str(i);id=h.begin('session.open',dict(sessionId=s,commandName='index',preferences={'autoPasteClipboard':True},argumentValues={}))
  f=h.read();assert f['method']=='host.clipboard.read';h.send(dict(type='cancel',requestId=id,reason='synthetic canceled open'));assert 'error'in h.wait(id);h.reply(f,'2024-01-01')
 v=h.open('final',search='2024-01-01');assert v['result']['mode']=='view';h.dispose('final');h.finish()
check('140 canceled opens release adapter and SDK capacity',canceled_opens)
def saturation():
 h=Host()
 for i in range(128):assert 'result'in h.open(str(i))
 assert 'error'in h.open('overflow')
 for i in range(128):h.dispose(str(i))
 assert 'result'in h.open('recovered');h.dispose('recovered');h.finish()
check('bounded 128-session capacity and recovery',saturation)
def eof_pending():
 h=Host();id=h.begin('session.open',dict(sessionId='eof',commandName='insert-natural-language-date',preferences={},argumentValues={'prompt':'July 4, 2025'}));f=h.read();assert f['method']=='host.clipboard.pasteText';h.finish()
check('EOF cancels pending reverse effect and exits',eof_pending)

def copy_hide_checkpoint():
 h=Host();v=h.open(search='2024-11-01')['result'];id=h.begin('session.call',dict(sessionId='s',operation='extension:invoke-action',payload=h.action(v)))
 f=h.read();assert f['method']=='host.clipboard.write';h.reply(f,PRODUCERS['host.clipboard.write']);f=h.read();assert f['method']=='host.window.visibility' and f['params']=={'visible':False};h.reply(f,True)
 recovery=h.wait(id)['result'];assert recovery['mode']=='view';start=len(h.calls);r=h.call('s','invoke-action',h.action(recovery));assert r['result']['mode']=='no-view';assert [c['method'] for c in h.calls[start:]]==['host.window.visibility'];h.dispose();h.finish()
check('copy closes window with NULL ACK; unknown hide retries no clipboard rewrite',copy_hide_checkpoint)

(v2/'evidence'/os.environ.get('QA_REPORT','process-checks.json')).write_text(json.dumps({'checks':checks,'producerFixture':'tests/host_extracts.rs','productionExecutable':str(os.environ.get('QA_BINARY') or v2/'target/debug/datetime-format-converter')},indent=2)+'\n')
print(len(checks),'real SDK process scenarios passed')
