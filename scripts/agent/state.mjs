// state.mjs - GitHub CLI helpers + per-issue agent state persistence.
//
// State is carried in a single bot-managed issue comment carrying a
// machine-readable JSON blob (AGENT-STATE marker). This is durable across
// separate GitHub Actions runs (approval is a separate run from plan), visible
// for review, and requires no extra branches or caches.

import { execSync } from "node:child_process";

const STATE_MARKER = "<!--AGENT-STATE-->\n```json\n";

function run(cmd, { stdin } = {}) {
  try {
    return execSync(cmd, {
      encoding: "utf8",
      input: stdin,
      maxBuffer: 64 * 1024 * 1024,
      stdio: stdin !== undefined ? ["pipe", "pipe", "inherit"] : ["inherit", "pipe", "inherit"],
    }).toString();
  } catch (err) {
    err.stdout = err.stdout ? String(err.stdout) : "";
    err.stderrAll = err.stderr ? String(err.stderr) : "";
    throw err;
  }
}

export function gh(method, path, body) {
  const args = `-X ${method} "${path}"`;
  return run(`gh api ${args} --input -`, { stdin: JSON.stringify(body ?? {}) });
}

export function ghNoBody(method, path) {
  return run(`gh api -X ${method} "${path}" --jq .`);
}

export function issueComments(owner, repo, issue) {
  const out = run(
    `gh api "repos/${owner}/${repo}/issues/${issue}/comments?per_page=100" ` +
    `--jq ".[] | {id, body, login: .user.login, at: .created_at}"`,
  );
  return parseLineJson(out);
}

// Parses JSON-lines output (a common gh --jq shape) robustly, skipping any
// malformed lines instead of propagating parse failures.
function parseLineJson(raw) {
  const lines = String(raw ?? "").trim().split(/\r?\n/).filter(Boolean);
  if (!lines.length) return [];
  const result = [];
  for (const l of lines) { try { result.push(JSON.parse(l)); } catch {/*skip*/} }
  return result;
}

export function issueInfo(owner, repo, issue) {
  const out = run(`gh api "repos/${owner}/${repo}/issues/${issue}" --jq "{title, body, state, number}"`);
  const obj = parseLineJson(out);
  return obj[0] ?? { title: `issue ${issue}`, body: "", state: "open", number: Number(issue) };
}

export function postComment(owner, repo, issue, body) {
  return run(`gh api -X POST "repos/${owner}/${repo}/issues/${issue}/comments" --input -`, { stdin: JSON.stringify({ body }) });
}

export function updateComment(owner, repo, commentId, body) {
  return run(`gh api -X PATCH "repos/${owner}/${repo}/issues/comments/${commentId}" --input -`, { stdin: JSON.stringify({ body }) });
}

export async function authorPermission(owner, repo, login) {
  try {
    const out = run(
      `gh api "repos/${owner}/${repo}/collaborators/${encodeURIComponent(login)}/permission" --jq .permission`,
    );
    return out.trim().toLowerCase();
  } catch {
    return "none";
  }
}

// Fail-closed by default: only write/admin may issue agent commands. The
// `allowReadForRun` opt-in exists for explicit, narrower deployments; the
// production runner never enables it.
export function requireAuthorized(permission, { allowReadForRun = false } = {}) {
  const allowed = new Set(allowReadForRun ? ["write", "admin", "read"] : ["write", "admin"]);
  if (!allowed.has(permission)) {
    throw new Error(`Actor is not an authorized collaborator (permission '${permission}' is not sufficient).`);
  }
}

export function stateBody(state, planMarkdown) {
  const json = JSON.stringify(state, null, 2);
  const md = planMarkdown ?? "";
  return `${STATE_MARKER}${json}\n\`\`\`\n\n${md}`;
}

export function parseState(body) {
  if (!body || !body.includes("```json\n")) return null;
  const start = body.indexOf(STATE_MARKER);
  if (start < 0) return null;
  const jsonStart = start + STATE_MARKER.length;
  const end = body.indexOf("```\n", jsonStart);
  if (end < 0) return null;
  try {
    return JSON.parse(body.slice(jsonStart, end));
  } catch {
    return null;
  }
}

export async function findState(owner, repo, issue) {
  const comments = issueComments(owner, repo, issue);
  for (const c of comments) {
    const st = c.body ? parseState(c.body) : null;
    if (st) return { ...c, state: st };
  }
  return null;
}

export function emptyState(issue) {
  return {
    version: 1,
    status: "NEW",
    issue: Number(issue),
    branch: null,
    pr: null,
    slug: null,
    plan: null,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    lastCommand: null,
    lastActor: null,
  };
}

export function tick(state, status, extra = {}) {
  return { ...state, ...extra, status, updatedAt: new Date().toISOString() };
}

// Persist the state comment: create it, or update the previously found one.
export async function persistState(owner, repo, issue, commentId, state, planMarkdown) {
  const body = stateBody(state, planMarkdown);
  if (commentId) {
    updateComment(owner, repo, commentId, body);
    return commentId;
  }
  const res = postComment(owner, repo, issue, body);
  return extractCommentId(res);
}

export async function readOrCreateState(owner, repo, issue) {
  const found = await findState(owner, repo, issue);
  return found ? { commentId: found.id, state: found.state } : { commentId: null, state: emptyState(issue) };
}

function extractCommentId(ghCreateOutput) {
  try {
    const obj = JSON.parse(ghCreateOutput);
    return obj.id;
  } catch {
    return null; // id unknown; next state write creates a new comment.
  }
}