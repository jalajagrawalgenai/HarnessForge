#!/usr/bin/env node
/**
 * Forge Observe Hook — Claude Code integration.
 *
 * Receives Claude Code hook events via stdin, enriches them with session context,
 * and POSTs to the Forge server for real-time harness analysis.
 *
 * Subcommands:
 *   hook             Fire-and-forget: reads stdin, POSTs to Forge, exits fast.
 *   hook-sync        Synchronous: POSTs then outputs systemMessage JSON to stdout.
 *   hook-autostart   Like hook-sync, but auto-starts Forge server if unreachable.
 *   health           Check Forge server health.
 *   status           Show hook registration status.
 *
 * If no subcommand given, defaults to "hook" (fire-and-forget).
 *
 * Forge server port discovery (priority order):
 *   --port CLI arg  >  FORGE_SERVER_PORT env var  >  port file (~/.forge/port)  >  scan 3000-3005
 */

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// ══════════════════════════════════════════════════════════
// CLI Args
// ══════════════════════════════════════════════════════════

function parseArgs(argv) {
  const commands = [];
  const flags = {};
  let i = 0;
  while (i < argv.length) {
    const arg = argv[i];
    if (arg === '--port' && i + 1 < argv.length) {
      flags.port = parseInt(argv[++i], 10);
    } else if (arg === '--project-slug' && i + 1 < argv.length) {
      flags.projectSlug = argv[++i];
    } else if (arg === '--force') {
      flags.force = true;
    } else if (arg.startsWith('--')) {
      flags[arg.slice(2)] = argv[i + 1] && !argv[i + 1].startsWith('--') ? argv[++i] : true;
    } else {
      commands.push(arg);
    }
    i++;
  }
  return { commands, ...flags };
}

const cliArgs = parseArgs(process.argv.slice(2));
const subcommand = cliArgs.commands[0] || 'hook';

// ══════════════════════════════════════════════════════════
// Config
// ══════════════════════════════════════════════════════════

const FORGE_STATE_DIR = path.join(os.homedir(), '.forge');
const FORGE_PORT_FILE = path.join(FORGE_STATE_DIR, 'port');
const FORGE_SESSION_FILE = path.join(FORGE_STATE_DIR, 'session.json');
const FORGE_LOG_DIR = path.join(FORGE_STATE_DIR, 'logs');
const FORGE_CLI_LOG = path.join(FORGE_LOG_DIR, 'hook.log');

const PROJECT_SLUG = cliArgs.projectSlug || process.env.FORGE_PROJECT_SLUG ||
                      path.basename(process.cwd());

// ══════════════════════════════════════════════════════════
// Port Discovery
// ══════════════════════════════════════════════════════════

function discoverPort() {
  if (cliArgs.port) return cliArgs.port;
  if (process.env.FORGE_SERVER_PORT) return parseInt(process.env.FORGE_SERVER_PORT, 10);

  // Check port file
  try {
    if (fs.existsSync(FORGE_PORT_FILE)) {
      const p = parseInt(fs.readFileSync(FORGE_PORT_FILE, 'utf8').trim(), 10);
      if (p > 0) return p;
    }
  } catch (_) {}

  return 3000; // default — server will be found via health check
}

const FORGE_PORT = discoverPort();
const FORGE_BASE = `http://127.0.0.1:${FORGE_PORT}`;
const INGEST_URL = `${FORGE_BASE}/api/v1/ingest/event`;
const HEALTH_URL = `${FORGE_BASE}/api/v1/health`;

// ══════════════════════════════════════════════════════════
// Logger
// ══════════════════════════════════════════════════════════

const LOG_LEVELS = { trace: 0, debug: 1, info: 2, warn: 3, error: 4 };
const MIN_LOG_LEVEL = LOG_LEVELS[(process.env.FORGE_LOG_LEVEL || 'info').toLowerCase()] ?? LOG_LEVELS.info;

function log(level, msg) {
  if (LOG_LEVELS[level] < MIN_LOG_LEVEL) return;
  try {
    if (!fs.existsSync(FORGE_LOG_DIR)) fs.mkdirSync(FORGE_LOG_DIR, { recursive: true });
    const ts = new Date().toISOString();
    fs.appendFileSync(FORGE_CLI_LOG, `[${ts}] [${level.toUpperCase()}] ${msg}\n`);
  } catch (_) {}
}

// ══════════════════════════════════════════════════════════
// HTTP Helpers
// ══════════════════════════════════════════════════════════

function postJson(url, data, opts = {}) {
  return new Promise((resolve) => {
    const body = JSON.stringify(data);
    const parsed = new URL(url);

    const options = {
      hostname: parsed.hostname,
      port: parsed.port || 80,
      path: parsed.pathname + parsed.search,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(body),
        'User-Agent': 'Forge-Observe/1.0',
      },
      timeout: opts.timeout || 5000,
    };

    const req = http.request(options, (res) => {
      if (opts.fireAndForget) {
        res.resume();
        resolve({ status: res.statusCode, body: null });
        return;
      }
      let responseData = '';
      res.on('data', (chunk) => { responseData += chunk; });
      res.on('end', () => {
        try {
          resolve({ status: res.statusCode, body: JSON.parse(responseData) });
        } catch {
          resolve({ status: res.statusCode, body: responseData });
        }
      });
    });

    req.on('error', (err) => {
      resolve({ status: 0, error: err.message, body: null });
    });

    req.on('timeout', () => {
      req.destroy();
      resolve({ status: 0, error: 'timeout', body: null });
    });

    req.write(body);
    req.end();
  });
}

function getJson(url, opts = {}) {
  return new Promise((resolve) => {
    const parsed = new URL(url);

    const options = {
      hostname: parsed.hostname,
      port: parsed.port || 80,
      path: parsed.pathname + parsed.search,
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'User-Agent': 'Forge-Observe/1.0',
      },
      timeout: opts.timeout || 5000,
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          resolve({ status: res.statusCode, body: JSON.parse(data) });
        } catch {
          resolve({ status: res.statusCode, body: data });
        }
      });
    });

    req.on('error', (err) => resolve({ status: 0, error: err.message, body: null }));
    req.on('timeout', () => { req.destroy(); resolve({ status: 0, error: 'timeout', body: null }); });
    req.end();
  });
}

// ══════════════════════════════════════════════════════════
// Session ID Detection
// ══════════════════════════════════════════════════════════

function detectCurrentSessionId() {
  // 1. Environment variables
  if (process.env.CLAUDE_SESSION_ID) return process.env.CLAUDE_SESSION_ID;
  if (process.env.CODEX_SESSION_ID) return process.env.CODEX_SESSION_ID;

  // 2. Scan ~/.claude/projects/ for most recent session
  try {
    const projectsDir = path.join(os.homedir(), '.claude', 'projects');
    if (fs.existsSync(projectsDir)) {
      const dirs = fs.readdirSync(projectsDir, { withFileTypes: true });
      let newestId = null, newestTime = 0;
      for (const dir of dirs) {
        if (!dir.isDirectory()) continue;
        // Check last-prompt.json first
        const lastPromptFile = path.join(projectsDir, dir.name, 'last-prompt.json');
        try {
          if (fs.existsSync(lastPromptFile)) {
            const data = JSON.parse(fs.readFileSync(lastPromptFile, 'utf8'));
            if (data.sessionId) return data.sessionId;
          }
        } catch (_) {}
        // Check JSONL files
        try {
          const files = fs.readdirSync(path.join(projectsDir, dir.name));
          for (const f of files) {
            if (!f.endsWith('.jsonl') || f.includes('agent-')) continue;
            const stat = fs.statSync(path.join(projectsDir, dir.name, f));
            if (stat.mtimeMs > newestTime) {
              newestTime = stat.mtimeMs;
              newestId = f.replace('.jsonl', '');
            }
          }
        } catch (_) {}
      }
      if (newestId) return newestId;
    }
  } catch (_) {}

  return null;
}

// ══════════════════════════════════════════════════════════
// Envelope Builder
// ══════════════════════════════════════════════════════════

// Claude Code event names (used for agent class detection)
const CLAUDE_EVENTS = new Set([
  'SessionStart', 'SessionEnd', 'UserPromptSubmit', 'UserPromptExpansion',
  'PreToolUse', 'PostToolUse', 'PostToolUseFailure', 'PostToolBatch',
  'PermissionRequest', 'PermissionDenied', 'Stop', 'StopFailure',
  'SubagentStart', 'SubagentStop', 'TeammateIdle', 'TaskCreated',
  'TaskCompleted', 'Notification', 'InstructionsLoaded', 'ConfigChange',
  'CwdChanged', 'FileChanged', 'PreCompact', 'PostCompact',
  'Elicitation', 'ElicitationResult', 'WorktreeRemove', 'Setup',
]);

function getAgentClass(hookPayload) {
  const data = hookPayload || {};
  const eventName = data.hook_event_name || data.event || data.type || '';
  if (CLAUDE_EVENTS.has(eventName)) return 'claude-code';
  if (data.source === 'codex' || eventName.startsWith('codex_')) return 'codex';
  return 'default';
}

// Strip large base64 image data to keep payloads manageable
function stripLargeImageData(hookPayload, maxChars = 500000) {
  if (!maxChars || maxChars <= 0) return;
  const resp = hookPayload?.tool_response;
  if (!Array.isArray(resp)) return;
  for (const item of resp) {
    if (!item || typeof item !== 'object') continue;
    if (item.type !== 'image') continue;
    const src = item.source;
    if (!src || typeof src !== 'object') continue;
    if (src.type !== 'base64') continue;
    if (typeof src.data === 'string' && src.data.length > maxChars) {
      src.data = '[REDACTED]';
    }
  }
}

// Normalize various timestamp formats to epoch millis
function normalizeTimestamp(ts) {
  if (typeof ts === 'number') return ts;
  if (typeof ts === 'string') {
    const d = new Date(ts);
    if (!isNaN(d.getTime())) return d.getTime();
  }
  return Date.now();
}

function findLocalTranscript(sessionId, cwd) {
  const home = os.homedir();
  const projectsDir = path.join(home, '.claude', 'projects');

  // 1. Direct match
  const directPath = path.join(projectsDir, `${sessionId}.jsonl`);
  try { if (fs.existsSync(directPath)) return directPath; } catch (_) {}

  // 2. Search subdirectories
  try {
    if (fs.existsSync(projectsDir)) {
      const entries = fs.readdirSync(projectsDir, { withFileTypes: true });
      for (const entry of entries) {
        if (!entry.isDirectory()) continue;
        const candidate = path.join(projectsDir, entry.name, `${sessionId}.jsonl`);
        try { if (fs.existsSync(candidate)) return candidate; } catch (_) {}
      }
    }
  } catch (_) {}

  // 3. CWD-based search
  if (cwd) {
    const projectName = path.basename(cwd).replace(/[^a-zA-Z0-9_-]/g, '-');
    const projectDir = path.join(projectsDir, projectName);
    try {
      if (fs.existsSync(projectDir)) {
        const files = fs.readdirSync(projectDir);
        const jsonlFiles = files.filter(f => f.endsWith('.jsonl') && !f.includes('agent-'));
        if (jsonlFiles.length > 0) {
          let best = null, bestTime = 0;
          for (const f of jsonlFiles) {
            const stat = fs.statSync(path.join(projectDir, f));
            if (stat.mtimeMs > bestTime) { best = path.join(projectDir, f); bestTime = stat.mtimeMs; }
          }
          if (best) return best;
        }
      }
    } catch (_) {}
  }

  // 4. Last resort: most recent JSONL in any project dir
  try {
    if (fs.existsSync(projectsDir)) {
      const entries = fs.readdirSync(projectsDir, { withFileTypes: true });
      let best = null, bestTime = 0;
      for (const entry of entries) {
        if (!entry.isDirectory()) continue;
        try {
          const files = fs.readdirSync(path.join(projectsDir, entry.name));
          for (const f of files) {
            if (!f.endsWith('.jsonl') || f.includes('agent-')) continue;
            const stat = fs.statSync(path.join(projectsDir, entry.name, f));
            if (stat.mtimeMs > bestTime) { best = path.join(projectsDir, entry.name, f); bestTime = stat.mtimeMs; }
          }
        } catch (_) {}
      }
      if (best) return best;
    }
  } catch (_) {}

  return null;
}

function buildEnvelope(hookPayload) {
  const data = hookPayload || {};
  const agentClass = getAgentClass(data);

  stripLargeImageData(data, 500000);

  const hookName = data.hook_event_name || data.event || 'unknown';
  const toolName = data.tool_name || data.tool_input?.tool || data.tool || '';
  const sessionId = data.session_id || detectCurrentSessionId() || 'unknown';
  const agentId = data.agent_id || data.subagent_id || `agent_${sessionId}_root`;

  const isSubagent = hookName === 'SubagentStart' || hookName === 'SubagentStop';
  const agentMeta = {
    name: data.agent_name || data.subagent_name || null,
    description: data.subagent_description || null,
    type: isSubagent ? 'subagent' : data.agent_type || 'primary',
  };

  let transcriptPath = data.transcript_path || null;
  if (!transcriptPath && sessionId !== 'unknown') {
    transcriptPath = findLocalTranscript(sessionId, data.cwd || process.cwd());
  }
  if (transcriptPath) {
    transcriptPath = transcriptPath.replace(/\\/g, '/');
  }

  const sessionMeta = {
    slug: null,
    transcriptPath,
    metadata: null,
    startCwd: data.cwd || process.cwd(),
  };

  const projectMeta = {
    slug: PROJECT_SLUG,
  };

  const flags = {};
  if (hookName === 'Notification' || hookName === 'PostToolUseFailure') {
    flags.startsNotification = true;
  }
  if (hookName === 'Stop' || hookName === 'SessionEnd') {
    flags.stopsSession = true;
  }
  if (hookName === 'SessionStart') {
    flags.clearsNotification = true;
    flags.startsSession = true;
  }

  const timestamp = data.timestamp ? normalizeTimestamp(data.timestamp) : Date.now();

  return {
    agentClass,
    sessionId,
    agentId,
    hookName,
    toolName,
    payload: data,
    cwd: data.cwd || process.cwd(),
    timestamp,
    _meta: {
      agent: agentMeta,
      session: sessionMeta,
      project: projectMeta,
    },
    flags,
  };
}

// ══════════════════════════════════════════════════════════
// Transcript Parsing (for SessionEnd auto-sync)
// ══════════════════════════════════════════════════════════

function parseTranscriptFile(transcriptPath) {
  const empty = {
    models: [],
    totalInputTokens: 0,
    totalOutputTokens: 0,
    totalCacheRead: 0,
    totalCacheCreate5m: 0,
    totalCacheCreate1h: 0,
    totalCacheWrite: 0,
    cacheHitRate: 0,
    modelBreakdown: {},
    entryCount: 0,
  };

  if (!transcriptPath || !fs.existsSync(transcriptPath)) return empty;

  let content;
  try { content = fs.readFileSync(transcriptPath, 'utf8'); } catch { return empty; }

  const lines = content.trim().split('\n');
  const modelSet = new Set();
  const modelBreakdown = {};
  let totalInput = 0, totalOutput = 0;
  let totalCacheRead = 0, totalCacheCreate5m = 0, totalCacheCreate1h = 0;
  let totalCacheWrite = 0;

  for (const line of lines) {
    try {
      const entry = JSON.parse(line);
      if (!entry || entry.type === 'system') continue;

      const msg = entry.message || entry;
      const usage = msg.usage || (msg.content ? null : null);

      // Extract from message.usage (Anthropic API format)
      if (msg.usage) {
        totalInput += msg.usage.input_tokens || 0;
        totalOutput += msg.usage.output_tokens || 0;
        if (msg.usage.cache_read_input_tokens) totalCacheRead += msg.usage.cache_read_input_tokens;
        if (msg.usage.cache_creation_input_tokens) totalCacheCreate5m += msg.usage.cache_creation_input_tokens;
      }

      // Extract model
      if (msg.model) {
        modelSet.add(msg.model);
        modelBreakdown[msg.model] = (modelBreakdown[msg.model] || 0) + 1;
      }
    } catch (_) {}
  }

  const totalCacheCreate = totalCacheCreate5m + totalCacheCreate1h;
  const totalCacheOps = totalCacheRead + totalCacheCreate;
  const cacheHitRate = totalInput > 0 ? (totalCacheRead / totalInput) : 0;

  return {
    models: [...modelSet],
    totalInputTokens: totalInput,
    totalOutputTokens: totalOutput,
    totalCacheRead,
    totalCacheCreate5m,
    totalCacheCreate1h,
    totalCacheWrite,
    cacheHitRate,
    modelBreakdown,
    entryCount: lines.length,
  };
}

// ══════════════════════════════════════════════════════════
// Server Start Helper
// ══════════════════════════════════════════════════════════

function tryStartServer() {
  // On Windows, use `forge serve` via Python
  try {
    const { spawn } = require('child_process');
    const cmd = process.platform === 'win32' ? 'python' : 'python3';
    const child = spawn(cmd, ['-m', 'forge_sdk.cli', 'serve'], {
      detached: true,
      stdio: 'ignore',
      env: { ...process.env, FORGE_SERVER_PORT: String(FORGE_PORT) },
    });
    child.unref();
    log('info', `Attempted to start Forge server on port ${FORGE_PORT}`);
    return true;
  } catch (err) {
    log('error', `Failed to start Forge server: ${err.message}`);
    return false;
  }
}

// ══════════════════════════════════════════════════════════
// I/O Helpers
// ══════════════════════════════════════════════════════════

function readStdin() {
  return new Promise((resolve) => {
    let input = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => { input += chunk; });
    process.stdin.on('end', () => resolve(input.trim() || null));
  });
}

function outputSystemMessage(message) {
  process.stdout.write(JSON.stringify({ systemMessage: message }) + '\n');
}

// ══════════════════════════════════════════════════════════
// Hook Commands
// ══════════════════════════════════════════════════════════

/** Fire-and-forget hook — reads stdin, POSTs to Forge, exits immediately. */
async function hookCommand() {
  const input = await readStdin();
  if (!input) return;

  let hookPayload;
  try {
    hookPayload = JSON.parse(input);
  } catch (err) {
    log('warn', `Failed to parse hook payload: ${err.message}`);
    return;
  }

  const envelope = buildEnvelope(hookPayload);
  log('debug', `${envelope.hookName}${envelope.toolName ? ' tool=' + envelope.toolName : ''} session=${envelope.sessionId.slice(0, 12)}`);

  const result = await postJson(INGEST_URL, envelope, { fireAndForget: true });

  if (result.status === 0) {
    log('warn', `Forge server unreachable at ${FORGE_BASE}: ${result.error}`);
    // Try to auto-start the server silently
    if (envelope.hookName === 'SessionStart') {
      log('info', 'Attempting to auto-start Forge server...');
      tryStartServer();
    }
  } else if (result.status >= 400) {
    log('warn', `Forge server returned HTTP ${result.status} for ${envelope.hookName}`);
  }

  // On session end, sync transcript data
  if (envelope.hookName === 'SessionEnd' || envelope.hookName === 'Stop') {
    const sessionId = envelope.sessionId;
    const transcriptPath = envelope._meta?.session?.transcriptPath || findLocalTranscript(sessionId, envelope.cwd);
    if (transcriptPath && fs.existsSync(transcriptPath)) {
      log('info', `Syncing transcript for session ${sessionId.slice(0, 12)}`);
      try {
        const data = parseTranscriptFile(transcriptPath);
        if (data.entryCount > 0) {
          const syncUrl = `${FORGE_BASE}/api/v1/ingest/transcript`;
          await postJson(syncUrl, {
            sessionId,
            transcriptPath,
            ...data,
          }, { timeout: 10000 });
        }
      } catch (err) {
        log('warn', `Transcript sync failed: ${err.message}`);
      }
    }
  }
}

/** Synchronous hook — POSTs and outputs systemMessage for Claude Code to display. */
async function hookSyncCommand() {
  // Silence console so only our JSON goes to stdout
  const noop = () => {};
  console.log = noop;
  console.error = noop;
  console.warn = noop;
  console.debug = noop;

  try {
    const input = await readStdin();
    if (!input) {
      outputSystemMessage('Forge: no hook data received.');
      return;
    }

    let hookPayload;
    try {
      hookPayload = JSON.parse(input);
    } catch (err) {
      outputSystemMessage(`Forge: invalid hook data — ${err.message}`);
      return;
    }

    const envelope = buildEnvelope(hookPayload);
    const result = await postJson(INGEST_URL, envelope);

    if (!result || result.status === 0) {
      outputSystemMessage('Forge harness is not running. Run `forge serve` to start.');
      return;
    }

    if (result.status === 200 && result.body) {
      if (result.body.systemMessage) {
        outputSystemMessage(result.body.systemMessage);
      } else if (result.body.intervention) {
        const intv = result.body.intervention;
        outputSystemMessage(`Forge: ${intv.type} intervention — ${intv.message}`);
      } else if (envelope.hookName === 'SessionStart') {
        outputSystemMessage(`Forge harness active. Dashboard: ${FORGE_BASE}`);
      }
    }
  } catch (err) {
    outputSystemMessage(`Forge error: ${err.message}`);
  }
}

/** Auto-start hook — tries to start Forge server if unreachable, then sync. */
async function hookAutostartCommand() {
  const health = await getJson(HEALTH_URL);
  if (health.status === 0 || health.status >= 400) {
    log('info', 'Forge server not reachable, attempting auto-start...');
    tryStartServer();
    // Give it a moment to start
    await new Promise(r => setTimeout(r, 2000));
  }
  await hookSyncCommand();
}

// ══════════════════════════════════════════════════════════
// Management Commands
// ══════════════════════════════════════════════════════════

async function healthCommand() {
  console.log(`Checking Forge server at ${FORGE_BASE}...`);
  const result = await getJson(HEALTH_URL);
  if (result.status === 200 && result.body?.status === 'ok') {
    console.log('Forge server is healthy.');
    console.log(JSON.stringify(result.body, null, 2));
  } else if (result.status === 0) {
    console.log('Forge server is NOT running.');
    console.log(`  Start it with: forge serve`);
  } else {
    console.log(`Forge server error (HTTP ${result.status})`);
    console.log(JSON.stringify(result.body, null, 2));
  }
}

async function statusCommand() {
  console.log('Forge Observe Hook Status');
  console.log('=========================');
  console.log(`  State dir:  ${FORGE_STATE_DIR}`);
  console.log(`  Port:       ${FORGE_PORT}`);
  console.log(`  Base URL:   ${FORGE_BASE}`);
  console.log(`  Project:    ${PROJECT_SLUG}`);
  console.log();

  // Check if registered in Claude Code settings
  const settingsPath = path.join(os.homedir(), '.claude', 'settings.json');
  try {
    if (fs.existsSync(settingsPath)) {
      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      const hooks = settings.hooks || {};
      let registered = false;
      for (const [event, hooksList] of Object.entries(hooks)) {
        for (const h of hooksList) {
          for (const hh of (h.hooks || [])) {
            if (hh.command && hh.command.includes('forge')) {
              console.log(`  Registered: ${event} → ${hh.command}`);
              registered = true;
            }
          }
        }
      }
      if (!registered) {
        console.log('  NOT registered in Claude Code settings.');
        console.log('  Run `forge serve` to auto-register.');
      }
    } else {
      console.log('  No Claude Code settings found.');
    }
  } catch (err) {
    console.log(`  Error reading settings: ${err.message}`);
  }
  console.log();

  // Check server health
  const health = await getJson(HEALTH_URL);
  if (health.status === 200) {
    console.log('  Server:     RUNNING');
  } else {
    console.log('  Server:     OFFLINE (run `forge serve` to start)');
  }
}

// ══════════════════════════════════════════════════════════
// Main Dispatch
// ══════════════════════════════════════════════════════════

(async () => {
  switch (subcommand) {
    case 'hook':
      await hookCommand();
      break;
    case 'hook-sync':
      await hookSyncCommand();
      break;
    case 'hook-autostart':
      await hookAutostartCommand();
      break;
    case 'health':
      await healthCommand();
      break;
    case 'status':
      await statusCommand();
      break;
    case 'help':
    case '--help':
    case '-h':
      console.log('Forge Observe Hook — Claude Code integration');
      console.log('');
      console.log('Subcommands:');
      console.log('  hook            Fire-and-forget: POST event and exit');
      console.log('  hook-sync       Synchronous: returns systemMessage');
      console.log('  hook-autostart  Auto-start Forge server, then sync');
      console.log('  health          Check Forge server health');
      console.log('  status          Show registration status');
      console.log('');
      console.log('Options:');
      console.log('  --port N        Forge server port (default: auto-detect)');
      console.log('  --project-slug  Project identifier');
      break;
    default:
      // Default to fire-and-forget hook
      await hookCommand();
  }
})();
