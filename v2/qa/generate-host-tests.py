import pathlib,json
v2=pathlib.Path(__file__).resolve().parents[1];before=v2/'qa/host';after=v2/'.qa/host-after'
def function(s,name):
 start=s.index('fn '+name);start=s.rfind('\n',0,start)+1
 i=s.index('{',start);depth=1;j=i+1;string=False;esc=False
 while depth:
  c=s[j]
  if string:
   if esc:esc=False
   elif c=='\\':esc=True
   elif c=='"':string=False
  elif c=='"':string=True
  elif c=='{':depth+=1
  elif c=='}':depth-=1
  j+=1
 return s[start:j]
def arm(s,needle):
 start=s.index(needle);i=s.index('{',start);d=1;j=i+1
 while d:
  if s[j]=='{':d+=1
  elif s[j]=='}':d-=1
  j+=1
 return s[start:j]
r='src-tauri/src/native_core/extensions/registry.rs';old=(before/r).read_text();new=(after/r).read_text()
common='''use std::time::Duration;use serde_json::{json,Value};use std::path::{Path,PathBuf,Component};
#[derive(Clone)]struct Installed{id:String,slug:String,path:PathBuf,resource_path:PathBuf,manifest:Value}
'''
mods=[]
for label,s in [('before',old),('after',new)]:
 code='\n'.join(function(s,n).replace('pub(super) ','') for n in ['fallback','icon_path','command_icon_path'])
 mods.append('mod '+label+' {use super::*;'+code+'\npub(super) fn icon(e:&Installed)->Option<String>{icon_path(e)} pub(super) fn cmd(e:&Installed,c:&Value)->Option<String>{command_icon_path(e,c)}}')
alias=function(new,'verified_alias').replace('pub(super) ','').replace('super::process_install::resolve','resolve')
mods.append('''#[derive(Default)]struct CoreContext{entries:Vec<Installed>,verified:std::collections::HashMap<String,Value>}
fn collect(ctx:&CoreContext)->Vec<Installed>{ctx.entries.clone()}
fn compatible(v:&Value,_command:bool)->bool{v["platforms"].as_array().is_some_and(|a|a.iter().any(|v|v=="macOS"))}
struct Package{manifest:Value}
fn resolve(ctx:&CoreContext,id:&str)->Result<Package,String>{ctx.verified.get(id).cloned().map(|manifest|Package{manifest}).ok_or("unverified".into())}
'''+alias)
tests='''
fn fixture()->Installed {let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".qa/host-resources").join(uuid::Uuid::new_v4().to_string());let active=root.join("versions/2.0/resources");std::fs::create_dir_all(&active).unwrap();std::fs::write(active.join("command-icon.png"),b"synthetic").unwrap();Installed{id:"com.tezbar.native.fixture".into(),slug:"fixture".into(),resource_path:root.join("versions/2.0"),path:root,manifest:json!({"icon":"resources/command-icon.png"})}}
#[test]fn actual_resource_functions_before_after(){let e=fixture();let root=e.path.clone();assert!(before::icon(&e).is_none());assert!(before::cmd(&e,&e.manifest).is_none());assert_eq!(after::icon(&e),Some(e.resource_path.join("resources/command-icon.png").to_string_lossy().into()));assert!(after::cmd(&e,&e.manifest).is_some());assert_eq!(e.path,root);assert!(after::cmd(&e,&json!({"icon":"../secret"})).is_none());}
#[test]fn verified_exact_covered_aliases(){let mut ctx=CoreContext::default();let mut e=fixture();e.manifest=json!({"replaces":["com.tezbar.fixture","raycast.fixture"],"commands":[{"name":"run","runtime":"rust","platforms":["macOS"]}]});ctx.entries.push(e.clone());assert!(verified_alias(&ctx,"com.tezbar.fixture","run").unwrap().is_none());ctx.verified.insert(e.id.clone(),e.manifest.clone());for id in ["com.tezbar.fixture","raycast.fixture"]{assert_eq!(verified_alias(&ctx,id,"run").unwrap(),Some(e.id.clone()));assert!(verified_alias(&ctx,id,"uncovered").unwrap().is_none());}for id in ["COM.TEZBAR.fixture","com.tezbar.fixture "," com.tezbar.fixture","raycast.fixture.evil"]{assert!(verified_alias(&ctx,id,"run").unwrap().is_none());}let mut e2=e.clone();e2.id="com.tezbar.native.other".into();ctx.verified.insert(e2.id.clone(),e2.manifest.clone());ctx.entries.push(e2);assert!(verified_alias(&ctx,"com.tezbar.fixture","run").is_err());}
'''
# Function bodies below are extracted from the actual clipboard/platform producers.
clipboard=(before/'src-tauri/src/native_core/launcher/clipboard.rs').read_text();platform=(before/'src-tauri/src/native_core/platform.rs').read_text()
read=arm(clipboard,'"clipboard:read" => {');write=arm(clipboard,'"clipboard:write" => {')
producers='''
#[allow(dead_code)]mod arboard{#[derive(Debug)]pub enum Error{ContentNotAvailable,Other}impl std::fmt::Display for Error{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"fixture error")}}}
struct Native{value:String}impl Native{fn get_text(&self)->Result<String,arboard::Error>{Ok(self.value.clone())}}
struct ClipboardFixture{native:Native}impl ClipboardFixture{fn native(&mut self)->Result<&mut Native,String>{Ok(&mut self.native)}fn write_text(&mut self,text:String)->Result<(),String>{self.native.value=text;Ok(())}fn handle(&mut self,c:&str,a:&[Value])->Result<Value,String>{match c{'''+read+','+write+''',_=>Err("blocked".into())}}}
fn text_arg(a:&[Value],i:usize)->Result<&str,String>{a.get(i).and_then(Value::as_str).ok_or("text required".into())}
#[derive(Clone)]struct App;impl App{fn hide(&self)->Result<(),String>{Ok(())}}
fn set_backend_app_visibility(_app:&App,_value:bool){}
fn open_settings_window(_app:App)->Result<(),String>{Ok(())}
mod native_input{pub fn press_key(_key:String,_modifiers:Vec<String>)->Result<(),String>{Ok(())}}
fn settings_surface(text:&str)->Option<&str>{Some(text)}
struct Ctx;impl Ctx{async fn request(&self,_c:&str,_a:Vec<Value>)->Result<Value,String>{Ok(json!({"ok":true}))}fn emit(&self,_event:&str,_value:Value){}}
struct PlatformFixture{ctx:Ctx,app:App}impl PlatformFixture{fn app(&self)->Result<&App,String>{Ok(&self.app)}async fn handle(&self,channel:&str,value:Value)->Result<Value,String>{let text=value.as_str().unwrap_or("");match channel{
'''+','.join(arm(platform,n) for n in ['"core:platform:visibility" => {','"core:platform:open-settings" => {','"core:platform:paste" | "core:platform:paste-text" => {'])+''',_=>Err("blocked".into())}}}
#[tokio::test]async fn actual_producer_default_shapes(){let mut c=ClipboardFixture{native:Native{value:"synthetic date".into()}};let p=PlatformFixture{ctx:Ctx,app:App};let values=json!({"host.clipboard.read":c.handle("clipboard:read",&[]).unwrap(),"host.clipboard.write":c.handle("clipboard:write",&[json!("synthetic")]).unwrap(),"host.clipboard.pasteText":p.handle("core:platform:paste-text",json!("synthetic")).await.unwrap(),"host.window.visibility":p.handle("core:platform:visibility",json!(false)).await.unwrap(),"host.settings.open":p.handle("core:platform:open-settings",json!("settings-extensions")).await.unwrap()});let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".qa/host-producers.json");std::fs::create_dir_all(path.parent().unwrap()).unwrap();std::fs::write(path,serde_json::to_vec_pretty(&values).unwrap()).unwrap();for method in ["host.clipboard.write","host.clipboard.pasteText","host.settings.open","host.window.visibility"]{assert!(datetime_format_converter_v2::adapter::ack(method,&values[method]).is_ok());}}
'''
(v2/'tests/host_extracts.rs').write_text('// GENERATED by qa/generate-host-tests.py; exact pinned host function/arm extracts.\n// Extracted host bodies preserve upstream returns and closure spelling.\n#![allow(clippy::needless_return, clippy::redundant_closure, clippy::nonminimal_bool)]\n'+common+'\n'.join(mods)+tests+producers)
# Actual package validation + launcher validation (no app build or test substitute).
install=(before/'src-tauri/src/native_core/extensions/process_install.rs').read_text()
launcher=(before/'src-tauri/src/native_core/extensions/launcher.rs').read_text()
constants=install[install.index('const CAPABILITY_GROUPS:'):install.index('fn valid_capability(')].replace('pub(super) ','')
validation='\nmod host_policy {use super::*;use std::collections::HashSet;mod process_install {use super::*;'+constants+'\n'+ '\n'.join(function(install,n).replace('pub(super) ','pub(super) ') for n in ['valid_capability','component','relative','validate'])+'}\nmod launcher {use super::*;use super::process_install;'+function(launcher,'bounded_text')+'\n'+function(launcher,'validate')+'''}
#[test]fn actual_pinned_manifest_validator_and_negatives(){let manifest:Value=serde_json::from_str(include_str!("../extension.json")).unwrap();assert!(process_install::validate(&manifest).is_ok());for (key,value) in [("schemaVersion",json!(2)),("runtime",json!("javascript")),("execution",json!("embedded")),("protocolVersion",json!(2)),("id",json!("raycast.fixture")),("capabilities",json!(["host.anything"]))]{let mut bad=manifest.clone();bad[key]=value;assert!(process_install::validate(&bad).is_err());}let mut duplicate=manifest.clone();duplicate["commands"].as_array_mut().unwrap().push(manifest["commands"][0].clone());assert!(process_install::validate(&duplicate).is_err());for p in ["../escape","/absolute","a\\\\b","CON","a/../b"]{assert!(process_install::relative(p).is_err());}}
}
'''
f=v2/'tests/host_extracts.rs';f.write_text(f.read_text()+validation)

process=(before/'src-tauri/src/native_core/extensions/process.rs').read_text()
start=process.index('        if hello["type"] != "hello"');end=process.index('        let paths =',start)
check=process[start:end].replace('process.package.manifest','manifest')
f=v2/'tests/host_extracts.rs';f.write_text(f.read_text()+'\nfn actual_host_hello(hello:&Value,manifest:&Value)->Result<(),String>{'+check+'Ok(())}\n'+r'''#[test]fn pinned_host_hello_policy(){let m:Value=serde_json::from_str(include_str!("../extension.json")).unwrap();let hello=json!({"type":"hello","protocol":1,"id":m["id"],"version":m["version"],"sdkVersion":"0.1.1"});assert!(actual_host_hello(&hello,&m).is_ok());for(k,v)in[("type",json!("event")),("id",json!("com.tezbar.native.wrong")),("version",json!("1.0.0")),("protocol",json!(2)),("sdkVersion",json!("1.0.0"))]{let mut h=hello.clone();h[k]=v;assert!(actual_host_hello(&h,&m).is_err())}}''')
