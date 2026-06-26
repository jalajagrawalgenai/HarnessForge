// Forge Dashboard
const API_BASE = '/api';
const API = {
  async get(path) { const r = await fetch(API_BASE + path); return r.json(); },
  async post(path, body) { const r = await fetch(API_BASE + path, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }); return r.json(); },
  async put(path, body) { const r = await fetch(API_BASE + path, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }); return r.json(); },
  async del(path) { const r = await fetch(API_BASE + path, { method: 'DELETE' }); return r.json(); },
  createSession(task, agentType, preset) { return this.post('/v1/sessions', { task, agent_type: agentType, preset }); },
  listSessions() { return this.get('/v1/sessions'); },
  getSession(id) { return this.get('/v1/sessions/' + id); },
  deleteSession(id) { return this.del('/v1/sessions/' + id); },
  pauseSession(id) { return this.post('/v1/sessions/' + id + '/pause'); },
  resumeSession(id) { return this.post('/v1/sessions/' + id + '/resume'); },
  getHarness() { return this.get('/v1/harness'); },
  getComplianceFrameworks() { return this.get('/v1/compliance/frameworks'); },
  getComplianceReport(framework, sessionId) { return this.get('/v1/compliance/report?framework=' + framework + '&session_id=' + sessionId); },
  getSkills() { return this.get('/v1/skills'); },
  composeSkills(skills) { return this.post('/v1/skills/compose', { skills }); },
  getMcpServers() { return this.get('/v1/mcp/servers'); },
  addMcpServer(name, transport, endpoint) { return this.post('/v1/mcp/servers', { name, transport, endpoint }); },
  discoverMcp() { return this.get('/v1/mcp/discover'); },
  getAuthConfig() { return this.get('/v1/auth/config'); },
  getExportConfigs() { return this.get('/v1/export/configs'); },
  searchMarketplace(q) { return this.get('/v1/marketplace/search?q=' + (q || '')); },
  getCloudProviders() { return this.get('/v1/cloud/providers'); },
  getAnalyticsOverview() { return this.get('/v1/analytics/overview'); },
  getWeaknesses() { return this.get('/v1/meta/weaknesses'); },
  getApiKeys() { return this.get('/v1/admin/keys'); },
  createApiKey(name, scopes) { return this.post('/v1/admin/keys', { name, scopes }); },
  getQuotas() { return this.get('/v1/admin/quotas'); },
  getHealth() { return this.get('/v1/health'); },
  getStatus() { return this.get('/v1/status'); },
};
// WebSocket Module
const WS = {
  ws: null, sessionId: null, onEvent: null, reconnectTimer: null,
  connect(sessionId, onEvent) {
    this.disconnect();
    this.sessionId = sessionId;
    this.onEvent = onEvent;
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    this.ws = new WebSocket(proto + '//' + location.host + '/ws');
    this.ws.onopen = () => { this.ws.send(JSON.stringify({ action: 'subscribe', session_id: sessionId })); };
    this.ws.onmessage = (evt) => { try { const d = JSON.parse(evt.data); if (onEvent) onEvent(d); } catch(e) {} };
    this.ws.onclose = () => { this.reconnectTimer = setTimeout(() => this.connect(sessionId, onEvent), 3000); };
  },
  disconnect() { if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); } if (this.ws) { this.ws.close(); this.ws = null; } },
  send(action, sid) { if (this.ws && this.ws.readyState === WebSocket.OPEN) this.ws.send(JSON.stringify({ action, session_id: sid || this.sessionId })); },
};

// Router
const Router = {
  routes: {},
  register(hash, renderer) { this.routes[hash] = renderer; },
  navigate(hash) { window.location.hash = hash; },
  current() { return window.location.hash.replace('#', '') || 'run'; },
  init() { window.addEventListener('hashchange', () => this.render()); this.render(); },
  render() {
    const page = this.current();
    document.querySelectorAll('#nav-tabs a').forEach(a => a.classList.toggle('active', a.getAttribute('href') === '#' + page));
    const renderer = this.routes[page] || this.routes['run'];
    document.getElementById('content').innerHTML = renderer ? renderer() : '<div class="card"><h2>404</h2></div>';
    if (page === 'live') { const sid = new URLSearchParams(location.hash.split('?')[1] || '').get('id'); if (sid) WS.connect(sid, handleLiveEvent); }
  }
};

// Components
function gaugeBar(name, value) {
  const pct = Math.round((value || 0) * 100);
  const cls = pct > 80 ? 'gauge-green' : pct > 50 ? 'gauge-yellow' : pct > 30 ? 'gauge-orange' : 'gauge-red';
  return '<div class="gauge"><div class="gauge-label">' + name + ' ' + pct + '%</div><div class="gauge-bar"><div class="gauge-fill ' + cls + '" style="width:' + pct + '%"></div></div></div>';
}
function statusBadge(status) { return '<span class="badge badge-' + (status || 'running') + '">' + (status || 'unknown') + '</span>'; }
function statCard(label, value, color) { return '<div class="stat-card"><div class="stat-value" style="color:' + (color || 'var(--accent-blue)') + '">' + value + '</div><div class="stat-label">' + label + '</div></div>'; }
// Run Page
function renderRun() {
  return '<div class="grid-2"><div class="card"><h2>Run Agent</h2><div class="form-group"><label>Task</label><textarea id="task-input" placeholder="Describe the task..."></textarea></div><div class="flex-row mb"><div class="form-group"><label>Agent Type</label><select id="agent-type"><option value="solo">Solo</option></select></div><div class="form-group"><label>Preset</label><select id="preset"><option value="solo">Solo</option></select></div></div><div class="flex-row"><button onclick="runTask()">Run with Harness</button><button class="warn" onclick="dryRunTask()">Dry Run</button></div><div id="run-result"></div></div><div class="card"><h2>Quick Stats</h2><div class="stats-grid" id="quick-stats">' + statCard('Sessions','...') + statCard('Tokens','0','var(--accent-green)') + statCard('Cost','$0.00','var(--accent-yellow)') + statCard('Health','100%','var(--accent-green)') + '</div><div id="recent-sessions"><h3>Recent</h3><p style="color:var(--text-secondary)">Loading...</p></div></div></div>';
}

async function runTask() {
  const task = document.getElementById('task-input').value || 'Default task';
  const at = document.getElementById('agent-type').value;
  const pr = document.getElementById('preset').value;
  document.getElementById('run-result').innerHTML = '<div class="card"><p>Starting...</p></div>';
  try {
    const r = await API.createSession(task, at, pr);
    document.getElementById('run-result').innerHTML = '<div class="card"><h3>Session Created</h3><p>ID: ' + r.id + '</p><p>' + statusBadge(r.status) + '</p><p><a href="#live?id=' + r.id + '" onclick="Router.navigate(\'live?id=' + r.id + '\')">View Live</a></p></div>';
    refreshQuickStats();
  } catch(e) { document.getElementById('run-result').innerHTML = '<div class="card" style="border-color:var(--accent-red)"><p>Error: ' + e.message + '</p></div>'; }
}
async function dryRunTask() {
  const task = document.getElementById('task-input').value || 'Default task';
  const at = document.getElementById('agent-type').value;
  const pr = document.getElementById('preset').value;
  try { const r = await API.createSession(task + ' [dry-run]', at, pr); document.getElementById('run-result').innerHTML = '<div class="card"><p>Dry run: ' + r.id + '</p></div>'; } catch(e) {}
}
async function refreshQuickStats() {
  try {
    const o = await API.getAnalyticsOverview();
    document.getElementById('quick-stats').innerHTML = statCard('Sessions', o.total_sessions||0) + statCard('Completed', o.completed||0, 'var(--accent-green)') + statCard('Running', o.running||0, 'var(--accent-yellow)') + statCard('Health', (o.avg_health*100||100)+'%','var(--accent-green)');
    const sessions = await API.listSessions();
    const html = (sessions.sessions||[]).slice(0,5).map(s => '<div style="padding:8px;border-bottom:1px solid var(--border)"><strong>' + (s.id||'').substring(0,8) + '</strong> ' + statusBadge(s.status) + ' <small>' + (s.agent_type||'') + '</small> <a href="#live?id=' + s.id + '">View</a></div>').join('');
    document.getElementById('recent-sessions').innerHTML = '<h3>Recent</h3>' + (html || '<p style="color:var(--text-secondary)">No sessions yet</p>');
  } catch(e) {}
}

// Sessions Page
function renderSessions() { setTimeout(refreshSessionsList, 100); return '<div class="card"><h2>Sessions</h2><div id="sessions-list"><p>Loading...</p></div></div>'; }
async function refreshSessionsList() {
  try {
    const d = await API.listSessions();
    const rows = (d.sessions||[]).map(s => '<tr><td>' + (s.id||'').substring(0,12) + '</td><td>' + (s.task||'').substring(0,40) + '</td><td>' + (s.agent_type||'') + '</td><td>' + statusBadge(s.status) + '</td><td>' + (s.result? s.result.observation_count : '-') + '</td><td>' + (s.result? s.result.detection_count : '-') + '</td><td>' + (s.result? s.result.intervention_count : '-') + '</td><td><a href="#live?id=' + s.id + '">Live</a> <a href="javascript:void(0)" onclick="deleteSession(\'' + s.id + '\')">Del</a></td></tr>').join('');
    document.getElementById('sessions-list').innerHTML = '<table><thead><tr><th>ID</th><th>Task</th><th>Agent</th><th>Status</th><th>Obs</th><th>Det</th><th>Int</th><th>Actions</th></tr></thead><tbody>' + (rows || '<tr><td colspan="8">No sessions</td></tr>') + '</tbody></table>';
  } catch(e) {}
}
async function deleteSession(id) { if (confirm('Delete ' + id + '?')) { await API.deleteSession(id); refreshSessionsList(); } }

// Live Session Page
function renderLive() {
  const sid = new URLSearchParams(location.hash.split('?')[1]||'').get('id');
  if (!sid) return '<div class="card"><h2>Live Session</h2><p>Select a session from <a href="#sessions">Sessions</a> or <a href="#run">Run</a>.</p></div>';
  return '<div class="live-layout"><div class="panel"><h2>Session: ' + sid.substring(0,12) + '</h2><div class="flex-row mb"><span id="session-status-text">connected</span><button onclick="WS.send(\'pause\')">Pause</button><button onclick="WS.send(\'resume\')">Resume</button></div><div class="conversation" id="conversation-stream"><p>Waiting for events...</p></div></div><div class="panel health-panel"><h3>Health</h3><div id="health-gauges">' + ['Token','Latency','Cost','Accuracy','Security','Reliability','Context','Orch','Compliance'].map(d => gaugeBar(d,1.0)).join('') + '</div><h3 style="margin-top:12px">Interventions</h3><div id="intervention-log"><p style="color:var(--text-secondary);font-size:12px">None yet</p></div></div></div>';
}

function handleLiveEvent(event) {
  const stream = document.getElementById('conversation-stream');
  if (!stream) return;
  const t = event.type || 'unknown';
  let cls = 'event', text = '';
  if (t === 'thinking_start') { cls += ' thinking'; text = 'D Thinking...'; }
  else if (t === 'tool_call_start') { cls += ' tool-call'; text = '> ' + (event.tool||'') + ': ' + JSON.stringify(event.args||{}); }
  else if (t === 'tool_call_end') { cls += ' tool-call'; text = '< ' + (event.tool||'') + ' done'; }
  else if (t === 'output_delta') { cls += ' output'; text = event.text || ''; }
  else if (t === 'completed') { cls += ' output'; text = 'OK Completed: ' + (event.summary||''); }
  else if (t === 'failed') { cls += ' detection'; text = 'XX Failed: ' + (event.error||''); }
  else if (t === 'started') { cls += ' thinking'; text = '>> Started: ' + (event.task||''); }
  else { text = JSON.stringify(event); }
  const div = document.createElement('div'); div.className = cls; div.textContent = text;
  stream.appendChild(div); stream.scrollTop = stream.scrollHeight;
  if (stream.children.length > 200) stream.removeChild(stream.firstChild);
}
// Compliance Page
function renderCompliance() {
  setTimeout(async () => {
    try {
      const fw = await API.getComplianceFrameworks();
      document.getElementById('fw-select').innerHTML = (fw.frameworks||[]).map(f => '<option value="' + f.id + '">' + f.name + '</option>').join('');
      const sessions = await API.listSessions();
      document.getElementById('cs-session').innerHTML = '<option value="">Select...</option>' + (sessions.sessions||[]).map(s => '<option value="' + s.id + '">' + (s.id||'').substring(0,8) + ' - ' + (s.task||'').substring(0,30) + '</option>').join('');
    } catch(e) {}
  }, 100);
  return '<div class="card"><h2>Compliance Reports</h2><div class="flex-row mb"><select id="fw-select"><option>Loading...</option></select><select id="cs-session"><option>Loading...</option></select><button onclick="genReport()">Generate</button></div><div id="cr-result"></div></div>';
}
async function genReport() {
  const fw = document.getElementById('fw-select').value;
  const sid = document.getElementById('cs-session').value || 'unknown';
  try {
    const r = await API.getComplianceReport(fw, sid);
    const rows = (r.checks||[]).map(c => '<tr><td>' + ((c.check||{}).id||'') + '</td><td>' + ((c.check||{}).requirement||'') + '</td><td>' + (c.passed ? 'PASS' : 'FAIL') + '</td></tr>').join('');
    document.getElementById('cr-result').innerHTML = '<div class="card"><h3>Report: ' + (r.framework||fw) + '</h3><p>Compliant: <strong>' + (r.overall_compliant ? 'YES' : 'NO') + '</strong></p><table><thead><tr><th>ID</th><th>Requirement</th><th>Passed</th></tr></thead><tbody>' + rows + '</tbody></table></div>';
  } catch(e) { document.getElementById('cr-result').innerHTML = '<p>Error: ' + e.message + '</p>'; }
}

// Skills Page
function renderSkills() {
  setTimeout(async () => {
    try { const d = await API.getSkills(); document.getElementById('sk-content').innerHTML = (d.skills||[]).map(s => '<div class="card"><h3>' + s.name + ' <small>v' + s.version + '</small></h3><p>' + (s.description||'') + '</p><p style="font-size:12px;color:var(--text-secondary)">Obs: ' + (s.observers||[]).join(', ') + ' | Det: ' + (s.detectors||[]).join(', ') + '</p></div>').join(''); } catch(e) {}
  }, 100);
  return '<div class="card"><h2>Skills</h2><div class="grid-2" id="sk-content"><p>Loading...</p></div></div>';
}

// MCP Page
function renderMcp() {
  setTimeout(async () => {
    try { const d = await API.getMcpServers(); document.getElementById('mcp-list').innerHTML = '<table><thead><tr><th>Name</th><th>Transport</th><th>Endpoint</th></tr></thead><tbody>' + ((d.servers||[]).map(s => '<tr><td>' + s.name + '</td><td>' + s.transport + '</td><td>' + s.endpoint + '</td></tr>').join('') || '<tr><td colspan="3">No servers</td></tr>') + '</tbody></table>'; } catch(e) {}
  }, 100);
  return '<div class="card"><h2>MCP Servers</h2><div class="flex-row mb"><input id="mcpn" placeholder="Name"><select id="mcpt"><option>stdio</option><option>sse</option><option>http</option></select><input id="mcpe" placeholder="Endpoint"><button onclick="addMcp()">Add</button></div><div id="mcp-list"><p>Loading...</p></div></div>';
}
async function addMcp() { const n = document.getElementById('mcpn').value; if (!n) return; await API.addMcpServer(n, document.getElementById('mcpt').value, document.getElementById('mcpe').value); Router.render(); }

// Export Page
function renderExport() {
  setTimeout(async () => {
    try { const d = await API.getExportConfigs(); document.getElementById('ex-content').innerHTML = (d.configs||[]).map(c => '<div class="card"><h3>' + c.target + '</h3><p>Enabled: ' + c.enabled + '</p></div>').join(''); } catch(e) {}
  }, 100);
  return '<div class="card"><h2>Export Targets</h2><div class="grid-3" id="ex-content"><p>Loading...</p></div></div>';
}

// Other pages (lightweight)
function renderAudit() { return '<div class="card"><h2>Audit Explorer</h2><p>Browse sessions from <a href="#sessions">Sessions</a>.</p></div>'; }
function renderMarketplace() { return '<div class="card"><h2>Marketplace</h2><div class="flex-row mb"><input id="mq" placeholder="Search..."><button onclick="searchMkt()">Search</button></div><div id="mkt-r"><p style="color:var(--text-secondary)">Registry connection needed</p></div></div>'; }
async function searchMkt() { try { const r = await API.searchMarketplace(document.getElementById('mq').value); document.getElementById('mkt-r').innerHTML = '<p>Results: ' + (r.total||0) + '</p>'; } catch(e) {} }

function renderCloud() {
  setTimeout(async () => {
    try { const d = await API.getCloudProviders(); document.getElementById('cl-content').innerHTML = (d.providers||[]).map(p => '<div class="card"><h3>' + p.name.toUpperCase() + '</h3><p>Status: ' + p.status + '</p><p>Regions: ' + (p.regions||[]).join(', ') + '</p></div>').join(''); } catch(e) {}
  }, 100);
  return '<div class="card"><h2>Cloud</h2><div class="grid-3" id="cl-content"><p>Loading...</p></div></div>';
}

function renderAnalytics() {
  setTimeout(async () => {
    try { const o = await API.getAnalyticsOverview(); document.getElementById('an-content').innerHTML = '<div class="stats-grid">' + statCard('Sessions',o.total_sessions||0) + statCard('Completed',o.completed||0,'var(--accent-green)') + statCard('Running',o.running||0,'var(--accent-yellow)') + statCard('Failed',o.failed||0,'var(--accent-red)') + '</div>'; } catch(e) {}
  }, 100);
  return '<div class="card"><h2>Analytics</h2><div id="an-content"><p>Loading...</p></div></div>';
}

function renderMeta() {
  setTimeout(async () => { try { const w = await API.getWeaknesses(); document.getElementById('mt-content').innerHTML = '<p>Weaknesses: ' + (w.total||0) + '</p><p style="color:var(--text-secondary)">Needs 20+ completed sessions.</p>'; } catch(e) {} }, 100);
  return '<div class="card"><h2>Meta-Harness</h2><div id="mt-content"><p>Loading...</p></div></div>';
}

function renderAuth() {
  setTimeout(async () => { try { const c = await API.getAuthConfig(); document.getElementById('au-content').innerHTML = '<p>SSO: ' + (c.sso ? 'Yes' : 'No') + ' | MFA: ' + (c.mfa_required ? 'Yes' : 'No') + ' | Providers: ' + (c.providers||[]).join(', ') + '</p>'; } catch(e) {} }, 100);
  return '<div class="card"><h2>Authentication</h2><div id="au-content"><p>Loading...</p></div></div>';
}

function renderAdmin() {
  setTimeout(async () => {
    try {
      const keys = await API.getApiKeys(); const quotas = await API.getQuotas();
      document.getElementById('ad-content').innerHTML = '<div class="card"><h3>API Keys</h3><div class="flex-row mb"><input id="kn" placeholder="Name"><input id="ks" placeholder="scopes"><button onclick="createKey()">Create</button></div><table>' + ((keys.keys||[]).map(k => '<tr><td>' + k.name + '</td><td><code>' + (k.key||'').substring(0,16) + '...</code></td><td>' + (k.scopes||[]).join(',') + '</td></tr>').join('')) + '</table></div><div class="card"><h3>Quotas</h3><table>
// Route Registration
Router.register("run", renderRun);
Router.register("sessions", renderSessions);
Router.register("live", renderLive);
Router.register("audit", renderAudit);
Router.register("compliance", renderCompliance);
Router.register("skills", renderSkills);
Router.register("mcp", renderMcp);
Router.register("export", renderExport);
Router.register("marketplace", renderMarketplace);
Router.register("cloud", renderCloud);
Router.register("analytics", renderAnalytics);
Router.register("meta", renderMeta);
Router.register("auth", renderAuth);
Router.register("admin", renderAdmin);
Router.register("settings", renderSettings);

(async function(){try{const h=await API.getHealth();document.getElementById("server-status").textContent="OK"}catch(e){document.getElementById("server-status").textContent="ERR"}Router.init();try{const h=await API.getHarness();const ao=(h.agent_types||["solo"]).map(t=>"<option>"+t+"</option>").join("");const po=(h.presets||["solo"]).map(p=>"<option>"+p+"</option>").join("");setTimeout(()=>{const at=document.getElementById("agent-type");if(at)at.innerHTML=ao;const pr=document.getElementById("preset");if(pr)pr.innerHTML=po},500)}catch(e){}})();
function renderCompliance(){setTimeout(async()=>{try{const d=await API.getComplianceFrameworks();document.getElementById("fw-sel").innerHTML=(d.frameworks||[]).map(f=>"<option value="+f.id+">"+f.name+"</option>").join("");const s=await API.listSessions();document.getElementById("cs-sel").innerHTML=(s.sessions||[]).map(s=>"<option value="+s.id+">"+(s.id||"").substring(0,8)+"</option>").join("")}catch(e){}},100);return "<div class=card><h2>Compliance</h2><select id=fw-sel></select><select id=cs-sel></select><button onclick=genCR()>Generate</button><div id=cr-r></div></div>"}
