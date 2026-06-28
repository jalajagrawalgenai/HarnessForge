#!/usr/bin/env node
/**
 * Forge Observe Hook — Claude Code integration.
 *
 * Receives Claude Code hook events via stdin (fire-and-forget).
 * POSTs to Forge server for real-time harness analysis.
 * NEVER crashes, NEVER blocks — always exits 0.
 *
 * Usage:
 *   node observe_hook.mjs [--port PORT] [--url FORGE_SERVER_URL]
 *
 * On Windows (no Node.js), use the PowerShell hook:
 *   powershell -File observe_hook.ps1
 */
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

// ── Parse CLI args ──
const args = process.argv.slice(2);
let cliPort = null;
let cliUrl = null;
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--port' && args[i + 1]) { cliPort = parseInt(args[i + 1], 10); i++; }
  if (args[i] === '--url' && args[i + 1]) { cliUrl = args[i + 1]; i++; }
}

// ── Config ──
const FORGE_SERVER_URL = cliUrl || process.env.FORGE_SERVER_URL || null;
const PORT = cliPort || (() => {
  try {
    const pf = path.join(os.homedir(), '.forge', 'port');
    if (fs.existsSync(pf)) return parseInt(fs.readFileSync(pf, 'utf8').trim(), 10) || 3000;
  } catch (_) {}
  return parseInt(process.env.FORGE_SERVER_PORT, 10) || 3000;
})();

const INGEST_URL = FORGE_SERVER_URL
  ? `${FORGE_SERVER_URL.replace(/\/+$/, '')}/api/v1/ingest/event`
  : `http://127.0.0.1:${PORT}/api/v1/ingest/event`;

const LOG_FILE = path.join(os.homedir(), '.forge', 'hook.log');

// Detect model from environment
function detectModel() {
  // Claude Code sets these env vars
  for (const key of ['CLAUDE_MODEL', 'ANTHROPIC_MODEL', 'ANTHROPIC_DEFAULT_MODEL']) {
    if (process.env[key]) return process.env[key];
  }
  // Check other providers
  for (const key of ['OPENAI_MODEL', 'DEEPSEEK_MODEL', 'GEMINI_MODEL']) {
    if (process.env[key]) return process.env[key];
  }
  return null;
}

function log(msg) {
  try {
    const dir = path.dirname(LOG_FILE);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.appendFileSync(LOG_FILE, `[${new Date().toISOString()}] ${msg}\n`);
  } catch (_) {}
}

// ── Fire-and-forget HTTP POST ──
function postEvent(data) {
  return new Promise((resolve) => {
    try {
      const body = JSON.stringify(data);
      const url = new URL(INGEST_URL);
      const req = http.request({
        hostname: url.hostname,
        port: url.port || 80,
        path: url.pathname,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(body),
          'User-Agent': 'Forge-Observe/0.3.0',
        },
        timeout: 5000,
      }, (res) => {
        let out = '';
        res.on('data', c => out += c);
        res.on('end', () => {
          if (res.statusCode === 200) {
            log(`OK ${data.hookName} session=${data.sessionId}`);
          } else {
            log(`HTTP ${res.statusCode} ${data.hookName}: ${out.substring(0,200)}`);
          }
          resolve(true);
        });
      });
      req.on('error', (e) => { log(`ERR ${data.hookName}: ${e.message}`); resolve(false); });
      req.on('timeout', () => { req.destroy(); log(`TIMEOUT ${data.hookName}`); resolve(false); });
      req.write(body);
      req.end();
    } catch (e) { log(`EXCEPTION: ${e.message}`); resolve(false); }
  });
}

// ── Build envelope from Claude Code hook data ──
function buildEnvelope(raw) {
  try {
    const data = typeof raw === 'string' ? JSON.parse(raw) : raw;
    if (!data || !data.hook_event_name) return null;

    const hookName = data.hook_event_name;
    const sessionId = data.session_id || process.env.CLAUDE_SESSION_ID || 'unknown';
    const agentId = data.agent_id || data.subagent_id || 'root';
    const toolName = data.tool_name || '';
    const cwd = data.cwd || process.cwd();

    // Strip large base64 content to keep payloads small
    const resp = data.tool_response;
    if (Array.isArray(resp)) {
      for (const item of resp) {
        if (item?.source?.type === 'base64' && item.source.data?.length > 100000) {
          item.source.data = '[REDACTED]';
        }
      }
    }

    const flags = {};
    if (hookName === 'SessionStart') { flags.startsSession = true; flags.clearsNotification = true; }
    if (hookName === 'SessionEnd' || hookName === 'Stop') flags.stopsSession = true;

    // Inject model into payload if missing (Claude Code hooks don't always include model)
    if (!data.model) {
      const envModel = detectModel();
      if (envModel) data.model = envModel;
    }

    return {
      agentClass: 'claude-code',
      sessionId, agentId, hookName, toolName,
      payload: data, cwd,
      timestamp: Date.now(),
      flags,
    };
  } catch (e) { log(`BUILD_ERR: ${e.message}`); return null; }
}

// ── Read stdin with timeout ──
function readStdin(timeoutMs) {
  return new Promise((resolve) => {
    let input = '';
    let settled = false;
    const timer = setTimeout(() => {
      if (!settled) { settled = true; resolve(input || null); }
    }, timeoutMs);

    try {
      // Check if stdin is a TTY (no pipe) — if so, resolve immediately
      if (process.stdin.isTTY) {
        clearTimeout(timer);
        resolve(null);
        return;
      }
      process.stdin.setEncoding('utf8');
      process.stdin.on('data', (chunk) => { input += chunk; });
      process.stdin.on('end', () => {
        if (!settled) { settled = true; clearTimeout(timer); resolve(input || null); }
      });
      process.stdin.on('error', (e) => {
        if (!settled) { settled = true; clearTimeout(timer); log(`STDIN_ERR: ${e.message}`); resolve(null); }
      });
      // Resume stdin in case it's paused
      process.stdin.resume();
    } catch (e) {
      clearTimeout(timer);
      log(`STDIN_SETUP_ERR: ${e.message}`);
      resolve(null);
    }
  });
}

// ── Main ──
(async () => {
  try {
    const raw = await readStdin(3000);
    if (!raw || raw.trim().length === 0) {
      process.exit(0);
    }

    const envelope = buildEnvelope(raw);
    if (!envelope) {
      log(`PARSE_FAIL: ${raw.substring(0, 200)}`);
      process.exit(0);
    }

    await postEvent(envelope);
  } catch (e) {
    log(`FATAL: ${e.message}`);
  }
  process.exit(0);
})();
