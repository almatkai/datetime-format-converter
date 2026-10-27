// AJV is a QA-only dependency with its own lock, never part of the product.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const Ajv=require('../.qa/schema/node_modules/ajv/dist/2020');
const schema=JSON.parse(fs.readFileSync(path.join(__dirname,'host/schemas/extension-v3.schema.json')));
const ajv=new Ajv({strict:false,validateFormats:false});const validate=ajv.compile(schema);
const m=JSON.parse(fs.readFileSync(path.join(__dirname,'../extension.json')));assert(validate(m),JSON.stringify(validate.errors));
for(const change of [v=>v.schemaVersion=2,v=>v.runtime='javascript',v=>v.execution='embedded',v=>v.protocolVersion=2,v=>v.capabilities.push('host.anything'),v=>v.id='raycast.fixture',v=>v.artifacts={fake:{}},v=>v.files={'../escape':'a'.repeat(64)},v=>v.extra=true,v=>v.commands[0].refreshMs=100]){const bad=structuredClone(m);change(bad);assert.equal(validate(bad),false)}
console.log('Actual pinned schema + 10 negative fixtures passed');
