// account-selector.mapping.test.mjs — Focused tests for account-to-secret mapping,
// duplicate rejection, secret name validation, and credential isolation.
import assert from "node:assert/strict";
import { writeFileSync, unlinkSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const T = []; let p = 0, f = 0;
function test(n, fn) { T.push([n, fn]); }

async function withRegistry(json, fn) {
  const d = path.join(__dirname, ".scratch-map");
  const f = path.join(d, "r.json");
  try { mkdirSync(d, { recursive: true }); } catch {}
  writeFileSync(f, JSON.stringify(json), "utf8");
  try {
    const { loadRegistry } = await import("./account-selector.mjs");
    await fn(loadRegistry(f));
  } finally {
    try { unlinkSync(f); unlinkSync(d); } catch {}
  }
}

async function assertRegistryThrows(json, needle) {
  const d = path.join(__dirname, ".scratch-map");
  const f = path.join(d, "bad.json");
  try { mkdirSync(d, { recursive: true }); } catch {}
  writeFileSync(f, JSON.stringify(json), "utf8");
  try {
    const { loadRegistry } = await import("./account-selector.mjs");
    assert.throws(() => loadRegistry(f), needle);
  } finally {
    try { unlinkSync(f); unlinkSync(d); } catch {}
  }
}

test("map: duplicate account ID rejected", async () => {
  await assertRegistryThrows({
    accounts: [
      { id: "acc-01", secretName: "CLINE_ACCOUNT_01_API_KEY", enabled: true },
      { id: "acc-01", secretName: "CLINE_ACCOUNT_02_API_KEY", enabled: true },
    ],
  }, /duplicate/i);
});

test("map: duplicate secretName rejected", async () => {
  await assertRegistryThrows({
    accounts: [
      { id: "acc-01", secretName: "CLINE_ACCOUNT_01_API_KEY", enabled: true },
      { id: "acc-02", secretName: "CLINE_ACCOUNT_01_API_KEY", enabled: true },
    ],
  }, /duplicate/i);
});

test("map: empty secretName rejected", async () => {
  await assertRegistryThrows({
    accounts: [{ id: "a", secretName: "", enabled: true }],
  }, /secretName/i);
});

test("map: acc-01 resolves to correct key", async () => {
  const { selectEligibleAccounts } = await import("./account-selector.mjs");
  const reg = { accounts: [
    { id: "a1", secretName: "S1", enabled: true, priority: 10, clinePass: false, preferredModels: [] },
    { id: "a2", secretName: "S2", enabled: true, priority: 20, clinePass: false, preferredModels: [] },
  ]};
  const env = { S1: "key-for-a1", S2: "key-for-a2" };
  const cs = selectEligibleAccounts(reg, "m/m", env);
  assert.equal(cs[0].apiKey, "key-for-a1");
  assert.equal(cs[1].apiKey, "key-for-a2");
});

test("map: accounts get different keys", async () => {
  const { selectEligibleAccounts } = await import("./account-selector.mjs");
  const reg = { accounts: [
    { id: "a1", secretName: "S1", enabled: true, priority: 10, clinePass: false, preferredModels: [] },
    { id: "a2", secretName: "S2", enabled: true, priority: 20, clinePass: false, preferredModels: [] },
  ]};
  const env = { S1: "key-a1", S2: "key-a2" };
  const cs = selectEligibleAccounts(reg, "m/m", env);
  assert.notEqual(cs[0].apiKey, cs[1].apiKey);
});

test("map: missing a1 secret does not give a2 key to a1", async () => {
  const { selectEligibleAccounts } = await import("./account-selector.mjs");
  const reg = { accounts: [
    { id: "a1", secretName: "S1", enabled: true, priority: 10, clinePass: false, preferredModels: [] },
    { id: "a2", secretName: "S2", enabled: true, priority: 20, clinePass: false, preferredModels: [] },
  ]};
  const cs = selectEligibleAccounts(reg, "m/m", { S2: "key-a2" });
  assert.equal(cs.length, 1, "only a2 should be eligible");
  assert.equal(cs[0].account.id, "a2");
  assert.equal(cs[0].apiKey, "key-a2");
});

test("map: rollover passes a2 key to session", async () => {
  const { createRolloverRunner } = await import("./account-selector.mjs");
  const received = [];
  const r = createRolloverRunner(
    [{ account: { id: "a1", clinePass: true, secretName: "S1" }, apiKey: "k1" },
     { account: { id: "a2", clinePass: true, secretName: "S2" }, apiKey: "k2" }],
    async (acct, key) => { received.push({ id: acct.id, key }); if (acct.id === "a1") throw new Error("you have reached your clinepass limit"); return { success: true }; },
    { fetchImpl: async () => ({ ok: true, status: 200, json: async () => ({ id: "u" }), text: async () => '{"id":"u"}', headers: { get: () => null } }) },
    { fetchImpl: async () => ({ ok: true, status: 200, json: async () => ({ plan: { id: "p" } }), text: async () => '{"plan":{"id":"p"}}', headers: { get: () => null } }) },
  );
  const res = await r();
  assert.equal(received.length, 2);
  assert.equal(received[0].key, "k1");
  assert.equal(received[1].key, "k2");
  assert.equal(res.success, true);
});
test("map: error messages do not contain key values", async () => {
  const { createRolloverRunner, isAccountSpecificFailure } = await import("./account-selector.mjs");
  const err = new Error("HTTP 401: Invalid API key");
  assert.equal(isAccountSpecificFailure(err), true);
  assert.ok(!err.message.includes("sk-"), "error must not contain key pattern");
  assert.ok(!err.message.includes("key-for-"), "error must not contain test key");
});

test("map: no API key leakage in aggregate error", async () => {
  const { createRolloverRunner } = await import("./account-selector.mjs");
  const r = createRolloverRunner(
    [{ account: { id: "a1", clinePass: true, secretName: "S1" }, apiKey: "sk-secret-value" }],
    async () => { throw new Error("you have reached your clinepass limit"); },
    { fetchImpl: async () => ({ ok: true, status: 200, json: async () => ({ id: "u" }), text: async () => '{"id":"u"}', headers: { get: () => null } }) },
    { fetchImpl: async () => ({ ok: true, status: 200, json: async () => ({ plan: { id: "p" } }), text: async () => '{"plan":{"id":"p"}}', headers: { get: () => null } }) },
  );
  try { await r(); assert.fail("should have thrown"); }
  catch (err) {
    assert.ok(!err.message.includes("sk-"), "aggregate error must not contain key");
    assert.ok(err.message.includes("a1"), "aggregate error should reference account id");
  }
});

async function main() {
  for (const [name, fn] of T) {
    try { await fn(); p++; console.log(`PASS: ${name}`); }
    catch (err) { f++; console.error(`FAIL: ${name}\n  ${err?.message ?? err}`); }
  }
  const hf = f > 0;
  console.log(`\n${p} passed, ${f} failed.`);
  await new Promise(r => setTimeout(r, 10));
  process.exit(hf ? 1 : 0);
}
main();