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
      return '<tr><td><code>' + (s.id||'').substring(0,12) + '</code></td><td>' + (s.task||'').substring(0,30) + '</td><td>' + (s.agent_type||'') + '</td><td><span class="badge badge-' + (s.status||'running') + '">' + s.status + '</span></td><td>' + healthHtml + '</td><td>' + (pipe.observation_count||0) + '/' + (pipe.detection_count||0) + '/' + (pipe.intervention_count||0) + '</td><td><a href="javascript:showAnalysis(\'' + s.id + '\')">Analysis</a></td></tr>';
    }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Sessions</h2>' +
      '<p style="color:var(--text-secondary);margin-bottom:8px">O/D/I = Observations / Detections / Interventions</p>' +
      '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th>Health</th><th>O/D/I</th><th></th></tr></thead><tbody>' +
      (rows || '<tr><td colspan="7">No sessions yet — sessions appear automatically when you use Claude Code or other agents</td></tr>') +
      '</tbody></table></div>';
  });
}

// ═══════════════════════════════════════════════════════════
// ANALYSIS — Full per-session harness report
// ═══════════════════════════════════════════════════════════

function showAnalysis(id) {
  currentPage = 'sessions';
  document.getElementById('content').innerHTML = '<div style="max-width:1200px;margin:0 auto"><div id="ana-header"><p>Loading harness analysis...</p></div><div id="ana-body"></div></div>';

  // Fetch both analysis summary and raw session data for event breakdown
  Promise.all([api('/v1/sessions/' + id + '/analysis'), api('/v1/sessions/' + id)]).then(function(results) {
    var a = results[0];
    var raw = results[1];
    if (a.error) { document.getElementById('content').innerHTML = '<div class="card"><h2>Error</h2><p>' + a.error + '</p></div>'; return; }
    renderFullAnalysis(a, raw, id);
  }).catch(function(e) {
    document.getElementById('content').innerHTML = '<div class="card"><h2>Error</h2><p>' + e.message + '</p></div>';
  });
}

function renderFullAnalysis(a, raw, id) {
  var hs = a.health_analysis || {};
  var tk = a.token_analysis || {};
  var tl = a.tool_analysis || {};
  var cx = a.context_analysis || {};
  var sm = a.session_summary || {};
  var recs = a.recommendations || [];
  var healthPct = Math.round((hs.overall||0) * 100);
  var healthColor = healthPct > 80 ? 'var(--accent-green)' : healthPct > 50 ? 'var(--accent-yellow)' : 'var(--accent-red)';
  var cost = tk.estimated_cost_usd || 0;

  // ── Detection and intervention detail data ──
  var detDetails = a.detection_details || [];
  var intDetails = a.intervention_details || [];

  // ── Event log ──
  var eventLog = a.event_log || [];
  var timelineHtml = eventLog.length > 0 ? eventLog.map(function(ev) {
    var time = (ev.time||'').substring(11,19) || '';
    var color = ev.type === 'error' ? 'var(--accent-red)' : ev.type === 'tool_end' && ev.status === 'FAILED' ? 'var(--accent-red)' : ev.type === 'context' ? 'var(--accent-yellow)' : ev.type === 'start' ? 'var(--accent-green)' : 'var(--text-secondary)';
    return '<div style="padding:3px 0;border-bottom:1px solid var(--border);display:flex;gap:8px;font-size:11px;font-family:monospace">' +
      '<span style="color:var(--text-secondary);min-width:50px">' + time + '</span>' +
      '<span>' + ev.icon + '</span>' +
      '<span style="color:' + color + '">' + (ev.detail||'').substring(0,120) + '</span>' +
      (ev.tokens > 0 ? '<span style="color:var(--accent-blue);margin-left:auto">' + ev.tokens + ' tok</span>' : '') +
      '</div>';
  }).join('') : '<p style="color:var(--text-secondary);font-size:12px">No events captured yet</p>';

  // ── Tool usage ──
  var toolRows = (tl.breakdown||[]).map(function(t) {
    return '<tr><td><strong>' + t.tool + '</strong></td><td>' + t.calls + '</td><td>' + (t.errors > 0 ? '<span style="color:var(--accent-red)">' + t.errors + '</span>' : '0') + '</td><td>' + (t.pct_of_total||0).toFixed(0) + '%</td></tr>';
  }).join('');

  // ── Meaningful observations only (skip 100% defaults) ──
  var obsDetails = a.observation_details || [];
  var meaningfulObs = obsDetails.filter(function(g) {
    var d = g.description || '';
    // Skip pure default observations
    if (d.indexOf('0 operations, 0.0%') >= 0) return false;
    if (d.indexOf('0 unique files tracked, 0%') >= 0) return false;
    if (d.indexOf('0 calls') >= 0) return false;
    if (d.indexOf('0 lint errors, 100%') >= 0) return false;
    if (d.indexOf('0 policy violations') >= 0) return false;
    if (d.indexOf('0 agents, 0 forks') >= 0) return false;
    if (d.indexOf('0 agent messages') >= 0) return false;
    return true;
  });
  var dimNames = {token:'Token',latency:'Latency',cost:'Cost',accuracy:'Accuracy',security:'Security',reliability:'Reliability',context_quality:'Context',orch:'Orchestration',comm:'Communication',compliance:'Compliance',memory:'Memory',diversity:'Diversity'};
  var obsHtml = meaningfulObs.length > 0 ? meaningfulObs.map(function(g) {
    return '<div style="padding:6px 8px;border-bottom:1px solid var(--border)"><span style="color:var(--accent-green)">●</span> <strong>' + (dimNames[g.dimension]||g.dimension) + '</strong>: ' + (g.description||'') + '</div>';
  }).join('') : '<p style="color:var(--text-secondary);font-size:12px">Waiting for more events to build observation data...</p>';

  var recsHtml = recs.length > 0 ? recs.map(function(r) { return '<div style="padding:6px 0;border-bottom:1px solid var(--border)">💡 ' + r + '</div>'; }).join('') : '<p style="color:var(--text-secondary)">No recommendations yet — run more agent events</p>';

  document.getElementById('ana-header').innerHTML =
    '<div style="display:flex;justify-content:space-between;align-items:flex-start;flex-wrap:wrap;gap:16px;margin-bottom:16px">' +
    '<div><h2 style="margin:0">' + (a.task||'Untitled').substring(0,60) + '</h2>' +
    '<p style="margin:4px 0;color:var(--text-secondary)">' + a.agent_type + ' · ' + a.status + ' · ' + formatDuration(a.duration_secs||0) + ' · session: <code>' + id.substring(0,12) + '</code></p>' +
    '<p style="margin:4px 0;font-size:13px">' + (a.stop_analysis||'') + '</p></div>' +
    '<div style="text-align:right;min-width:120px"><div style="font-size:42px;font-weight:700;color:' + healthColor + '">' + healthPct + '%</div><div style="font-size:12px;color:var(--text-secondary)">Health</div></div></div>';

  document.getElementById('ana-body').innerHTML =
    // ── WHAT HAPPENED: Event Timeline ──
    '<div class="card"><h3>📋 What Happened</h3>' +
    '<div style="max-height:400px;overflow-y:auto">' + timelineHtml + '</div></div>' +

    // ── COST & TOKENS ──
    '<div class="card"><h3>💰 Cost & Tokens</h3>' +
    '<div class="stats-grid" style="grid-template-columns:repeat(4,1fr)">' +
    '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green);font-size:22px">$' + cost.toFixed(4) + '</div><div class="stat-label">Est. Cost</div></div>' +
    '<div class="stat-card"><div class="stat-value">' + (tk.total_tokens||0).toLocaleString() + '</div><div class="stat-label">Total Tokens</div></div>' +
    '<div class="stat-card"><div class="stat-value">' + (tk.cache_hit_pct||0).toFixed(0) + '%</div><div class="stat-label">Cache Hit</div></div>' +
    '<div class="stat-card"><div class="stat-value">' + (tk.model_family||'?') + '</div><div class="stat-label">Model</div></div>' +
    '</div>' +
    '<p style="font-size:12px;color:var(--text-secondary);margin-top:4px">Input: ' + (tk.input_tokens||0).toLocaleString() + ' · Output: ' + (tk.output_tokens||0).toLocaleString() + ' · Cache read: ' + (tk.cache_read_tokens||0).toLocaleString() + ' · Gross: $' + (tk.gross_cost_usd||0).toFixed(4) + ' · Saved: $' + (tk.cache_savings_usd||0).toFixed(4) + '</p></div>' +

    // ── TOOLS USED ──
    '<div class="card"><h3>🛠️ Tools Used</h3>' +
    (toolRows ? '<table><thead><tr><th>Tool</th><th>Calls</th><th>Errors</th><th>%</th></tr></thead><tbody>' + toolRows + '</tbody></table>' : '<p style="color:var(--text-secondary)">No tools called</p>') + '</div>' +

    // ── OBSERVATIONS (meaningful only) ──
    '<div class="card"><h3>🔍 What Was Observed</h3>' + obsHtml + '</div>' +

    // ── ISSUES FOUND ──
    '<div class="card"><h3>⚠ Issues Found</h3>' +
    (detDetails.length > 0 ? detDetails.map(function(d) {
      var sevColor = (d.severity||'').indexOf('Critical') >= 0 ? 'var(--accent-red)' : (d.severity||'').indexOf('Warning') >= 0 ? 'var(--accent-yellow)' : 'var(--accent-blue)';
      return '<div style="margin:8px 0;padding:10px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid ' + sevColor + '">' +
        '<div style="display:flex;justify-content:space-between;align-items:center">' +
        '<strong>' + (d.category||'issue') + '</strong>' +
        '<span style="font-size:11px;padding:2px 8px;border-radius:10px;background:' + sevColor + ';color:#000">' + (d.severity||'?') + ' · ' + ((d.confidence||0)*100).toFixed(0) + '%</span>' +
        '</div><p style="margin:6px 0 0 0;font-size:12px;color:var(--text-secondary)">' + (d.description||'') + '</p></div>';
    }).join('') : '<p style="color:var(--accent-green)">✅ No issues detected</p>') +
    '<p style="font-size:12px;color:var(--text-secondary);margin-top:8px">Context: avg ' + (cx.avg_pressure_pct||0).toFixed(0) + '% / max ' + (cx.max_pressure_pct||0).toFixed(0) + '% · ' + (cx.compaction_events||0) + ' compactions</p></div>' +

    // ── INTERVENTIONS ──
    '<div class="card"><h3>🔧 Interventions</h3>' +
    (intDetails.length > 0 ? intDetails.map(function(i) {
      return '<div style="margin:6px 0;padding:8px;background:var(--bg-secondary);border-radius:6px;border-left:4px solid var(--accent-blue)">' +
        '<strong style="font-size:13px">' + (i.strategy||'intervention') + '</strong>' +
        '<p style="margin:4px 0 0 0;font-size:12px;color:var(--text-secondary)">' + (i.action||'') + '</p></div>';
    }).join('') : '<p style="color:var(--text-secondary)">No interventions needed</p>') + '</div>' +

    // ── RECOMMENDATIONS ──
    '<div class="card" style="border-left:4px solid var(--accent-purple)"><h3>🧠 What To Improve</h3>' + recsHtml + '</div>' +

    '<div class="flex-row" style="margin-top:16px;gap:8px">' +
    '<button onclick="showAnalysis(\'' + id + '\')">🔄 Refresh</button>' +
    '<button onclick="showPage(\'sessions\')">← Back</button></div>';
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
        '<p style="color:var(--text-secondary);margin-bottom:12px">Mines weakness patterns across sessions and proposes harness rule improvements. Based on the Self-Harness paper (Shanghai AI Lab, 2026).</p>' +
        '<div class="flex-row mb"><button onclick="runImprove()">Run Improvement Cycle</button><button onclick="showPage(\'meta\')">🔄 Refresh</button></div></div>' +

        '<div class="card"><h3>Weakness Patterns</h3>' +
        (weaknesses || '<p style="color:var(--text-secondary)">No patterns mined yet. Requires 20+ completed sessions.</p>') + '</div>' +

        '<div class="card"><h3>Proposed Rule Changes</h3>' +
        '<table><thead><tr><th>ID</th><th>Rule</th><th>Change</th><th></th></tr></thead><tbody>' +
        (edits || '<tr><td colspan="4">No pending proposals</td></tr>') + '</tbody></table></div>';
    });
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
