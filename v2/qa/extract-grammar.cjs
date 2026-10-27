// Pure, offline extraction from npm ci of the ORIGINAL root lockfile.
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '../.qa/v1/node_modules/chrono-node/dist/cjs');
const chrono = require(root);
const constants = require(root + '/locales/en/constants.js');
const timezone = require(root + '/timezone.js');
const cfg = new (require(root + '/locales/en/configuration.js').default)().createCasualConfiguration();
const context = {option:{},text:'',refDate:new Date('2024-11-01T01:56:27.456Z')};
const parsers = cfg.parsers.map(p => ({name:p.constructor.name,pattern:(p.innerPattern ? p.innerPattern(context) : p.pattern(context)).source, boundary:!!p.innerPattern}));
const data = {version:'chrono-node@2.8.3',parsers,constants:Object.fromEntries(Object.entries(constants).filter(([k,v]) => typeof v !== 'function')),timezones:timezone.TIMEZONE_ABBR_MAP};
fs.writeFileSync(path.resolve(__dirname,'../resources/chrono-en.json'),JSON.stringify(data,null,2)+'\n');
