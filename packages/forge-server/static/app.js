// Forge Dashboard — Simple, working version
var API_BASE = '/api';

function api(path, opts) {
  opts = opts || {};
  return fetch(API_BASE + path, opts).then(function(r) { return r.json(); });
}

// Simple page switcher
var currentPage = 'run';

function showPage(name) {
  currentPage = name;
  document.querySelectorAll('#nav-tabs a').forEach(function(a) {
    a.classList.toggle('active', a.getAttribute('data-page') === name);
  });
  var el = document.getElementById('content');
  el.innerHTML = '<div class="card"><p>Loading...</p></div>';
  try {
    if (name === 'run') renderRun();
    else if (name === 'sessions') renderSessions();
    else if (name === 'live') renderLive();
    else if (name === 'audit') renderAudit();
    else if (name === 'compliance') renderCompliance();
    else if (name === 'skills') renderSkills();
    else if (name === 'mcp') renderMCP();
    else if (name === 'export') renderExport();
    else if (name === 'marketplace') renderMarketplace();
    else if (name === 'cloud') renderCloud();
    else if (name === 'analytics') renderAnalytics();
    else if (name === 'meta') renderMeta();
    else if (name === 'auth') renderAuth();
    else if (name === 'admin') renderAdmin();
    else if (name === 'settings') renderSettings();
  } catch(e) { el.innerHTML = '<div class="card"><p>Error: ' + e.message + '</p></div>'; }
}

// On page load, show Run page
document.addEventListener('DOMContentLoaded', function() {
  showPage('run');
});

// ── PAGE RENDERERS ──

function renderRun() {
  document.getElementById('content').innerHTML =
    '<div class="card" style="border-left:3px solid var(--accent-green)"><h2>Live Monitoring</h2>' +
    '<div id="ingest-status"><p>Checking for agent activity...</p></div></div>' +
    '<div class="card"><h2>Quick Stats</h2><div id="quick-stats"><p>Loading stats...</p></div></div>';

  checkIngestStatus();
  refreshStats();
}

function checkIngestStatus() {
  api('/v1/ingest/status').then(function(s) {
    var el = document.getElementById('ingest-status');
    if (!el) return;
    var running = s.activeSessions || 0;
    var total = s.totalSessions || 0;
    var events = s.totalEventsInRing || 0;
    var msg = s.message || 'Waiting for agent activity...';
    var color = running > 0 ? 'var(--accent-green)' : 'var(--accent-yellow)';
    var dot = running > 0 ? '●' : '○';
    el.innerHTML =
      '<div style="display:flex;align-items:center;gap:12px">' +
      '<span style="font-size:24px;color:' + color + '">' + dot + '</span>' +
      '<div><strong style="font-size:16px">' + msg + '</strong>' +
      '<p style="margin:4px 0;color:var(--text-secondary)">' +
      running + ' active, ' + total + ' total sessions | ' + events + ' events captured' +
      '</p><p style="margin:4px 0;color:var(--text-secondary);font-size:13px">' +
      'Use Claude Code, LangGraph, CrewAI, or any agent — Forge auto-detects and monitors.' +
      '</p></div></div>';
    if (running > 0) {
      el.innerHTML += '<p style="margin-top:8px"><a href="javascript:showPage(\'sessions\')">View sessions →</a></p>';
    }
  }).catch(function() {
    var el = document.getElementById('ingest-status');
    if (el) el.innerHTML = '<p style="color:var(--text-secondary)">Forge server starting up...</p>';
  });
  setTimeout(function() { if (currentPage === 'run') checkIngestStatus(); }, 5000);
}

function doRun() {
  var task = document.getElementById('task').value || 'Default task';
  var agent = document.getElementById('agent-type').value;
  var preset = document.getElementById('preset').value;
  document.getElementById('run-result').innerHTML = '<p>Starting...</p>';
  api('/v1/sessions', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({task:task, agent_type:agent, preset:preset}) })
    .then(function(r) {
      document.getElementById('run-result').innerHTML = '<div class="card" style="border-color:var(--accent-green)"><h3>Session Created</h3><p>ID: ' + r.id + '</p><p>Status: <span class="badge badge-running">' + r.status + '</span></p><p><a href="javascript:showLive(\'' + r.id + '\')">View Live</a></p></div>';
      refreshStats();
    })
    .catch(function(e) {
      document.getElementById('run-result').innerHTML = '<p style="color:var(--accent-red)">Error: ' + e.message + '</p>';
    });
}

function doDryRun() {
  var task = document.getElementById('task').value || 'Default task';
  document.getElementById('run-result').innerHTML = '<p>Dry run started...</p>';
  api('/v1/sessions', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({task:task + ' [dry-run]', agent_type:'solo', preset:'solo'}) })
    .then(function(r) { document.getElementById('run-result').innerHTML = '<p>Dry run session: ' + r.id + '</p>'; });
}

function refreshStats() {
  api('/v1/analytics/overview').then(function(o) {
    document.getElementById('quick-stats').innerHTML =
      '<div class="stats-grid">' +
      '<div class="stat-card"><div class="stat-value">' + (o.total_sessions||0) + '</div><div class="stat-label">Sessions</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">' + (o.completed||0) + '</div><div class="stat-label">Completed</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-yellow)">' + (o.running||0) + '</div><div class="stat-label">Running</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">-</div><div class="stat-label">Health</div></div>' +
      '</div>';
  }).catch(function(){});
}

function showLive(id) {
  currentPage = 'live';
  document.getElementById('content').innerHTML =
    '<div class="live-layout">' +
    '<div class="panel"><h2>Session: ' + id.substring(0,12) + '</h2>' +
    '<div class="flex-row mb"><button onclick="doPause(\'' + id + '\')">Pause</button><button onclick="doResume(\'' + id + '\')">Resume</button></div>' +
    '<div class="conversation" id="stream" style="max-height:400px;overflow-y:auto;font-family:monospace;font-size:13px"><p>Connecting...</p></div></div>' +
    '<div class="panel"><h3>Health</h3><div id="gauges">' +
    ['Token','Latency','Cost','Accuracy','Security','Reliability','Context','Orch','Compliance'].map(function(d) { return '<div class="gauge"><div class="gauge-label">' + d + '</div><div class="gauge-bar"><div class="gauge-fill gauge-green" style="width:100%"></div></div></div>'; }).join('') +
    '</div></div></div>';
  // Connect WebSocket
  var proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  var ws = new WebSocket(proto + '//' + location.host + '/ws');
  ws.onopen = function() { ws.send(JSON.stringify({action:'subscribe', session_id:id})); };
  ws.onmessage = function(evt) {
    try {
      var e = JSON.parse(evt.data);
      var s = document.getElementById('stream');
      var t = e.type || 'unknown';
      var txt = '';
      if (t === 'started') txt = '>> ' + (e.task || '');
      else if (t === 'thinking_start') txt = 'Thinking...';
      else if (t === 'thinking_delta') txt = e.text || '';
      else if (t === 'tool_call_start') txt = '> ' + (e.tool||'') + ': ' + JSON.stringify(e.args||{});
      else if (t === 'tool_call_end') txt = '< ' + (e.tool||'') + ' done';
      else if (t === 'completed') txt = 'OK Completed: ' + (e.summary||'');
      else if (t === 'failed') txt = 'XX Failed: ' + (e.error||'');
      else txt = JSON.stringify(e);
      var div = document.createElement('div');
      div.style.padding = '2px 0';
      div.style.borderBottom = '1px solid var(--border)';
      div.textContent = txt;
      s.appendChild(div);
      s.scrollTop = s.scrollHeight;
    } catch(ex) {}
  };
}

function doPause(id) { api('/v1/sessions/' + id + '/pause', {method:'POST',headers:{'Content-Type':'application/json'},body:'{}'}); }
function doResume(id) { api('/v1/sessions/' + id + '/resume', {method:'POST',headers:{'Content-Type':'application/json'},body:'{}'}); }

function renderSessions() {
  api('/v1/sessions').then(function(d) {
    var rows = (d.sessions||[]).map(function(s) {
      var isAuto = (s.task === '(live agent session)') ? ' ⚡auto' : '';
      var sourceStyle = isAuto ? 'color:var(--accent-green)' : 'color:var(--text-secondary)';
      return '<tr><td>' + (s.id||'').substring(0,12) + '</td><td>' + (s.task||'').substring(0,40) + '</td><td>' + (s.agent_type||'') + '</td><td><span class="badge badge-' + (s.status||'running') + '">' + s.status + '</span></td><td style="' + sourceStyle + ';font-size:12px">' + (isAuto || 'manual') + '</td><td><a href="javascript:showLive(\'' + s.id + '\')">Live</a></td></tr>';
    }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Sessions</h2>' +
      '<p style="color:var(--text-secondary);margin-bottom:8px">⚡ = auto-detected from real agent activity | manual = created via dashboard</p>' +
      '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th>Source</th><th></th></tr></thead><tbody>' + (rows || '<tr><td colspan="6">No sessions yet. Sessions appear automatically when you use Claude Code or other agents.</td></tr>') + '</tbody></table></div>';
  }).catch(function(e) {
    document.getElementById('content').innerHTML = '<div class="card"><h2>Sessions</h2><p>Error: ' + e.message + '</p></div>';
  });
}

function renderLive() {
  // Auto-connect to most recent running session if no specific ID
  api('/v1/sessions').then(function(d) {
    var running = (d.sessions || []).filter(function(s) { return s.status === 'running'; });
    if (running.length > 0) {
      showLive(running[0].id);
      return;
    }
    // Show all sessions with Live links
    var rows = (d.sessions || []).slice(0, 10).map(function(s) {
      return '<tr><td>' + (s.id||'').substring(0,12) + '</td><td>' + (s.task||'').substring(0,50) + '</td><td>' + (s.agent_type||'') + '</td><td><span class="badge badge-' + (s.status||'pending') + '">' + s.status + '</span></td><td><a href="javascript:showLive(\'' + s.id + '\')">View</a></td></tr>';
    }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Live Sessions</h2>' +
      '<p style="color:var(--text-secondary)">No active sessions. Select a past session to replay:</p>' +
      '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th></th></tr></thead><tbody>' + (rows||'<tr><td colspan="5">No sessions</td></tr>') + '</tbody></table></div>';
  }).catch(function(e) {
    document.getElementById('content').innerHTML = '<div class="card"><h2>Live Session</h2><p>Error: ' + e.message + '</p></div>';
  });
}

function renderCompliance() {
  api('/v1/compliance/frameworks').then(function(fw) {
    var opts = (fw.frameworks||[]).map(function(f) { return '<option value="' + f.id + '">' + f.name + '</option>'; }).join('');
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Compliance Reports</h2>' +
      '<div class="flex-row mb"><select id="fw">' + opts + '</select>' +
      '<input id="csid" placeholder="Session ID">' +
      '<button onclick="genReport()">Generate Report</button></div>' +
      '<div id="cr"></div></div>';
  });
}

function genReport() {
  var fw = document.getElementById('fw').value;
  var sid = document.getElementById('csid').value || 'unknown';
  api('/v1/compliance/report?framework=' + fw + '&session_id=' + sid).then(function(r) {
    var rows = (r.checks||[]).map(function(c) { return '<tr><td>' + (c.check||{}).id + '</td><td>' + (c.check||{}).requirement + '</td><td>' + (c.passed ? 'PASS' : 'FAIL') + '</td></tr>'; }).join('');
    document.getElementById('cr').innerHTML = '<div class="card"><h3>' + (r.framework||fw) + '</h3><p>Compliant: <strong>' + (r.overall_compliant ? 'YES' : 'NO') + '</strong></p><table><thead><tr><th>ID</th><th>Requirement</th><th>Passed</th></tr></thead><tbody>' + rows + '</tbody></table></div>';
  });
}

function renderAudit() { document.getElementById('content').innerHTML = '<div class="card"><h2>Audit Explorer</h2><p>Browse sessions from <a href="javascript:showPage(\'sessions\')">Sessions</a> to explore audit trails.</p></div>'; }
function renderSkills() {
  api('/v1/skills').then(function(d) {
    var cards = (d.skills||[]).map(function(s) {
      return '<div class="card"><h3>' + s.name + ' <small>v' + s.version + '</small></h3><p>' + s.description + '</p><p style="font-size:12px;color:var(--text-secondary)">Observers: ' + (s.observers||[]).join(', ') + ' | Detectors: ' + (s.detectors||[]).join(', ') + '</p></div>';
    }).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>Skills</h2><div class="grid-2">' + cards + '</div></div>';
  });
}
function renderMCP() {
  api('/v1/mcp/servers').then(function(d) {
    var rows = (d.servers||[]).map(function(s) { return '<tr><td>' + s.name + '</td><td>' + s.transport + '</td><td>' + s.endpoint + '</td></tr>'; }).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>MCP Servers</h2><table><thead><tr><th>Name</th><th>Transport</th><th>Endpoint</th></tr></thead><tbody>' + (rows||'<tr><td colspan="3">No servers</td></tr>') + '</tbody></table><div class="card" style="margin-top:16px"><h3>MCP Tools</h3><p>Start Forge MCP server from the CLI</p></div></div>';
  });
}
function renderExport() {
  api('/v1/export/configs').then(function(d) {
    var cards = (d.configs||[]).map(function(c) { return '<div class="card"><h3>' + c.target + '</h3><p>Enabled: ' + c.enabled + '</p></div>'; }).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>Export Targets</h2><div class="grid-3">' + cards + '</div></div>';
  });
}
function renderMarketplace() { document.getElementById('content').innerHTML = '<div class="card"><h2>Plugin Marketplace</h2><div class="flex-row mb"><input id="mq" placeholder="Search plugins..."><button onclick="searchMkt()">Search</button></div><div id="mkt-r"><p>Registry connection needed for full marketplace</p></div></div>'; }
function searchMkt() { api('/v1/marketplace/search?q=' + (document.getElementById('mq').value||'')).then(function(r) { document.getElementById('mkt-r').innerHTML = '<p>Results: ' + (r.total||0) + '</p>'; }); }
function renderCloud() {
  api('/v1/cloud/providers').then(function(d) {
    var cards = (d.providers||[]).map(function(p) { return '<div class="card"><h3>' + p.name.toUpperCase() + '</h3><p>Status: ' + p.status + '</p><p>Regions: ' + (p.regions||[]).join(', ') + '</p></div>'; }).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>Cloud Providers</h2><div class="grid-3">' + cards + '</div></div>';
  });
}
function renderAnalytics() {
  api('/v1/analytics/overview').then(function(o) {
    document.getElementById('content').innerHTML =
      '<div class="card"><h2>Analytics</h2>' +
      '<div class="stats-grid">' +
      '<div class="stat-card"><div class="stat-value">' + (o.total_sessions||0) + '</div><div class="stat-label">Total Sessions</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-green)">' + (o.completed||0) + '</div><div class="stat-label">Completed</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-yellow)">' + (o.running||0) + '</div><div class="stat-label">Running</div></div>' +
      '<div class="stat-card"><div class="stat-value" style="color:var(--accent-red)">' + (o.failed||0) + '</div><div class="stat-label">Failed</div></div>' +
      '</div></div>';
  });
}
function renderMeta() { document.getElementById('content').innerHTML = '<div class="card"><h2>Meta-Harness</h2><p>Self-improving harness. Requires 20+ completed sessions for pattern mining.</p></div>'; }
function renderAuth() {
  api('/v1/auth/config').then(function(c) {
    document.getElementById('content').innerHTML = '<div class="card"><h2>Authentication</h2><p>SSO: ' + (c.sso ? 'Configured' : 'Not configured') + '</p><p>MFA Required: ' + (c.mfa_required ? 'Yes' : 'No') + '</p><p>Providers: ' + (c.providers||[]).join(', ') + '</p></div>';
  });
}
function renderAdmin() {
  api('/v1/admin/keys').then(function(k) {
    var keyRows = (k.keys||[]).map(function(x) { return '<tr><td>' + (x.name||'') + '</td><td><code>' + (x.key||'').substring(0,16) + '...</code></td></tr>'; }).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>Admin</h2><h3>API Keys</h3><div class="flex-row mb"><input id="kn" placeholder="Key name"><button onclick="createKey()">Create Key</button></div><table><thead><tr><th>Name</th><th>Key</th></tr></thead><tbody>' + (keyRows||'<tr><td colspan="2">No keys</td></tr>') + '</tbody></table></div>';
  });
}
function createKey() { api('/v1/admin/keys', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({name:document.getElementById('kn').value||'default',scopes:['read:audit']})}).then(function(){showPage('admin');}); }
function renderSettings() {
  api('/v1/harness').then(function(h) {
    var obs = ((h.observers||{}).list||[]).map(function(o){return '<label class="toggle"><input type="checkbox" checked>' + o + '</label>';}).join('');
    var det = ((h.detectors||{}).list||[]).map(function(d){return '<label class="toggle"><input type="checkbox" checked>' + d + '</label>';}).join('');
    var stg = ((h.strategies||{}).list||[]).map(function(s){return '<label class="toggle"><input type="checkbox" checked>' + s + '</label>';}).join('');
    document.getElementById('content').innerHTML = '<div class="card"><h2>Harness Settings</h2><p>Version: ' + (h.version||'?') + '</p><h3>Observers (' + ((h.observers||{}).count||0) + ')</h3><div class="grid-4">' + obs + '</div><h3>Detectors (' + ((h.detectors||{}).count||0) + ')</h3><div class="grid-4">' + det + '</div><h3>Strategies (' + ((h.strategies||{}).count||0) + ')</h3><div class="grid-4">' + stg + '</div></div>';
  });
}
