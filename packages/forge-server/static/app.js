// Forge Dashboard — Agent Harness
var API_BASE = '/api';

function api(path, opts) {
  opts = opts || {};
  return fetch(API_BASE + path, opts).then(function(r) { return r.json(); });
}

var currentPage = 'run';

function showPage(name) {
  currentPage = name;
  document.querySelectorAll('#nav-tabs a').forEach(function(a) {
    a.classList.toggle('active', a.getAttribute('data-page') === name);
  });
  var el = document.getElementById('content');
  el.innerHTML = '<div class="card"><p>Loading...</p></div>';
  try {
    if (name === 'run') renderMonitor();
    else if (name === 'sessions') renderSessions();
    else if (name === 'audit') renderAudit();
    else if (name === 'analytics') renderAnalytics();
    else if (name === 'meta') renderMeta();
    else if (name === 'settings') renderSettings();
  } catch(e) { el.innerHTML = '<div class="card"><p>Error: ' + e.message + '</p></div>'; }
}

document.addEventListener('DOMContentLoaded', function() { showPage('run'); });

// ═══════════════════════════════════════════════════════════
// MONITOR — Live status + ingest test
// ═══════════════════════════════════════════════════════════

function renderMonitor() {
  document.getElementById('content').innerHTML =
    '<div class="card" style="border-left:3px solid var(--accent-green)"><h2>Live Monitoring</h2>' +
    '<div id="ingest-status"><p>Checking for agent activity...</p></div></div>' +
    '<div class="card" id="cloud-connect-card"><h2>☁️ Cloud Session Setup</h2>' +
    '<div id="cloud-info"><p>Loading connection info...</p></div></div>' +
    '<div class="card"><h2>Quick Stats</h2><div id="quick-stats"><p>Loading stats...</p></div></div>';
  checkIngestStatus();
  showCloudInfo();
  refreshStats();
}

function checkIngestStatus() {
  api('/v1/sessions').then(function(d) {
    var el = document.getElementById('ingest-status');
    if (!el) return;
    var sessions = d.sessions || [];
    var running = sessions.filter(function(s) { return s.status === 'running'; });
    var cc = sessions.filter(function(s) { return s.agent_type === 'claude-code'; });
    var cursor = sessions.filter(function(s) { return s.agent_type === 'cursor'; });
    var ag = sessions.filter(function(s) { return s.agent_type === 'antigravity'; });

    el.innerHTML =
      '<div style="display:flex;gap:12px;flex-wrap:wrap">' +
      '<div class="stat-card" style="flex:1;min-width:140px"><div class="stat-value" style="color:' + (cc.length > 0 ? 'var(--accent-green)' : 'var(--accent-red)') + '">' + (cc.length > 0 ? '●' : '○') + '</div><div class="stat-label">Claude Code' + (cc.length > 0 ? ' (' + cc.length + ')' : ' (not detected)') + '</div></div>' +
      '<div class="stat-card" style="flex:1;min-width:140px"><div class="stat-value" style="color:' + (cursor.length > 0 ? 'var(--accent-purple)' : 'var(--text-secondary)') + '">' + (cursor.length > 0 ? '●' : '○') + '</div><div class="stat-label">Cursor' + (cursor.length > 0 ? ' (' + cursor.length + ')' : ' (not detected)') + '</div></div>' +
      '<div class="stat-card" style="flex:1;min-width:140px"><div class="stat-value" style="color:' + (ag.length > 0 ? 'var(--accent-green)' : 'var(--text-secondary)') + '">' + (ag.length > 0 ? '●' : '○') + '</div><div class="stat-label">Antigravity' + (ag.length > 0 ? ' (' + ag.length + ')' : ' (not detected)') + '</div></div>' +
      '</div>' +
      '<p style="margin-top:8px;font-size:13px;color:var(--text-secondary)">' +
      '<strong>' + sessions.length + '</strong> total sessions · <strong>' + running.length + '</strong> active · ' +
      'Events auto-detected from Claude Code hooks. Cursor/Antigravity: POST to ingest endpoint to connect.</p>';
    if (running.length > 0) el.innerHTML += '<p style="margin-top:4px"><a href="javascript:showPage(\'sessions\')">View all sessions →</a></p>';
  }).catch(function() {
    var el = document.getElementById('ingest-status');
    if (el) el.innerHTML = '<p style="color:var(--text-secondary)">Connecting...</p>';
  });
  setTimeout(function() { if (currentPage === 'run') checkIngestStatus(); }, 5000);
}

function showCloudInfo() {
  var el = document.getElementById('cloud-info');
  if (!el) return;
  var hookUrl = window.location.origin + '/api/v1/ingest/event';
  el.innerHTML =
    '<div style="background:#1e1e2e;border-radius:8px;padding:12px;margin:8px 0">' +
    '<p style="margin:0 0 8px 0;font-weight:600;color:var(--accent-blue)">Universal Ingest Endpoint:</p>' +
    '<code style="font-size:13px;word-break:break-all;color:var(--accent-green)">' + hookUrl + '</code></div>' +

    '<div class="stats-grid" style="grid-template-columns:repeat(3,1fr);gap:8px;margin:12px 0">' +

    '<div class="card" style="border-left:3px solid var(--accent-blue)"><h4>🔧 Claude Code</h4>' +
    '<p style="font-size:12px;color:var(--accent-green)">✅ Auto-connected</p>' +
    '<p style="font-size:11px;color:var(--text-secondary)">Hooks registered in settings.json. Local sessions appear automatically.</p>' +
    '<p style="font-size:11px;color:var(--text-secondary)">Remote: <code>export FORGE_SERVER_URL=...</code></p></div>' +

    '<div class="card" style="border-left:3px solid var(--accent-purple)"><h4>🖥️ Cursor / VS Code</h4>' +
    '<p style="font-size:12px;color:var(--accent-yellow)">⚡ Via REST</p>' +
    '<p style="font-size:11px;color:var(--text-secondary)">POST session events to the ingest endpoint. Use the Forge VS Code extension or curl.</p>' +
    '<p style="font-size:11px"><code>curl -X POST ' + hookUrl.substring(0,40) + '...</code></p></div>' +

    '<div class="card" style="border-left:3px solid var(--accent-green)"><h4>🌐 Any Agent</h4>' +
    '<p style="font-size:12px;color:var(--accent-green)">✅ Universal API</p>' +
    '<p style="font-size:11px;color:var(--text-secondary)">Antigravity, LangGraph, CrewAI, AutoGen, etc. — POST events with <code>agentClass</code> field.</p>' +
    '<p style="font-size:11px;color:var(--text-secondary)">All 31 agent types supported.</p></div>' +

    '</div>' +

    '<p style="margin:8px 0;font-size:12px;color:var(--text-secondary)">' +
    'Cloud/remote: <code>ngrok http ' + window.location.port + '</code> → <code>FORGE_SERVER_URL=https://xxxx.ngrok.io</code> on remote machine.' +
    '</p>';
}

function refreshStats() {
  api('/v1/ingest/status').then(function(s) {
    document.getElementById('quick-stats').innerHTML =
      '<div class="stats-grid">' +
      '<div class="stat-card"><div class="stat-value">' + (s.totalSessions||0) + '</div><div class="stat-label">Sessions</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">' + (s.activeSessions||0) + '</div><div class="stat-label">Active</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-yellow)">' + (s.totalObservations||0) + '</div><div class="stat-label">Observations</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-red)">' + (s.totalDetections||0) + '</div><div class="stat-label">Detections</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-blue)">' + (s.totalInterventions||0) + '</div><div class="stat-label">Interventions</div></div>' +
      '<div class="stat-card"><div class="stat-value">' + (s.totalEventsInRing||0) + '</div><div class="stat-label">Events</div></div></div>';
  });
}

// ═══════════════════════════════════════════════════════════
// SESSIONS
// ═══════════════════════════════════════════════════════════

function renderSessions() {
  api('/v1/sessions').then(function(d) {
    var rows = (d.sessions||[]).map(function(s) {
      var hs = s.health_score;
      var healthHtml = '-';
      if (hs && hs.overall !== undefined) {
        var pct = Math.round(hs.overall * 100);
        var hColor = hs.overall > 0.8 ? 'var(--accent-green)' : hs.overall > 0.5 ? 'var(--accent-yellow)' : 'var(--accent-red)';
        healthHtml = '<span style="color:' + hColor + ';font-weight:600">' + pct + '%</span>';
      }
      var pipe = s.pipeline || {};
      var tokens = (s.total_tokens||0);
      var modelShort = (s.model||'?').substring(0,15);
      var tokenStr = tokens > 0 ? (tokens >= 1000 ? (tokens/1000).toFixed(1) + 'k' : tokens) : '-';
      return '<tr><td><code>' + (s.id||'').substring(0,12) + '</code></td><td>' + (s.task||'').substring(0,30) + '</td><td>' + (s.agent_type||'') + '</td><td><span class="badge badge-' + (s.status||'running') + '">' + s.status + '</span></td><td>' + healthHtml + '</td><td>' + tokenStr + '</td><td>' + modelShort + '</td><td>' + (pipe.observation_count||0) + '/' + (pipe.detection_count||0) + '/' + (pipe.intervention_count||0) + '</td><td><a href="javascript:showAnalysis(\'' + s.id + '\')">Analysis</a></td></tr>';
    }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Sessions</h2>' +
      '<p style="color:var(--text-secondary);margin-bottom:8px">O/D/I = Observations / Detections / Interventions</p>' +
      '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th>Health</th><th>Tokens</th><th>Model</th><th>O/D/I</th><th></th></tr></thead><tbody>' +
      (rows || '<tr><td colspan="9">No sessions yet — sessions appear automatically when you use Claude Code or other agents</td></tr>') +
      '</tbody></table></div>';
  });
}

// ═══════════════════════════════════════════════════════════
// ANALYSIS — Full per-session harness report
// ═══════════════════════════════════════════════════════════

function showAnalysis(id) {
  currentPage = 'sessions';
  document.getElementById('content').innerHTML = '<div style="max-width:1200px;margin:0 auto"><div id="ana-header"><p>Loading harness analysis...</p></div><div id="ana-tabs"></div><div id="ana-body"></div></div>';
  api('/v1/sessions/' + id + '/analysis').then(function(a) {
    if (a.error) { document.getElementById('content').innerHTML = '<div class="card"><h2>Error</h2><p>' + a.error + '</p></div>'; return; }
    currentAnalysis = a;
    currentAnalysisId = id;
    renderAnaTabs(a, id);
    renderAnaOverview(a, id);
  }).catch(function(e) {
    document.getElementById('content').innerHTML = '<div class="card"><h2>Error</h2><p>' + e.message + '</p></div>';
  });
}
var currentAnalysis = null;
var currentAnalysisId = null;
var currentAnaTab = 'overview';

function renderAnaTabs(a, id) {
  var tabs = [
    {id:'overview', label:'Overview', icon:'📊'},
    {id:'timeline', label:'Timeline', icon:'📋'},
    {id:'toolsprompts', label:'Tools & Prompts', icon:'🛠'},
    {id:'detections', label:'Detections', icon:'⚠'},
    {id:'hooks', label:'Hooks & Context', icon:'🔗'},
  ];
  document.getElementById('ana-tabs').innerHTML = '<div class="flex-row" style="gap:4px;margin-bottom:12px;flex-wrap:wrap">' +
    tabs.map(function(t) {
      return '<button onclick="switchAnaTab(\'' + t.id + '\')" id="anatab-' + t.id + '" style="padding:8px 16px;font-size:13px;border-radius:6px;' + (t.id === 'overview' ? 'background:var(--accent-blue);color:#fff' : 'background:var(--bg-secondary)') + '">' + t.icon + ' ' + t.label + '</button>';
    }).join('') +
    '<button onclick="generateReport(\'' + id + '\')" class="success" style="margin-left:auto;padding:8px 16px;font-size:13px">📥 Download Report</button>' +
    '<button onclick="showPage(\'sessions\')" style="padding:8px 16px;font-size:13px">← Back</button></div>';
}

function switchAnaTab(tab) {
  currentAnaTab = tab;
  ['overview','timeline','toolsprompts','detections','hooks'].forEach(function(t) {
    var btn = document.getElementById('anatab-' + t);
    if (btn) btn.style.background = t === tab ? 'var(--accent-blue)' : 'var(--bg-secondary)';
    if (btn) btn.style.color = t === tab ? '#fff' : '';
  });
  var a = currentAnalysis;
  var id = currentAnalysisId;
  if (tab === 'overview') renderAnaOverview(a, id);
  else if (tab === 'timeline') renderAnaTimeline(a, id);
  else if (tab === 'toolsprompts') renderAnaToolsPrompts(a, id);
  else if (tab === 'detections') renderAnaDetections(a, id);
  else if (tab === 'hooks') renderAnaHooks(a, id);
}

// ═══ TAB 1: OVERVIEW ═══
function renderAnaOverview(a, id) {
  var tk = a.token_analysis || {};
  var sm = a.session_summary || {};
  var ts = a.tool_summary || {};
  var cx = a.context_analysis || {};
  var health = a.health_score;
  var healthPct = health ? Math.round(health.overall * 100) : 100;
  var healthColor = healthPct > 80 ? 'var(--accent-green)' : healthPct > 50 ? 'var(--accent-yellow)' : 'var(--accent-red)';

  document.getElementById('ana-header').innerHTML =
    '<div style="display:flex;justify-content:space-between;align-items:flex-start;flex-wrap:wrap;gap:16px;margin-bottom:8px">' +
    '<div><h2 style="margin:0;font-size:20px">' + escapeHtml((a.task||'Untitled').substring(0,80)) + '</h2>' +
    '<p style="margin:4px 0;color:var(--text-secondary);font-size:13px"><strong>' + a.agent_type + '</strong> · ' + a.status + ' · ' + (a.duration_display||'?') + ' · <code style="font-size:11px">' + (id||'').substring(0,12) + '</code></p>' +
    '<p style="margin:4px 0;font-size:13px">' + (a.stop_analysis||'') + '</p></div>' +
    '<div style="text-align:right;min-width:120px"><div style="font-size:42px;font-weight:700;color:' + healthColor + '">' + healthPct + '%</div><div style="font-size:12px;color:var(--text-secondary)">Health Score</div></div></div>';

  document.getElementById('ana-body').innerHTML =
    // Cost + Token stats cards
    '<div class="stats-grid" style="grid-template-columns:repeat(4,1fr);gap:12px;margin-bottom:16px">' +
    '<div class="stat-card" style="border-left:3px solid var(--accent-green)"><div class="stat-value" style="font-size:22px;color:var(--accent-green)">$' + (tk.effective_cost_usd||0).toFixed(4) + '</div><div class="stat-label">Cost' + (tk.is_estimated ? ' (est.)' : '') + '</div></div>' +
    '<div class="stat-card" style="border-left:3px solid var(--accent-blue)"><div class="stat-value">' + (tk.total_tokens||0).toLocaleString() + '</div><div class="stat-label">Tokens (in:' + (tk.input_tokens||0).toLocaleString() + ' / out:' + (tk.output_tokens||0).toLocaleString() + ')</div></div>' +
    '<div class="stat-card" style="border-left:3px solid var(--accent-purple)"><div class="stat-value" style="font-size:13px">' + (a.model_family||a.model||'?') + '</div><div class="stat-label">Model · ' + (a.model||'?') + '</div></div>' +
    '<div class="stat-card" style="border-left:3px solid var(--accent-yellow)"><div class="stat-value">' + (tk.cache_hit_pct||0).toFixed(0) + '%</div><div class="stat-label">Cache Hit · ' + (tk.data_source||'?') + '</div></div>' +
    '</div>' +

    // Pricing detail
    '<div class="card" style="margin-bottom:12px"><div style="display:flex;gap:16px;flex-wrap:wrap;font-size:12px">' +
    '<span>📊 <strong>Pricing:</strong> ${:.2f}/1M in · ${:.2f}/1M out</span>'.replace('{:.2f}', (tk.input_price_per_1m||0).toFixed(2)).replace('{:.2f}', (tk.output_price_per_1m||0).toFixed(2)) +
    '<span>📐 <strong>Tokens/event:</strong> ' + (tk.tokens_per_event||0).toFixed(0) + '</span>' +
    '<span>📤 <strong>Output:Input ratio:</strong> ' + (tk.output_to_input_ratio||0).toFixed(2) + '</span>' +
    '<span>💰 <strong>Gross:</strong> $' + (tk.gross_cost_usd||0).toFixed(4) + ' · <strong>Saved:</strong> $' + (tk.cache_savings_usd||0).toFixed(4) + '</span>' +
    '</div></div>' +

    // Summary grid
    '<div class="stats-grid" style="grid-template-columns:repeat(4,1fr);gap:12px;margin-bottom:16px">' +
    '<div class="stat-card"><div class="stat-value">' + (sm.total_events||0) + '</div><div class="stat-label">Total Events</div></div>' +
    '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">' + (sm.user_prompts||0) + '</div><div class="stat-label">User Prompts</div></div>' +
    '<div class="stat-card"><div class="stat-value">' + (ts.total_calls||0) + '</div><div class="stat-label">Tool Calls (' + (ts.total_errors||0) + ' errors)</div></div>' +
    '<div class="stat-card"><div class="stat-value">' + (sm.subagents_spawned||0) + '</div><div class="stat-label">Subagents</div></div>' +
    '</div>' +

    // Tool breakdown
    '<div class="card" style="margin-bottom:12px"><h3>🛠 Tool Usage</h3>' +
    '<table><thead><tr><th>Tool</th><th>Calls</th><th>Errors</th><th>Error Rate</th><th>% of Total</th></tr></thead><tbody>' +
    ((ts.by_tool||[]).map(function(t) {
      return '<tr><td><strong>' + t.tool + '</strong></td><td>' + t.calls + '</td><td>' + (t.errors > 0 ? '<span style="color:var(--accent-red)">' + t.errors + '</span>' : '0') + '</td><td>' + (t.error_rate_pct||0).toFixed(0) + '%</td><td>' + (t.pct_of_total||0).toFixed(0) + '%</td></tr>';
    }).join('') || '<tr><td colspan="5">No tools used</td></tr>') +
    '</tbody></table></div>' +

    // Health dimensions
    '<div class="card" style="margin-bottom:12px"><h3>❤ Health Dimensions</h3>' +
    '<p style="font-size:12px;color:var(--text-secondary);margin-bottom:8px">' + (a.health_verdict||'No health data') + '</p>' +
    (health ? '<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:6px;font-size:12px">' +
      ['token_efficiency','latency','cost','accuracy','orchestration','security','reliability','context_quality','compliance'].map(function(d) {
        var val = (health.dimensions && health.dimensions[d]) ? health.dimensions[d] : 1.0;
        var pct = Math.round(val * 100);
        var c = pct > 80 ? 'var(--accent-green)' : pct > 50 ? 'var(--accent-yellow)' : 'var(--accent-red)';
        return '<div style="display:flex;justify-content:space-between;padding:4px 8px;background:var(--bg-secondary);border-radius:4px"><span>' + d.replace(/_/g,' ') + '</span><span style="color:' + c + ';font-weight:600">' + pct + '%</span></div>';
      }).join('') + '</div>' : '<p style="color:var(--text-secondary)">No health data</p>') +
    '</div>' +

    // Context summary
    '<div class="card" style="margin-bottom:12px"><h3>📐 Context Pressure</h3>' +
    '<div style="display:flex;gap:24px;font-size:13px">' +
    '<span>Readings: <strong>' + (cx.pressure_readings||0) + '</strong></span>' +
    '<span>Average: <strong>' + (cx.avg_pressure_pct||0).toFixed(0) + '%</strong></span>' +
    '<span>Peak: <strong style="color:' + ((cx.max_pressure_pct||0) > 85 ? 'var(--accent-red)' : 'var(--accent-yellow)') + '">' + (cx.max_pressure_pct||0).toFixed(0) + '%</strong></span>' +
    '<span>Status: <strong>' + (cx.status||'healthy') + '</strong></span>' +
    (cx.pressure_history && cx.pressure_history.length > 0 ? '<span>History: [' + cx.pressure_history.join('%, ') + '%]</span>' : '') +
    '</div></div>' +

    // Recommendations
    '<div class="card" style="border-left:4px solid var(--accent-purple);margin-bottom:12px"><h3>🧠 Recommendations</h3>' +
    ((a.recommendations||[]).length > 0 ? a.recommendations.map(function(r) { return '<div style="padding:4px 0">💡 ' + escapeHtml(r) + '</div>'; }).join('') : '<p style="color:var(--text-secondary)">No recommendations</p>') +
    '</div>';
}

// ═══ TAB 2: TIMELINE ═══
function renderAnaTimeline(a, id) {
  var events = a.event_log || [];
  document.getElementById('ana-header').innerHTML = '<h2 style="margin:0">📋 Event Timeline</h2><p style="color:var(--text-secondary)">' + events.length + ' events captured — every agent action in sequence</p>';

  var typeColors = {
    session_start: 'var(--accent-green)',
    session_end: 'var(--accent-green)',
    session_failed: 'var(--accent-red)',
    tool_start: 'var(--accent-blue)',
    tool_end: 'var(--accent-blue)',
    user_prompt: 'var(--accent-purple)',
    context_pressure: 'var(--accent-yellow)',
    subagent_fork: 'var(--accent-yellow)',
    token_usage: 'var(--accent-green)',
    output: 'var(--text-secondary)',
  };

  document.getElementById('ana-body').innerHTML =
    '<div class="card"><div style="max-height:600px;overflow-y:auto">' +
    events.map(function(ev) {
      var color = typeColors[ev.type] || 'var(--text-secondary)';
      var time = (ev.time||'').substring(11,19) || '--:--:--';
      var detailHtml = '';

      if (ev.type === 'tool_end') {
        detailHtml = '<div style="margin-top:4px;font-size:11px;color:var(--text-secondary)">' +
          '<strong>' + (ev.tool||'?') + '</strong>' +
          (ev.is_error ? ' <span style="color:var(--accent-red)">FAILED</span>' : ' <span style="color:var(--accent-green)">ok</span>') +
          (ev.duration_ms > 0 ? ' · ' + ev.duration_ms + 'ms' : '') +
          (ev.token_count > 0 ? ' · ' + ev.token_count + ' tokens' : '') +
          (ev.result ? '<div style="margin-top:4px;padding:6px;background:var(--bg);border-radius:4px;font-family:monospace;white-space:pre-wrap;word-break:break-word;max-height:200px;overflow-y:auto">' + escapeHtml(ev.result) + '</div>' : '') +
          '</div>';
      } else if (ev.type === 'user_prompt') {
        detailHtml = '<div style="margin-top:4px;font-size:12px;white-space:pre-wrap">' + escapeHtml(ev.text||'') + '</div>' +
          '<div style="font-size:10px;color:var(--text-secondary)">' + (ev.text_len||0) + ' chars · ~' + (ev.estimated_tokens||0) + ' tokens</div>';
      } else if (ev.type === 'tool_start') {
        detailHtml = '<div style="margin-top:4px;font-size:12px"><strong>' + (ev.tool||'?') + '</strong>: ' + escapeHtml(ev.args_display||'') + '</div>';
      } else if (ev.type === 'session_start') {
        detailHtml = '<div style="margin-top:4px;font-size:12px">Task: ' + escapeHtml(ev.task||'') + '</div>';
      } else if (ev.type === 'context_pressure') {
        detailHtml = '<div style="margin-top:4px;font-size:12px">Pressure: <strong>' + (ev.pressure_pct||0) + '%</strong> · Trend: ' + (ev.trend||0).toFixed(2) + '</div>';
      } else if (ev.type === 'token_usage') {
        detailHtml = '<div style="margin-top:4px;font-size:11px">' +
          'Input: ' + (ev.input_tokens||0).toLocaleString() + ' · Output: ' + (ev.output_tokens||0).toLocaleString() +
          ' · Cache: ' + (ev.cache_read||0).toLocaleString() + ' · Model: ' + (ev.model||'?') +
          '</div>';
      } else if (ev.type === 'subagent_fork') {
        detailHtml = '<div style="margin-top:4px;font-size:12px">Subagent: <strong>' + (ev.child||'?') + '</strong> — ' + escapeHtml(ev.task||'') + '</div>';
      }

      return '<div style="display:flex;gap:10px;padding:8px 0;border-bottom:1px solid var(--border)">' +
        '<span style="color:var(--text-secondary);font-family:monospace;font-size:11px;min-width:60px;padding-top:2px">' + time + '</span>' +
        '<span style="font-size:14px;min-width:20px;text-align:center">' + (ev.icon||'●') + '</span>' +
        '<span style="color:' + color + ';font-size:11px;min-width:80px;font-weight:600;text-transform:uppercase;padding-top:2px">' + (ev.type||'?').replace(/_/g,' ') + '</span>' +
        '<div style="flex:1">' + detailHtml + '</div>' +
        '<span style="color:var(--text-secondary);font-size:10px;min-width:30px;text-align:right;padding-top:2px">#' + (ev.seq||'?') + '</span>' +
        '</div>';
    }).join('') +
    '</div></div>';
}

// ═══ TAB 3: TOOLS & PROMPTS ═══
function renderAnaToolsPrompts(a, id) {
  var tools = a.tool_instances || [];
  var prompts = a.prompt_instances || [];

  document.getElementById('ana-header').innerHTML = '<h2 style="margin:0">🛠 Tools & Prompts</h2><p style="color:var(--text-secondary)">' + tools.length + ' tool calls · ' + prompts.length + ' prompts</p>';

  var toolsHtml = '<div class="card" style="margin-bottom:12px"><h3>🛠 Every Tool Call</h3>' +
    (tools.length > 0 ? tools.map(function(t) {
      return '<div style="margin:8px 0;padding:10px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid ' + (t.is_error ? 'var(--accent-red)' : 'var(--accent-green)') + '">' +
        '<div style="display:flex;justify-content:space-between;align-items:center">' +
        '<strong>' + t.tool + '</strong>' +
        '<span style="font-size:11px">' + (t.is_error ? '<span style="color:var(--accent-red)">FAILED</span>' : '<span style="color:var(--accent-green)">ok</span>') + ' · ' + (t.duration_ms||0) + 'ms · ' + (t.token_count||0) + ' tokens</span>' +
        '</div>' +
        '<div style="font-size:12px;color:var(--text-secondary);margin-top:4px"><strong>Args:</strong> ' + escapeHtml(t.args_display||'') + '</div>' +
        (t.result ? '<div style="margin-top:6px;padding:8px;background:var(--bg);border-radius:4px;font-family:monospace;font-size:11px;white-space:pre-wrap;word-break:break-word;max-height:150px;overflow-y:auto">' + escapeHtml(t.result) + '</div>' : '') +
        '</div>';
    }).join('') : '<p style="color:var(--text-secondary)">No tool calls recorded</p>') +
    '</div>';

  var promptsHtml = '<div class="card"><h3>💬 Every Prompt</h3>' +
    (prompts.length > 0 ? prompts.map(function(p) {
      return '<div style="margin:8px 0;padding:10px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid var(--accent-purple)">' +
        '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px">' +
        '<strong>Prompt #' + p.index + '</strong>' +
        '<span style="font-size:11px;color:var(--text-secondary)">~' + (p.estimated_input_tokens||0) + ' tokens · ' + (p.following_tool_calls||0) + ' tools after · ' + (p.first_response_latency_ms||0) + 'ms latency</span>' +
        '</div>' +
        '<div style="font-size:13px;line-height:1.5;white-space:pre-wrap">' + escapeHtml(p.text||'') + '</div>' +
        (p.tools_after && p.tools_after.length > 0 ? '<div style="margin-top:6px;font-size:11px;color:var(--text-secondary)">Following tools: ' + p.tools_after.map(function(ft) { return ft.tool + (ft.is_error ? ' (FAILED)' : '') + ' ' + ft.duration_ms + 'ms'; }).join(', ') + '</div>' : '') +
        '</div>';
    }).join('') : '<p style="color:var(--text-secondary)">No prompts captured</p>') +
    '</div>';

  document.getElementById('ana-body').innerHTML = toolsHtml + promptsHtml;
}

// ═══ TAB 4: DETECTIONS ═══
function renderAnaDetections(a, id) {
  var dr = a.detector_report || {};
  var dets = dr.by_category || [];
  var ints = dr.interventions || [];
  var strats = dr.strategies || [];

  document.getElementById('ana-header').innerHTML = '<h2 style="margin:0">⚠ Detections & Interventions</h2><p style="color:var(--text-secondary)">' + (dr.total_detections||0) + ' detections · ' + (dr.total_interventions||0) + ' interventions · ' + (dr.total_strategies||0) + ' strategies</p>';

  var detsHtml = '<div class="card" style="margin-bottom:12px"><h3>⚠ Detections by Category</h3>' +
    (dets.length > 0 ? dets.map(function(cat) {
      var sevColor = cat.severity === 'Critical' ? 'var(--accent-red)' : cat.severity === 'Error' ? 'var(--accent-red)' : cat.severity === 'Warning' ? 'var(--accent-yellow)' : 'var(--accent-blue)';
      return '<div style="margin:10px 0;padding:12px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid ' + sevColor + '">' +
        '<div style="display:flex;justify-content:space-between;align-items:center">' +
        '<strong style="font-size:14px">' + escapeHtml(cat.category||'?') + '</strong>' +
        '<span style="font-size:11px;padding:3px 10px;border-radius:12px;background:' + sevColor + ';color:#000;font-weight:600">' + cat.severity + ' · ' + cat.count + 'x · ' + (cat.avg_confidence*100).toFixed(0) + '% conf</span>' +
        '</div>' +
        (cat.instances||[]).map(function(inst) {
          return '<div style="margin-top:6px;font-size:12px;color:var(--text-secondary);padding:6px 10px;background:var(--bg);border-radius:4px">' + escapeHtml(inst.description||'') + '</div>';
        }).join('') +
        '</div>';
    }).join('') : '<p style="color:var(--accent-green);font-size:14px">✅ No issues detected — agent is running clean</p>') +
    '</div>';

  var intsHtml = '<div class="card" style="margin-bottom:12px"><h3>🔧 Interventions Applied</h3>' +
    (ints.length > 0 ? ints.map(function(i) {
      return '<div style="margin:6px 0;padding:10px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid var(--accent-blue)">' +
        '<strong>' + (i.strategy||'intervention') + '</strong>' +
        '<p style="margin:4px 0 0 0;font-size:12px;color:var(--text-secondary)">' + escapeHtml(i.action||'') + '</p></div>';
    }).join('') : '<p style="color:var(--text-secondary)">No interventions needed</p>') +
    '</div>';

  var stratsHtml = strats.length > 0 ? '<div class="card"><h3>📋 Strategy Evaluations</h3>' +
    strats.map(function(s) {
      return '<div style="margin:6px 0;padding:8px;background:var(--bg-secondary);border-radius:6px;font-size:12px"><strong>' + (s.strategy||'?') + '</strong> → ' + escapeHtml(s.detection||'') + ': ' + escapeHtml(s.intervention||'') + '</div>';
    }).join('') + '</div>' : '';

  document.getElementById('ana-body').innerHTML = detsHtml + intsHtml + stratsHtml;
}

// ═══ TAB 5: HOOKS & CONTEXT ═══
function renderAnaHooks(a, id) {
  var hooks = a.hook_trace || [];
  var cx = a.context_analysis || {};

  document.getElementById('ana-header').innerHTML = '<h2 style="margin:0">🔗 Hook Trace & Context</h2><p style="color:var(--text-secondary)">' + hooks.length + ' hook types detected · ' + (cx.pressure_readings||0) + ' context readings</p>';

  var hooksHtml = '<div class="card" style="margin-bottom:12px"><h3>🔗 Hook Events by Type</h3>' +
    (hooks.length > 0 ? hooks.map(function(h) {
      return '<div style="margin:8px 0;padding:10px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid var(--accent-blue)">' +
        '<strong>' + h.hook + '</strong> <span style="color:var(--text-secondary);font-size:12px">— ' + (h.description||'') + '</span>' +
        '<span style="float:right;font-size:13px;font-weight:600;color:var(--accent-blue)">' + h.count + ' event(s)</span>' +
        '<div style="margin-top:8px;font-size:11px;color:var(--text-secondary);max-height:200px;overflow-y:auto">' +
        (h.events||[]).slice(0, 10).map(function(ev) {
          var detail = ev.tool||ev.detail||ev.text||ev.task||'';
          if (typeof detail === 'object') detail = JSON.stringify(detail);
          detail = String(detail).substring(0, 120);
          return '<div style="padding:2px 0;border-bottom:1px solid var(--border)">#' + (ev.seq||'?') + ' ' + escapeHtml(detail) + '</div>';
        }).join('') +
        ((h.events||[]).length > 10 ? '<div style="padding:2px 0;color:var(--text-secondary)">... and ' + ((h.events||[]).length - 10) + ' more</div>' : '') +
        '</div></div>';
    }).join('') : '<p style="color:var(--text-secondary)">No hooks traced yet</p>') +
    '</div>';

  var ctxHtml = '<div class="card"><h3>📐 Context Pressure Analysis</h3>' +
    '<div class="stats-grid" style="grid-template-columns:repeat(3,1fr);margin-bottom:12px">' +
    '<div class="stat-card"><div class="stat-value">' + (cx.pressure_readings||0) + '</div><div class="stat-label">Readings</div></div>' +
    '<div class="stat-card"><div class="stat-value" style="color:' + ((cx.avg_pressure_pct||0) > 75 ? 'var(--accent-red)' : 'var(--accent-yellow)') + '">' + (cx.avg_pressure_pct||0).toFixed(0) + '%</div><div class="stat-label">Average Pressure</div></div>' +
    '<div class="stat-card"><div class="stat-value" style="color:' + ((cx.max_pressure_pct||0) > 85 ? 'var(--accent-red)' : '') + '">' + (cx.max_pressure_pct||0).toFixed(0) + '%</div><div class="stat-label">Peak Pressure</div></div>' +
    '</div>' +
    '<div style="font-size:12px;color:var(--text-secondary)">Status: <strong style="color:' + (cx.status === 'critical' ? 'var(--accent-red)' : cx.status === 'warning' ? 'var(--accent-yellow)' : 'var(--accent-green)') + '">' + (cx.status||'healthy') + '</strong></div>' +
    (cx.pressure_history && cx.pressure_history.length > 0 ?
      '<div style="margin-top:8px"><strong style="font-size:12px">Pressure History:</strong>' +
      '<div style="display:flex;gap:4px;margin-top:4px;flex-wrap:wrap">' +
      cx.pressure_history.map(function(p) {
        var c = p > 85 ? 'var(--accent-red)' : p > 60 ? 'var(--accent-yellow)' : 'var(--accent-green)';
        return '<div style="padding:4px 10px;background:' + c + '22;border:1px solid ' + c + ';border-radius:4px;font-size:11px;font-family:monospace">' + p + '%</div>';
      }).join('') + '</div></div>' : '') +
    '</div>';

  document.getElementById('ana-body').innerHTML = hooksHtml + ctxHtml;
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// Build event timeline from raw session data
function buildEventTimeline(raw) {
  var events = [];
  // From pipeline observations
  var obs = (raw.pipeline && raw.pipeline.observations) ? raw.pipeline.observations : [];
  // From session events list
  var sev = raw.events || [];

  // Build rows from AgentEvents
  var rows = sev.map(function(e, i) {
    var icon = '●';
    var type = 'event';
    var detail = '';
    var costStr = '';
    if (typeof e === 'string') { detail = e; }
    else if (e && e.type) {
      type = e.type;
      if (type === 'started') { icon = '▶'; detail = e.task || ''; }
      else if (type === 'completed') { icon = '✓'; detail = e.summary || ''; }
      else if (type === 'failed') { icon = '✗'; detail = e.error || ''; }
      else if (type === 'tool_call_start') { icon = '→'; detail = (e.tool||'') + ': ' + JSON.stringify(e.args||{}).substring(0,80); }
      else if (type === 'tool_call_end') { icon = '←'; detail = (e.tool||'') + ' done'; costStr = e.result && e.result.token_count ? ' <span style="color:var(--accent-blue)">' + e.result.token_count + ' tok</span>' : ''; }
      else if (type === 'message_sent') { icon = '💬'; detail = (e.content && e.content.text) ? e.content.text.substring(0,80) : ''; }
      else if (type === 'context_pressure') { icon = '📐'; detail = 'Pressure: ' + ((e.current_ratio||0)*100).toFixed(0) + '%'; }
      else { detail = JSON.stringify(e).substring(0,100); }
    }
    return '<div style="padding:2px 0;border-bottom:1px solid var(--border)"><span style="color:var(--text-secondary)">' + icon + '</span> ' + '<span style="color:var(--accent-blue);font-size:10px">[' + type + ']</span> ' + detail + costStr + '</div>';
  });

  if (rows.length === 0 && obs.length > 0) {
    rows = obs.slice(0, 20).map(function(o) {
      return '<div style="padding:2px 0;border-bottom:1px solid var(--border);font-size:11px"><span style="color:var(--accent-green)">●</span> ' + (o.dimension||'obs') + ': ' + JSON.stringify(o).substring(0,80) + '</div>';
    });
  }

  return rows.length > 0 ? rows.join('') : '<p style="color:var(--text-secondary)">No events captured yet</p>';
}

function showLive(id) { showAnalysis(id); }

// ═══════════════════════════════════════════════════════════
// AUDIT — Per-session audit trail
// ═══════════════════════════════════════════════════════════

function renderAudit() {
  api('/v1/sessions').then(function(d) {
    var rows = (d.sessions||[]).map(function(s) {
      return '<tr><td><code>' + (s.id||'').substring(0,12) + '</code></td><td>' + (s.task||'').substring(0,40) + '</td><td>' + (s.agent_type||'') + '</td><td><span class="badge badge-' + (s.status||'running') + '">' + s.status + '</span></td><td><a href="javascript:showAuditDetail(\'' + s.id + '\')">View Trail</a></td></tr>';
    }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Audit Trail</h2><p style="color:var(--text-secondary);margin-bottom:8px">Complete event-by-event audit for every session. All events, detections, and interventions are recorded.</p>' +
      '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th></th></tr></thead><tbody>' +
      (rows || '<tr><td colspan="5">No sessions</td></tr>') + '</tbody></table></div>';
  });
}

function showAuditDetail(id) {
  api('/v1/sessions/' + id + '/analysis').then(function(a) {
    var sm = a.session_summary || {};
    var tk = a.token_analysis || {};
    var tl = a.tool_analysis || {};
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Audit: ' + id.substring(0,12) + '</h2>' +
      '<p style="color:var(--text-secondary)">' + (a.task||'') + ' · ' + a.status + ' · ' + formatDuration(a.duration_secs||0) + '</p>' +
      '<h3>Event Summary</h3>' +
      '<table><tr><td>Total Events</td><td><strong>' + (sm.total_events||0) + '</strong></td><td>User Prompts</td><td><strong>' + (sm.user_prompts||0) + '</strong></td></tr>' +
      '<tr><td>Tool Calls</td><td><strong>' + (tl.total_calls||0) + '</strong></td><td>Tool Errors</td><td><strong>' + (tl.total_errors||0) + '</strong></td></tr>' +
      '<tr><td>Subagents</td><td><strong>' + (sm.subagents_spawned||0) + '</strong></td><td>Interventions</td><td><strong>' + (sm.interventions||0) + '</strong></td></tr>' +
      '<tr><td>Detections</td><td><strong>' + (sm.detections||0) + '</strong></td><td>Observations</td><td><strong>' + (sm.observations||0) + '</strong></td></tr>' +
      '<tr><td>Tokens</td><td><strong>' + (tk.total_tokens||0).toLocaleString() + '</strong></td><td>Est. Cost</td><td><strong>$' + (tk.estimated_cost_usd||0).toFixed(5) + '</strong></td></tr></table>' +
      '<p style="margin-top:8px"><strong>Stop Reason:</strong> ' + (a.stop_analysis||'N/A') + '</p>' +
      '<p style="color:var(--text-secondary);font-size:12px;margin-top:8px">Audit trail captures every event with full traceability — all data is stored in-memory per session.</p>' +
      '<div class="flex-row" style="margin-top:12px"><button onclick="showAnalysis(\'' + id + '\')">Full Analysis</button><button onclick="showPage(\'sessions\')">← Sessions</button></div></div>';
  });
}

// ═══════════════════════════════════════════════════════════
// ANALYTICS — Aggregate across all sessions
// ═══════════════════════════════════════════════════════════

function renderAnalytics() {
  api('/v1/ingest/status').then(function(s) {
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Analytics</h2>' +
      '<div class="stats-grid">' +
      '<div class="stat-card"><div class="stat-value">' + (s.totalSessions||0) + '</div><div class="stat-label">Total Sessions</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">' + (s.activeSessions||0) + '</div><div class="stat-label">Active</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-yellow)">' + (s.totalObservations||0) + '</div><div class="stat-label">Observations</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-red)">' + (s.totalDetections||0) + '</div><div class="stat-label">Detections</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-blue)">' + (s.totalInterventions||0) + '</div><div class="stat-label">Interventions</div></div>' +
      '<div class="stat-card"><div class="stat-value">' + (s.totalEventsInRing||0) + '</div><div class="stat-label">Events Ingested</div></div>' +
      '</div>' +
      '<p style="margin-top:12px;color:var(--text-secondary)">' + (s.message||'') + '</p>' +
      '<p style="color:var(--text-secondary);font-size:13px">12 observers · 16 detectors · 14 strategies running on every event</p>' +
      '</div>';
  });
}

// ═══════════════════════════════════════════════════════════
// META — Self-improving harness
// ═══════════════════════════════════════════════════════════

function renderMeta() {
  api('/v1/meta/weaknesses').then(function(w) {
    var weaknesses = (w.weaknesses||[]).map(function(wk) {
      return '<div class="card"><h3>' + wk.pattern + '</h3><p>Frequency: ' + wk.count + ' | Agent: ' + (wk.agent_type||'all') + '</p><p style="color:var(--text-secondary);font-size:12px">' + (wk.description||'') + '</p></div>';
    }).join('');

    api('/v1/meta/edits').then(function(e) {
      var edits = (e.edits||[]).map(function(ed) {
        return '<tr><td>' + (ed.id||'').substring(0,8) + '</td><td>' + (ed.rule||'') + '</td><td>' + (ed.change||'') + '</td><td><button class="success" onclick="acceptEdit(\'' + ed.id + '\')">Accept</button> <button class="danger" onclick="rejectEdit(\'' + ed.id + '\')">Reject</button></td></tr>';
      }).join('');

      document.getElementById('content').innerHTML =
        '<div class="card" style="border-left:4px solid var(--accent-purple)"><h2>🧠 META — Self-Improving Harness</h2>' +
        '<p style="color:var(--text-secondary);margin-bottom:12px">Mines weakness patterns across ALL sessions and proposes harness rule improvements. No minimum session limit — runs with whatever data is available.</p>' +
        '<div class="flex-row mb"><button onclick="runImprove()">Run Improvement Cycle</button><button onclick="showPage(\'meta\')">🔄 Refresh</button></div></div>' +

        '<div class="card"><h3>Weakness Patterns</h3>' +
        (weaknesses || '<p style="color:var(--text-secondary)">No patterns mined yet. Run an improvement cycle to analyze sessions.</p>') + '</div>' +

        '<div class="card"><h3>Proposed Rule Changes</h3>' +
        '<table><thead><tr><th>ID</th><th>Rule</th><th>Change</th><th></th></tr></thead><tbody>' +
        (edits || '<tr><td colspan="4">No pending proposals — run improvement cycle</td></tr>') + '</tbody></table></div>';
    });
  });
}

function generateReport(id) {
  api('/v1/sessions/' + id + '/analysis').then(function(a) {
    var tk = a.token_analysis || {};
    var report = '# Forge Harness Report\n\n';
    report += '**Session:** ' + (a.task||'Untitled') + '\n';
    report += '**Agent:** ' + a.agent_type + ' | **Status:** ' + a.status + ' | **Duration:** ' + formatDuration(a.duration_secs||0) + '\n';
    report += '**Model:** ' + (a.model||tk.model_family||'?') + (tk.is_estimated ? ' (auto-detected)' : '') + '\n\n';
    report += '## Token Analysis\n- Total: ' + (tk.total_tokens||0).toLocaleString() + ' (input: ' + (tk.input_tokens||0).toLocaleString() + ', output: ' + (tk.output_tokens||0).toLocaleString() + ')\n';
    report += '- Est. Cost: $' + (tk.estimated_cost_usd||0).toFixed(5) + '\n';
    report += '- Data Source: ' + (tk.data_source||'unknown') + '\n';
    report += '- Cache: ' + (tk.cache_hit_pct||0).toFixed(0) + '% hit rate\n\n';
    report += '## Prompt History\n';
    if (a.prompt_history && a.prompt_history.length > 0) {
      a.prompt_history.forEach(function(p) { report += '### Prompt #' + p.index + ' (~' + (p.estimated_tokens||0) + ' tokens)\n' + p.text + '\n\n'; });
    } else { report += 'No prompts captured\n\n'; }
    report += '## Detections\n';
    (a.detection_details||[]).forEach(function(d) { report += '- **' + d.category + '** [' + d.severity + '] ' + (d.confidence*100).toFixed(0) + '%: ' + d.description + '\n'; });
    if (!a.detection_details?.length) report += 'No issues detected\n';
    report += '\n## Interventions\n';
    (a.intervention_details||[]).forEach(function(i) { report += '- ' + i.strategy + ': ' + i.action + '\n'; });
    if (!a.intervention_details?.length) report += 'No interventions\n';
    report += '\n## Recommendations\n';
    (a.recommendations||[]).forEach(function(r) { report += '- ' + r + '\n'; });
    report += '\n---\n*Generated by Forge v0.3.0*';

    var blob = new Blob([report], {type:'text/markdown'});
    var url = URL.createObjectURL(blob);
    var aEl = document.createElement('a');
    aEl.href = url; aEl.download = 'forge-report-' + id.substring(0,8) + '.md';
    document.body.appendChild(aEl); aEl.click(); document.body.removeChild(aEl);
    URL.revokeObjectURL(url);
  });
}

function runImprove() {
  api('/v1/meta/improve', {method:'POST',headers:{'Content-Type':'application/json'},body:'{}'}).then(function(r) {
    alert('Improvement cycle started: ' + (r.message||''));
    showPage('meta');
  });
}
function acceptEdit(id) { api('/v1/meta/edits/' + id + '/accept', {method:'POST',headers:{'Content-Type':'application/json'},body:'{}'}).then(function() { showPage('meta'); }); }
function rejectEdit(id) { api('/v1/meta/edits/' + id + '/reject', {method:'POST',headers:{'Content-Type':'application/json'},body:'{}'}).then(function() { showPage('meta'); }); }

// ═══════════════════════════════════════════════════════════
// SETTINGS
// ═══════════════════════════════════════════════════════════

var ALL_OBSERVERS = [
  {id:'token', name:'Token Watcher', desc:'Token efficiency, cache hit rate, waste detection'},
  {id:'latency', name:'Latency Watcher', desc:'Response time, p50/p95, slow tool detection'},
  {id:'cost', name:'Cost Watcher', desc:'Per-turn cost tracking, budget monitoring'},
  {id:'accuracy', name:'Accuracy Watcher', desc:'Output quality, lint errors, test pass rate'},
  {id:'security', name:'Security Watcher', desc:'Secret leaks, prompt injection, unsafe patterns'},
  {id:'reliability', name:'Reliability Watcher', desc:'Error rates, success ratios, retry patterns'},
  {id:'context_quality', name:'Context Quality', desc:'Context pressure, redundancy, compaction need'},
  {id:'orch', name:'Orchestration Watcher', desc:'Multi-agent coordination, fork/join health'},
  {id:'comm', name:'Communication Watcher', desc:'Agent-to-agent message efficiency'},
  {id:'compliance', name:'Compliance Watcher', desc:'Rules alignment, policy violations'},
  {id:'memory', name:'Memory Watcher', desc:'Memory usage, retention, context window pressure'},
  {id:'diversity', name:'Diversity Watcher', desc:'Output variety, mode collapse detection'}
];
var ALL_DETECTORS = [
  {id:'loop', name:'Loop Detector', desc:'Repeated tool calls with no progress'},
  {id:'stale_context', name:'Stale Context', desc:'Re-reading same files, context bloat'},
  {id:'cost_anomaly', name:'Cost Anomaly', desc:'Unexpected cost spikes vs baseline'},
  {id:'deadlock', name:'Deadlock', desc:'Agents waiting on each other indefinitely'},
  {id:'hallucination', name:'Hallucination', desc:'Referencing non-existent symbols/files'},
  {id:'prompt_injection', name:'Prompt Injection', desc:'Injection patterns in user input'},
  {id:'secret_leak', name:'Secret Leak', desc:'API keys, tokens, passwords in output'},
  {id:'variety_collapse', name:'Variety Collapse', desc:'Output becoming too similar'},
  {id:'conversation_stall', name:'Conversation Stall', desc:'Agent stops making progress'},
  {id:'goal_drift', name:'Goal Drift', desc:'Task divergence from original intent'},
  {id:'model_mismatch', name:'Model Mismatch', desc:'Task too complex for current model'},
  {id:'accuracy_risk', name:'Accuracy Risk', desc:'Generated code without tests'},
  {id:'runaway_cost', name:'Runaway Cost', desc:'Cost accelerating beyond threshold'},
  {id:'resource_exhaustion', name:'Resource Exhaustion', desc:'CPU, memory, disk limits'},
  {id:'output_degradation', name:'Output Degradation', desc:'Quality declining over time'},
  {id:'compliance_gap', name:'Compliance Gap', desc:'Missing required compliance steps'}
];
var ALL_STRATEGIES = [
  {id:'nudge', name:'Nudge', desc:'Inject hint into agent context'},
  {id:'compact', name:'Compact', desc:'Trigger context compaction'},
  {id:'pause', name:'Pause', desc:'Pause agent, wait for human review'},
  {id:'escalate', name:'Escalate', desc:'Upgrade model or expand budget'},
  {id:'fork', name:'Fork', desc:'Split into parallel sub-agents'},
  {id:'reroute', name:'Reroute', desc:'Redirect to different tool/path'},
  {id:'rollback', name:'Rollback', desc:'Restore from last checkpoint'},
  {id:'diversify', name:'Diversify', desc:'Increase output variety'},
  {id:'isolate', name:'Isolate', desc:'Quarantine suspicious agent'},
  {id:'circuit_break', name:'Circuit Break', desc:'Stop all operations immediately'},
  {id:'replace', name:'Replace', desc:'Swap model or tool mid-session'},
  {id:'interject', name:'Interject', desc:'Insert message as user'},
  {id:'degrade', name:'Degrade', desc:'Gracefully reduce capability'},
  {id:'quarantine', name:'Quarantine', desc:'Isolate and audit agent output'}
];

function renderSettings() {
  api('/v1/harness').then(function(h) {
    var obsChecks = ALL_OBSERVERS.map(function(o) {
      var checked = (h.observers||[]).indexOf(o.id) >= 0 || !h.observers ? 'checked' : '';
      return '<label class="toggle" style="display:flex;align-items:flex-start;gap:8px;padding:6px 0;border-bottom:1px solid var(--border)"><input type="checkbox" ' + checked + ' onchange="toggleSetting(\'observer\',\'' + o.id + '\',this.checked)" style="margin-top:2px"><div><strong style="font-size:13px">' + o.name + '</strong><p style="font-size:11px;color:var(--text-secondary);margin:2px 0">' + o.desc + '</p></div></label>';
    }).join('');
    var detChecks = ALL_DETECTORS.map(function(d) {
      var checked = (h.detectors||[]).indexOf(d.id) >= 0 || !h.detectors ? 'checked' : '';
      return '<label class="toggle" style="display:flex;align-items:flex-start;gap:8px;padding:6px 0;border-bottom:1px solid var(--border)"><input type="checkbox" ' + checked + ' onchange="toggleSetting(\'detector\',\'' + d.id + '\',this.checked)" style="margin-top:2px"><div><strong style="font-size:13px">' + d.name + '</strong><p style="font-size:11px;color:var(--text-secondary);margin:2px 0">' + d.desc + '</p></div></label>';
    }).join('');
    var strChecks = ALL_STRATEGIES.map(function(s) {
      var checked = (h.strategies||[]).indexOf(s.id) >= 0 || !h.strategies ? 'checked' : '';
      return '<label class="toggle" style="display:flex;align-items:flex-start;gap:8px;padding:6px 0;border-bottom:1px solid var(--border)"><input type="checkbox" ' + checked + ' onchange="toggleSetting(\'strategy\',\'' + s.id + '\',this.checked)" style="margin-top:2px"><div><strong style="font-size:13px">' + s.name + '</strong><p style="font-size:11px;color:var(--text-secondary);margin:2px 0">' + s.desc + '</p></div></label>';
    }).join('');

    document.getElementById('content').innerHTML =
      '<div class="card"><h2>⚙ Harness Settings</h2>' +
      '<div class="flex-row mb"><label class="toggle"><input type="checkbox" ' + (h.dry_run ? '' : 'checked') + ' onchange="toggleSetting(\'intervention\',\'enabled\',this.checked)"> <strong>Auto-Intervention</strong></label></div>' +
      '<p style="font-size:13px;color:var(--text-secondary);margin-bottom:12px">Preset: <strong>' + (h.preset||'claude-code') + '</strong> | v0.3.0</p></div>' +

      '<div class="card"><h3>🔍 Observers (' + ALL_OBSERVERS.length + ')</h3>' +
      '<div style="display:flex;gap:8px;margin-bottom:12px"><button onclick="checkAll(\'observer\',true)" style="font-size:11px;padding:4px 8px">Select All</button><button onclick="checkAll(\'observer\',false)" style="font-size:11px;padding:4px 8px">Deselect All</button></div>' +
      obsChecks + '</div>' +

      '<div class="card"><h3>⚠ Detectors (' + ALL_DETECTORS.length + ')</h3>' +
      '<div style="display:flex;gap:8px;margin-bottom:12px"><button onclick="checkAll(\'detector\',true)" style="font-size:11px;padding:4px 8px">Select All</button><button onclick="checkAll(\'detector\',false)" style="font-size:11px;padding:4px 8px">Deselect All</button></div>' +
      detChecks + '</div>' +

      '<div class="card"><h3>🔧 Strategies (' + ALL_STRATEGIES.length + ')</h3>' +
      '<div style="display:flex;gap:8px;margin-bottom:12px"><button onclick="checkAll(\'strategy\',true)" style="font-size:11px;padding:4px 8px">Select All</button><button onclick="checkAll(\'strategy\',false)" style="font-size:11px;padding:4px 8px">Deselect All</button></div>' +
      strChecks + '</div>' +

      '<div class="card"><h3>Server Info</h3>' +
      '<p style="font-size:13px;color:var(--text-secondary)">Ingest: <code>' + window.location.origin + '/api/v1/ingest/event</code></p>' +
      '<p style="font-size:13px;color:var(--text-secondary)">API: <a href="/api/v1/health" target="_blank">/api/v1/health</a></p></div>';
  });
}

function toggleSetting(type, id, enabled) {
  api('/v1/harness', {method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({type:type, id:id, enabled:enabled})})
    .then(function(r) { console.log('Settings updated:', r); });
}
function checkAll(type, enable) {
  var items = type === 'observer' ? ALL_OBSERVERS : type === 'detector' ? ALL_DETECTORS : ALL_STRATEGIES;
  items.forEach(function(item) {
    var cb = document.querySelector('input[onchange*=\"' + type + '\"][onchange*=\"' + item.id + '\"]');
    if (cb) { cb.checked = enable; toggleSetting(type, item.id, enable); }
  });
}

// ═══════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════

function formatDuration(secs) {
  if (secs < 60) return secs + 's';
  if (secs < 3600) return Math.floor(secs/60) + 'm ' + (secs%60) + 's';
  return Math.floor(secs/3600) + 'h ' + Math.floor((secs%3600)/60) + 'm';
}
