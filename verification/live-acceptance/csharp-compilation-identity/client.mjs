// Campaign-local stdio transport. No provider, shell, or external service.
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';

export class McpClient {
  constructor(binary, root, evidenceDir) {
    this.pending = new Map();
    this.nextId = 1;
    this.dead = null;
    this.trace = path.join(evidenceDir, 'rpc.jsonl');
    this.stderr = path.join(evidenceDir, 'stderr.log');
    fs.writeFileSync(this.trace, '', 'utf8');
    fs.writeFileSync(this.stderr, '', 'utf8');
    this.child = spawn(binary, [], { cwd: root, windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
    this.exit = new Promise((resolve) => {
      this.child.once('exit', (code, signal) => {
        this.record('exit', { code, signal });
        this.fail(new Error(`server exited: code=${code}, signal=${signal}`));
        resolve();
      });
      this.child.once('error', (error) => { this.fail(error); resolve(); });
    });
    this.child.stdin.on('error', (error) => this.fail(error));
    this.child.stderr.on('data', (chunk) => fs.appendFileSync(this.stderr, chunk));
    const lines = readline.createInterface({ input: this.child.stdout });
    lines.on('line', (line) => {
      let message;
      try { message = JSON.parse(line); }
      catch {
        this.record('invalid-stdout', line);
        this.fail(new Error('non-JSON server stdout; see rpc.jsonl'));
        return;
      }
      this.record('receive', message);
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      clearTimeout(pending.timer);
      pending.resolve(message);
    });
  }

  record(direction, message, label) {
    fs.appendFileSync(this.trace, JSON.stringify({
      time: new Date().toISOString(), direction, label, message,
    }) + '\n', 'utf8');
  }

  fail(error) {
    this.dead ??= error;
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }

  request(method, params, label) {
    if (this.dead) return Promise.reject(this.dead);
    const id = this.nextId++;
    const message = { jsonrpc: '2.0', id, method, params };
    this.record('send', message, label);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`120s request timeout: ${label}`));
      }, 120000);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(JSON.stringify(message) + '\n');
    });
  }

  async initialize() {
    const response = await this.request('initialize', {
      protocolVersion: '2024-11-05', capabilities: {},
      clientInfo: { name: 'csharp-compilation-identity-153-154', version: '1.0.0' },
    }, 'initialize');
    if (response.error || !response.result) throw new Error('initialize failed');
    const message = { jsonrpc: '2.0', method: 'notifications/initialized', params: {} };
    this.record('send', message);
    this.child.stdin.write(JSON.stringify(message) + '\n');
  }

  async call(name, args, label) {
    const response = await this.request('tools/call', { name, arguments: args }, label);
    if (response.error || !response.result || response.result.isError) {
      throw new Error(`${label}: MCP/tool error; see rpc.jsonl`);
    }
    return response.result;
  }

  async close() {
    const priorError = this.dead;
    this.fail(new Error('harness closing'));
    this.child.stdin.end();
    if (this.child.exitCode !== null || this.child.signalCode !== null) {
      if (priorError) throw priorError;
      return;
    }
    const force = setTimeout(() => this.child.kill(), 2000);
    const hard = setTimeout(() => this.child.kill('SIGKILL'), 4000);
    let shutdownTimer;
    try {
      await Promise.race([
        this.exit,
        new Promise((_, reject) => {
          shutdownTimer = setTimeout(() => reject(new Error('server failed to stop within 6s')), 6000);
        }),
      ]);
      if (priorError) throw priorError;
    } finally {
      clearTimeout(force);
      clearTimeout(hard);
      clearTimeout(shutdownTimer);
    }
  }
}
