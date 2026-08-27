// account-selector.mjs — Multi-account registry, selection, verification, rollover.
// Metadata only; API keys live in GitHub Secrets. Never log API keys or auth headers.

import { readFileSync, existsSync } from "node:fs";

// ===== 1. Registry loading =====

export function loadRegistry(filePath) {
  if (!existsSync(filePath)) throw new Error(`Account registry not found: ${filePath}`);
  let raw;
  try { raw = readFileSync(filePath, "utf8"); } catch (err) { throw new Error(`Cannot read: ${err.message}`); }
  let parsed;
  try { parsed = JSON.parse(raw); } catch (err) { throw new Error(`Invalid JSON: ${err.message}`); }
  if (!parsed || typeof parsed !== "object") throw new Error("Registry must be a JSON object");
  if (!Array.isArray(parsed.accounts)) throw new Error("Registry must have an 'accounts' array");
  for (const [i, acct] of parsed.accounts.entries()) {
    if (!acct.id || typeof acct.id !== "string") throw new Error(`Account ${i}: missing or invalid 'id'`);
    if (!acct.secretName || typeof acct.secretName !== "string") throw new Error(`Account ${i}: missing or invalid 'secretName'`);
    if (!/^CLINE_ACCOUNT_\d+_API_KEY$/.test(acct.secretName)) {
      throw new Error(`Account ${i} ("${acct.id}"): secretName '${acct.secretName}' must match CLINE_ACCOUNT_N_API_KEY format`);
    }
  }
  // Reject duplicate account IDs
  const ids = new Set();
  for (const acct of parsed.accounts) {
    if (ids.has(acct.id)) throw new Error(`Duplicate account ID: ${acct.id}`);
    ids.add(acct.id);
  }
  // Reject duplicate secretName values
  const secrets = new Set();
  for (const acct of parsed.accounts) {
    if (secrets.has(acct.secretName)) throw new Error(`Duplicate secretName: ${acct.secretName} (both accounts would use the same credential)`);
    secrets.add(acct.secretName);
  }
  return parsed;
}

// ===== 2. Selection =====

function isClinePassModel(modelId) { return modelId.startsWith("cline-pass/"); }

export function selectEligibleAccounts(registry, modelId, env) {
  const requiresPass = isClinePassModel(modelId);
  const candidates = registry.accounts.filter((acct) => {
    if (acct.enabled !== true) return false;
    if (requiresPass && !acct.clinePass) return false;
    const secret = env[acct.secretName];
    if (!secret || typeof secret !== "string" || secret.trim() === "") return false;
    return true;
  });
  function score(acct) {
    let s = 0;
    if (Array.isArray(acct.preferredModels) && acct.preferredModels.includes(modelId)) s -= 50;
    if (requiresPass && acct.clinePass) s -= 10;
    return s + (typeof acct.priority === "number" ? acct.priority : 100);
  }
  candidates.sort((a, b) => {
    const sa = score(a), sb = score(b);
    return sa !== sb ? sa - sb : a.id.localeCompare(b.id);
  });
  return candidates.map((acct) => ({
    account: acct,
    apiKey: (env[acct.secretName] || "").trim(),
  }));
}

// ===== 2b. Tier builder (ClinePass first, then free accounts) =====

export function buildAccountTiers(registry, modelId, env) {
  const passModel = modelId || "cline-pass/deepseek-v4-flash";
  const freeProvider = process.env.CLINE_FREE_PROVIDER || "cline";
  const freeModel = process.env.CLINE_FREE_MODEL || "deepseek/deepseek-v4-flash";
  const tiers = [];

  const byId = (a, b) => (a.priority || 100) - (b.priority || 100) || a.id.localeCompare(b.id);
  const hasKey = (a) => { const s = env[a.secretName]; return s && typeof s === "string" && s.trim().length > 0; };

  const passAccounts = registry.accounts.filter(a => a.enabled !== false && a.clinePass && hasKey(a)).sort(byId);
  if (passAccounts.length > 0) {
    tiers.push({
      name: "cline-pass",
      accounts: passAccounts.map(a => ({ account: a, apiKey: (env[a.secretName] || "").trim() })),
      providerId: "cline-pass",
      modelId: passModel,
    });
  }

  const freeAccounts = registry.accounts.filter(a => a.enabled !== false && !a.clinePass && hasKey(a)).sort(byId);
  if (freeAccounts.length > 0) {
    tiers.push({
      name: "free",
      accounts: freeAccounts.map(a => ({ account: a, apiKey: (env[a.secretName] || "").trim() })),
      providerId: freeProvider,
      modelId: freeModel,
    });
  }

  return tiers;
}

// ===== 3. Identity verification =====

export async function verifyAccount(apiKey, account, options = {}) {
  const baseUrl = options.baseUrl || "https://api.cline.bot";
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  let response;
  try { response = await fetchImpl(`${baseUrl}/api/v1/users/me`, {
    headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
  }); } catch (err) { return { verified: false, reason: `Identity request failed: ${err.message}` }; }
  let body;
  try { body = await response.json(); } catch { return { verified: false, reason: `Identity response not JSON (HTTP ${response.status})` }; }
  if (!response.ok) return { verified: false, reason: `Identity check HTTP ${response.status}` };
  const userId = (body.data && body.data.id) || body.id || null;
  if (!userId) return { verified: false, reason: "Identity response missing user ID" };
  if (account.expectedUserId && userId !== account.expectedUserId) {
    return { verified: false, reason: `Identity mismatch: expected ${account.expectedUserId}, got ${userId}` };
  }
  return { verified: true, userId };
}

// ===== 4. ClinePass entitlement =====

export async function verifyEntitlement(apiKey, account, options = {}) {
  if (!account.clinePass) return { hasEntitlement: true, skipped: true };
  const baseUrl = options.baseUrl || "https://api.cline.bot";
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  let response;
  try { response = await fetchImpl(`${baseUrl}/api/v1/users/me/plan`, {
    headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
  }); } catch (err) { return { hasEntitlement: false, reason: `Entitlement request failed: ${err.message}` }; }
  let body;
  try { body = await response.json(); } catch { return { hasEntitlement: false, reason: `Entitlement response not JSON (HTTP ${response.status})` }; }
  const plan = body.data || body;
  if (!plan || !plan.plan) return { hasEntitlement: false, reason: "No active ClinePass subscription (stale metadata?)" };
  return { hasEntitlement: true };
}
// ===== 5. Failure classification =====

export function isAccountSpecificFailure(error) {
  const msg = String(error?.message ?? error ?? "").toLowerCase();
  const rollover = ["not subscribed to required model plan", "you have reached your", "clinepass limit",
    "free limit reached on model", "402", "payment required", "401", "unauthorized", "invalid api key",
    "403", "forbidden", "model not found"];
  for (const p of rollover) { if (msg.includes(p)) return true; }
  const infra = ["500", "502", "503", "timeout", "etimedout", "econnrefused", "econnreset", "enotfound",
    "eai_again", "400", "bad request"];
  for (const p of infra) { if (msg.includes(p)) return false; }
  return false;
}

// ===== 6. Rollover orchestration (account level) =====

export function createRolloverRunner(eligibleAccounts, sessionFn, identityOpts, entitlementOpts, rolloverOpts) {
  return async (...apiKeys) => {
    const attempted = new Set();
    const failures = [];
    const available = rolloverOpts?.availableSecrets;
    let accounts = eligibleAccounts;
    if (Array.isArray(available)) accounts = eligibleAccounts.filter((e) => available.includes(e.account.secretName));
    if (!accounts.length) throw new Error("No eligible accounts available for selection");
    for (let i = 0; i < accounts.length; i++) {
      const { account, apiKey } = accounts[i];
      if (attempted.has(account.id)) continue;
      attempted.add(account.id);
      const key = apiKeys[i] ?? apiKey;
      if (!key) { console.log(`[agent] Account ${account.id} skipped: key unavailable`); failures.push({ id: account.id, reason: "key unavailable" }); continue; }
      console.log(`[agent] Cline account attempt ${i + 1}/${accounts.length}: ${account.id}`);
      const idv = await verifyAccount(key, account, identityOpts || {});
      if (!idv.verified) { console.log(`[agent] ${account.id} identity: ${idv.reason}`); failures.push({ id: account.id, reason: idv.reason }); continue; }
      console.log(`[agent] ${account.id} identity=verified`);
      const ent = await verifyEntitlement(key, account, entitlementOpts || {});
      if (!ent.hasEntitlement) { console.log(`[agent] ${account.id} entitlement: ${ent.reason}`); failures.push({ id: account.id, reason: ent.reason }); continue; }
      if (!ent.skipped) console.log(`[agent] ${account.id} clinepass=verified`);
      try { return await sessionFn(account, key); }
      catch (err) {
        const isSpecific = isAccountSpecificFailure(err);
        const msg = err?.message ?? String(err);
        if (isSpecific) { console.log(`[agent] ${account.id} failed (account-specific): ${msg}`); failures.push({ id: account.id, reason: msg }); }
        else { console.log(`[agent] ${account.id} failed (infrastructure): ${msg}`); throw err; }
      }
    }
    throw new Error(`All eligible accounts exhausted (${failures.length} attempted).\n${failures.map((f) => `  ${f.id}: ${f.reason}`).join("\n")}`);
  };
}

// ===== 7. Tiered mid-task failover orchestration =====

export function createTieredFailoverRunner(tiers, sessionFactory, identityOpts, entitlementOpts) {
  if (!tiers || tiers.length === 0) throw new Error("No account tiers available");
  return async () => {
    const attempted = new Set();
    const failures = [];
    let attemptCount = 0;

    for (const tier of tiers) {
      for (const { account, apiKey } of tier.accounts) {
        if (attempted.has(account.id)) continue;
        attempted.add(account.id);
        attemptCount++;

        const key = apiKey;
        if (!key) { console.log(`[agent] Account ${account.id} skipped: key unavailable`); failures.push({ id: account.id, reason: "key unavailable", tier: tier.name }); continue; }

        console.log(`[agent] Cline account attempt ${attemptCount}: ${account.id} (tier: ${tier.name})`);
        const idv = await verifyAccount(key, account, identityOpts || {});
        if (!idv.verified) { console.log(`[agent] ${account.id} identity: ${idv.reason}`); failures.push({ id: account.id, reason: idv.reason, tier: tier.name }); continue; }
        console.log(`[agent] ${account.id} identity=verified`);
        const ent = await verifyEntitlement(key, account, entitlementOpts || {});
        if (!ent.hasEntitlement) { console.log(`[agent] ${account.id} entitlement: ${ent.reason}`); failures.push({ id: account.id, reason: ent.reason, tier: tier.name }); continue; }
        if (!ent.skipped) console.log(`[agent] ${account.id} clinepass=verified`);

        try {
          const sessionFn = sessionFactory(account, key, tier);
          return await sessionFn();
        } catch (err) {
          const isSpecific = isAccountSpecificFailure(err);
          const msg = err?.message ?? String(err);
          if (isSpecific) {
            console.log(`[agent] ${account.id} failed (account-specific): ${msg}`);
            failures.push({ id: account.id, reason: msg, tier: tier.name });
          } else {
            console.log(`[agent] ${account.id} failed (infrastructure): ${msg}`);
            throw err;
          }
        }
      }
    }

    const aggMsg = `All accounts exhausted (${failures.length} attempted across ${tiers.length} tier(s)).\n${failures.map(f => `  ${f.id} (${f.tier}): ${f.reason}`).join("\n")}`;
    throw new Error(aggMsg);
  };
}