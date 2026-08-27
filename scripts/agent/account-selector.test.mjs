import assert from "node:assert/strict";
import{writeFileSync,unlinkSync,mkdirSync}from "node:fs";
import path from "node:path";
import{fileURLToPath}from "node:url";
const __dirname=path.dirname(fileURLToPath(import.meta.url));
const T=[];let p=0,f=0;
function test(n,fn){T.push([n,fn]);}
function regStr(ov){return JSON.stringify({accounts:[{id:"cp-01",secretName:"S1",enabled:true,priority:10,clinePass:true,preferredModels:["cline-pass/deepseek-v4-flash"]},{id:"cp-02",secretName:"S2",enabled:true,priority:20,clinePass:true,preferredModels:[]},{id:"ub-01",secretName:"S3",enabled:true,priority:30,clinePass:false,preferredModels:[]}],...ov});}
function env(e){return{S1:"k1",S2:"k2",S3:"k3",...e};}
function mF(s,b){const t=typeof b==="string"?b:JSON.stringify(b);return async()=>({ok:s>=200&&s<300,status:s,json:async()=>b,text:async()=>t,headers:{get:()=>null}});}
test("reg:valid JSON",async()=>{const{loadRegistry}=await import("./account-selector.mjs");const d=path.join(__dirname,".scratch");const f=path.join(d,"r.json");try{mkdirSync(d,{recursive:true})}catch{}writeFileSync(f,regStr(),"utf8");const r=loadRegistry(f);assert.equal(r.accounts.length,3);try{unlinkSync(f);unlinkSync(d)}catch{}});
test("reg:missing file",async()=>{const{loadRegistry}=await import("./account-selector.mjs");assert.throws(()=>loadRegistry("/nonexistent.json"),/not found/)});
test("id:success",async()=>{const{verifyAccount}=await import("./account-selector.mjs");const r=await verifyAccount("k",{id:"t",clinePass:true},{fetchImpl:mF(200,{id:"u1"})});assert.equal(r.verified,true);assert.equal(r.userId,"u1")});
test("id:failure",async()=>{const{verifyAccount}=await import("./account-selector.mjs");assert.equal((await verifyAccount("k",{id:"t",clinePass:true},{fetchImpl:mF(401,{})})).verified,false)});
test("id:mismatch",async()=>{const{verifyAccount}=await import("./account-selector.mjs");assert.equal((await verifyAccount("k",{id:"t",clinePass:true,expectedUserId:"u1"},{fetchImpl:mF(200,{id:"u2"})})).verified,false)});
test("ent:active Pass",async()=>{const{verifyEntitlement}=await import("./account-selector.mjs");assert.equal((await verifyEntitlement("k",{clinePass:true},{fetchImpl:mF(200,{plan:{id:"p"}})})).hasEntitlement,true)});
test("ent:missing Pass",async()=>{const{verifyEntitlement}=await import("./account-selector.mjs");assert.equal((await verifyEntitlement("k",{clinePass:true},{fetchImpl:mF(200,{plan:null})})).hasEntitlement,false)});
test("ent:non-Pass skip",async()=>{const{verifyEntitlement}=await import("./account-selector.mjs");const r=await verifyEntitlement("k",{clinePass:false});assert.equal(r.skipped,true);assert.equal(r.hasEntitlement,true)});
test("fail:not subscribed",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(isAccountSpecificFailure(new Error("not subscribed to required model plan")))});
test("fail:pass limit",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(isAccountSpecificFailure(new Error("you have reached your clinepass limit")))});
test("fail:402",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(isAccountSpecificFailure(new Error("402 Payment Required")))});
test("fail:401",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(isAccountSpecificFailure(new Error("401 Unauthorized")))});
test("fail:403",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(isAccountSpecificFailure(new Error("403 Forbidden")))});
test("fail:500 NOT",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(!isAccountSpecificFailure(new Error("HTTP 500 error")))});
test("fail:502 NOT",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(!isAccountSpecificFailure(new Error("HTTP 502 Bad Gateway")))});
test("fail:timeout NOT",async()=>{const{isAccountSpecificFailure}=await import("./account-selector.mjs");assert.ok(!isAccountSpecificFailure(new Error("fetch ETIMEDOUT")))});
test("roll:q1->a2",async()=>{const{createRolloverRunner}=await import("./account-selector.mjs");const att=[];const r=createRolloverRunner([{account:{id:"a1",clinePass:true,secretName:"S1"},apiKey:"k1"},{account:{id:"a2",clinePass:true,secretName:"S2"},apiKey:"k2"}],async a=>{att.push(a.id);if(a.id==="a1")throw new Error("you have reached your clinepass limit");return{success:true}},{fetchImpl:mF(200,{id:"u"})},{fetchImpl:mF(200,{plan:{id:"p"}})});const res=await r();assert.deepEqual(att,["a1","a2"]);assert.equal(res.success,true)});
test("roll:500 NOT",async()=>{const{createRolloverRunner}=await import("./account-selector.mjs");const att=[];const r=createRolloverRunner([{account:{id:"a1",clinePass:true,secretName:"S1"},apiKey:"k1"}],async a=>{att.push(a.id);throw new Error("HTTP 500 error")},{fetchImpl:mF(200,{id:"u"})},{fetchImpl:mF(200,{plan:{id:"p"}})});await assert.rejects(()=>r());assert.equal(att.length,1)});
test("roll:exhausted",async()=>{const{createRolloverRunner}=await import("./account-selector.mjs");await assert.rejects(()=>createRolloverRunner([{account:{id:"a1",clinePass:true,secretName:"S1"},apiKey:"k1"}],async()=>{throw new Error("you have reached your clinepass limit")},{fetchImpl:mF(200,{id:"u"})},{fetchImpl:mF(200,{plan:{id:"p"}})})(),e=>{assert.ok(e.message.includes("eligible accounts"),"exhausted message");return true})});
test("roll:not twice",async()=>{const{createRolloverRunner}=await import("./account-selector.mjs");const att=[];await assert.rejects(()=>createRolloverRunner([{account:{id:"a1",clinePass:true,secretName:"S1"},apiKey:"k1"}],async a=>{att.push(a.id);throw new Error("you have reached your clinepass limit")},{fetchImpl:mF(200,{id:"u"})},{fetchImpl:mF(200,{plan:{id:"p"}})})(),()=>{assert.equal(att.length,1);return true})});

async function main(){for(const[n,fn]of T){try{await fn();p++;console.log("PASS: "+n)}catch(err){f++;console.error("FAIL: "+n+"\n"+(err?.message??err))}}const hf=f>0;console.log("\n"+p+" passed, "+f+" failed.");process.exit(hf?1:0);}
main();
test("sel:ClinePass filters",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");const cs=selectEligibleAccounts(JSON.parse(regStr()),"cline-pass/deepseek-v4-flash",env());assert.equal(cs.length,2);assert.ok(cs.every(c=>c.account.clinePass))});
test("sel:non-Pass allows all",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");assert.equal(selectEligibleAccounts(JSON.parse(regStr()),"m/m",env()).length,3)});
test("sel:disabled excluded",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");const reg=JSON.parse(regStr());reg.accounts[0].enabled=false;assert.equal(selectEligibleAccounts(reg,"cline-pass/deepseek-v4-flash",env()).length,1)});
test("sel:missing secret skipped",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");assert.equal(selectEligibleAccounts(JSON.parse(regStr()),"cline-pass/deepseek-v4-flash",env({S2:undefined})).length,1)});
test("sel:preferred model",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");const reg=JSON.parse(regStr());reg.accounts[0].preferredModels=[];reg.accounts[1].preferredModels=["cline-pass/deepseek-v4-flash"];reg.accounts[0].priority=20;reg.accounts[1].priority=10;assert.equal(selectEligibleAccounts(reg,"cline-pass/deepseek-v4-flash",env())[0].account.id,"cp-02")});
test("sel:deterministic",async()=>{const{selectEligibleAccounts}=await import("./account-selector.mjs");const reg=JSON.parse(regStr());reg.accounts.forEach(a=>{a.preferredModels=[];a.priority=10});const a=selectEligibleAccounts(reg,"cline-pass/deepseek-v4-flash",env());const b=selectEligibleAccounts(reg,"cline-pass/deepseek-v4-flash",env());assert.deepEqual(a.map(x=>x.account.id),b.map(x=>x.account.id))});

