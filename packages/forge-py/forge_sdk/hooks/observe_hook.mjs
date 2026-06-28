#!/usr/bin/env node
/**
 * Forge Observe Hook — Claude Code integration.
 *
 * Receives Claude Code hook events via stdin (fire-and-forget).
 * POSTs to Forge server for real-time harness analysis.
 * NEVER crashes, NEVER blocks — always exits 0.
 */

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

// ── Config ──
// FORGE_SERVER_URL overrides everything — use for cloud/remote setups.
// Example: https://my-forge.ngrok.io or http://192.168.1.50:3001
const INGEST_URL = (() => {
  if (process.env.FORGE_SERVER_URL) {
    const base = process.env.FORGE_SERVER_URL.replace(/\/+$/, '');
    return `${base}/api/v1/ingest/event`;
  }
  const port = (() => {
    try {
      const pf = path.join(os.homedir(), '.forge', 'port');
      if (fs.existsSync(pf)) return parseInt(fs.readFileSync(pf, 'utf8').trim(), 10) || 3000;
    } catch (_) {}
    return parseInt(process.env.FORGE_SERVER_PORT, 10) || 3000;
  })();
  return `http://127.0.0.1:${port}/api/v1/ingest/event`;
})();

// ── Fire-and-forget HTTP POST ──
function postFireAndForget(data) {
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
        'User-Agent': 'Forge-Observe/1.0',
      },
      timeout: 3000,
    }, (res) => { res.resume(); });
    req.on('error', () => {});
    req.on('timeout', () => { req.destroy(); });
    req.write(body);
    req.end();
  } catch (_) {}
}

// ── Build envelope ──
function buildEnvelope(raw) {
  try {
    const data = typeof raw === 'string' ? JSON.parse(raw) : raw;
    if (!data || !data.hook_event_name) return null;

    const hookName = data.hook_event_name;
    const sessionId = data.session_id || process.env.CLAUDE_SESSION_ID || 'unknown';
    const agentId = data.agent_id || data.subagent_id || 'root';
    const toolName = data.tool_name || '';
    const cwd = data.cwd || process.cwd();

    // Strip large base64 to keep payloads small
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

    return {
      agentClass: 'claude-code',
      sessionId, agentId, hookName, toolName,
      payload: data, cwd,
      timestamp: Date.now(),
      flags,
    };
  } catch (_) { return null; }
}

// ── Read stdin with timeout ──
function readStdin(timeoutMs) {
  return new Promise((resolve) => {
    let input = '';
    let settled = false;
    const timer = setTimeout(() => { if (!settled) { settled = true; resolve(input || null); } }, timeoutMs);
    try {
      process.stdin.setEncoding('utf8');
      process.stdin.on('data', (chunk) => { input += chunk; });
      process.stdin.on('end', () => { if (!settled) { settled = true; clearTimeout(timer); resolve(input || null); } });
      process.stdin.on('error', () => { if (!settled) { settled = true; clearTimeout(timer); resolve(null); } });
    } catch (_) {
      clearTimeout(timer);
      resolve(null);
    }
  });
}

// ── Main ──
(async () => {
  try {
    const raw = await readStdin(2000); // 2s timeout max
    if (!raw) { process.exit(0); }

    const envelope = buildEnvelope(raw);
    if (!envelope) { process.exit(0); }

    postFireAndForget(envelope);
  } catch (_) {}
  process.exit(0);
})();
