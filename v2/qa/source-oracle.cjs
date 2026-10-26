// Executes WHOLE original TSX commands, bundled with exact locked libraries.
// VM has no require/process/OS/fs/network/browser; every host producer is synthetic.
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const crypto = require('node:crypto');
const cp = require('node:child_process');
const esbuild = require('../.qa/v1/node_modules/esbuild');
const base = path.resolve(__dirname,'../.qa/v1');
const keys = require(base+'/package.json').preferences.filter(p=>p.type==='checkbox').map(p=>p.name);
const host = `export const Clipboard={readText:async()=>__env.clipboard, paste:async(text)=>{__env.effects.push({method:'paste',text})}};
export const getPreferenceValues=()=>__env.preferences;
export const showToast=async(v)=>__env.effects.push({method:'toast',...v});
export const Toast={Style:{Failure:'failure'}};
export const openCommandPreferences=()=>__env.effects.push({method:'settings'});
export const Color={Yellow:'yellow'};export const Icon={Warning:'warning'};
const component=(name)=>Object.assign((props)=>({type:name,props}),{});
export const List=Object.assign(component('List'),{Item:component('List.Item'),EmptyView:component('List.EmptyView')});
export const Action=Object.assign(component('Action'),{CopyToClipboard:component('copy'),Paste:component('paste')});
export const ActionPanel=component('ActionPanel');`;
const react = `export const useState=(initial)=>__hooks.state(initial);export const useEffect=(fn,deps)=>__hooks.effect(fn,deps);
const React={useState,useEffect,createElement:(type,props,...children)=>({type,props:{...props,children}})};export default React;`;
async function bundle(file){return (await esbuild.build({entryPoints:[path.join(base,'src',file+'.tsx')],bundle:true,jsx:'transform',tsconfigRaw:{compilerOptions:{jsx:'react'}},write:false,format:'iife',globalName:'original',platform:'neutral',mainFields:['main'],plugins:[{name:'closed-host',setup(b){b.onResolve({filter:/^(@raycast\/api|react)$/},a=>({path:a.path,namespace:'mock'}));b.onLoad({filter:/.*/,namespace:'mock'},a=>({contents:a.path==='react'?react:host,loader:'js'}))}}]})).outputFiles[0].text}
function make(code,test){let cursor=0,states=[],oldDeps=[],pending=[],dirty=false;
const env={preferences:{...Object.fromEntries(keys.map(k=>[k,true])),naturalLanguageDateFormat:'YYYY-MM-DD',autoPasteClipboard:false,...test.preferences},clipboard:test.clipboard||'',effects:[]};
const hooks={state(init){const i=cursor++;if(!(i in states))states[i]=init;return [states[i],v=>{const n=typeof v==='function'?v(states[i]):v;if(n!==states[i]){states[i]=n;dirty=true}}]},effect(fn,deps){const i=cursor++;if(!deps||!oldDeps[i]||deps.some((v,j)=>v!==oldDeps[i][j])){pending.push(fn);oldDeps[i]=deps}}};
const NativeDate=Date;const fixed=test.now;
class FixedDate extends NativeDate{constructor(...args){super(...(args.length?args:[fixed]))}static now(){return fixed}}
const sandbox={__env:env,__hooks:hooks,Date:FixedDate,console:{log(){},warn(){},error(){}},setTimeout(){throw Error('timers forbidden')},fetch(){throw Error('network forbidden')},WebSocket:undefined,XMLHttpRequest:undefined};
const ctx=vm.createContext(sandbox,{codeGeneration:{strings:false,wasm:false}});vm.runInContext(code,ctx,{timeout:5000});
let tree;async function render(){for(let j=0;j<12;j++){dirty=false;cursor=0;pending=[];tree=ctx.original.default();for(const fn of pending)fn();await Promise.resolve();await Promise.resolve();if(!dirty)break}}
return {env,ctx,render,get tree(){return tree}}}
function walk(node,out=[]){if(!node)return out;if(Array.isArray(node)){for(const n of node)walk(n,out);return out}if(typeof node==='object'){out.push(node);walk(node.props?.children,out)}return out}
async function execute(code,test){const h=make(code,test);if(test.kind==='natural'){await h.ctx.original.default({arguments:{prompt:test.text,format:test.format||''}});const paste=h.env.effects.find(e=>e.method==='paste');return paste?{text:paste.text}:{failure:h.env.effects[0]?.title?.startsWith('Invalid date format:')?'format':'parse'}}
await h.render();h.tree.props.onSearchTextChange(test.text);await h.render();const nodes=walk(h.tree);const values=nodes.filter(n=>typeof n.props?.title==='string'&&n.props.actions).map(n=>n.props.title).filter(t=>!t.includes('preferences'));
// Verify both source action payloads for each rendered row.
for(const row of nodes.filter(n=>values.includes(n.props?.title))){const a=row.props.actions;const component=a.type(a.props);const acts=walk(component).filter(n=>n.props?.content!==undefined);if(acts.length!==2||acts.some(a=>a.props.content!==row.props.title))throw Error('source action mismatch')}
const empty=nodes.find(n=>n.props?.title==='An error occurred');return {valid:!empty,values}}
if(process.argv[2]==='--worker'){(async()=>{const tests=JSON.parse(fs.readFileSync(0,'utf8'));const codes={format:await bundle('index'),natural:await bundle('insert-natural-language-date')};const result=[];for(const t of tests)result.push({...t,expected:await execute(codes[t.kind],t)});process.stdout.write(JSON.stringify(result))})().catch(e=>{console.error(e);process.exitCode=1});}
else if(require.main===module){const tests=JSON.parse(fs.readFileSync(path.join(__dirname,'vectors.json'),'utf8'));const all=[];for(const tz of [...new Set(tests.map(t=>t.tz))]){const tmp=path.resolve(__dirname,'../.qa');const r=cp.spawnSync(process.execPath,[__filename,'--worker'],{input:JSON.stringify(tests.filter(t=>t.tz===tz)),env:{PATH:process.env.PATH,TZ:tz,HOME:tmp,TMPDIR:tmp},encoding:'utf8',maxBuffer:32*1024*1024});if(r.status!==0)throw Error(r.stderr);all.push(...JSON.parse(r.stdout))}fs.writeFileSync(path.join(__dirname,'../evidence/source-vectors.json'),JSON.stringify(all,null,2)+'\n');console.log(all.length+' whole-source vectors');}
