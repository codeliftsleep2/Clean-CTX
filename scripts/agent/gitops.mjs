// gitops.mjs - Git operations for the issue-driven agent runner.
//
// Every operation independently rejects any ref outside `agent/**`, and never
// uses --force. The runner never pushes to the default branch. These guards
// are defense-in-depth on top of workflow permissions and branch protection.

import { execSync } from "node:child_process";

const RUNNER_EMAIL = "agent[bot]@users.noreply.github.com";
const RUNNER_NAME = "agent[bot]";

function run(cmd) {
  return execSync(cmd, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024, stdio: ["inherit", "pipe", "inherit"] }).toString();
}

function assertAgentBranch(branch) {
  if (!branch || !branch.startsWith("agent/") || branch.includes("..")) {
    throw new Error(`Refusing to operate on non-agent branch '${branch}'.`);
  }
}

export function currentBranch() {
  return run("git rev-parse --abbrev-ref HEAD").trim();
}

export function slugFromIssue(issue, base) {
  const raw = String(base ?? "task")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 40);
  return `${issue}-${raw || "task"}`;
}

export function isDirty() {
  return run("git status --porcelain").trim().length > 0;
}

export function ensureBranch(branch) {
  assertAgentBranch(branch);
  // Start from the current (default) branch tip.
  if (currentBranch() !== branch) {
    run(`git switch -c ${JSON.stringify(branch)}`);
  }
}

export function configureCommitter() {
  run(`git config user.name "${RUNNER_NAME}"`);
  run(`git config user.email "${RUNNER_EMAIL}"`);
}

export function commitAll(message) {
  assertAgentBranch(currentBranch());
  run("git add -A");
  run(`git commit -m ${JSON.stringify(message)}`);
}

export function pushBranch(branch) {
  assertAgentBranch(branch);
  // No --force ever; ref is shell-quoted for defense in depth.
  run(`git push -q origin HEAD:refs/heads/${JSON.stringify(branch)}`);
}

export function deleteRemoteBranch(owner, repo, branch) {
  assertAgentBranch(branch);
  run(`git push -q origin --delete ${branch} 2>/dev/null || true`);
  void owner;
  void repo;
}

export function createPr(owner, repo, issue, base, headBranch, title, body) {
  assertAgentBranch(headBranch);
  // gh writes the PR; --head is branch on the same base repo.
  const cmd =
    `gh pr create --repo "${owner}/${repo}" --base "${base}" --head "${headBranch}" ` +
    `--title ${JSON.stringify(title)} --body - <<'PRBODY'\n${body}\nPRBODY`;
  return run(cmd).trim();
}