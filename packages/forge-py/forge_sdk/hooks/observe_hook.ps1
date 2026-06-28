# Forge Observe Hook — PowerShell (Windows fallback)
# Receives Claude Code hook events via stdin, POSTs to Forge server.
# Usage: powershell -NoProfile -File observe_hook.ps1 -Port 3701
param([int]$Port = 3000, [string]$Url = "")

$ErrorActionPreference = "SilentlyContinue"

# Read stdin
$input = $input | Out-String
if ([string]::IsNullOrWhiteSpace($input)) { exit 0 }

try {
    $data = $input | ConvertFrom-Json
    if (-not $data -or -not $data.hook_event_name) { exit 0 }
} catch { exit 0 }

# Build ingest URL
if ($Url) {
    $ingestUrl = "$Url/api/v1/ingest/event"
} else {
    $portFile = Join-Path $HOME ".forge\port"
    if (Test-Path $portFile) { $Port = [int](Get-Content $portFile).Trim() }
    $ingestUrl = "http://127.0.0.1:$Port/api/v1/ingest/event"
}

# Build envelope
$flags = @{}
if ($data.hook_event_name -eq "SessionStart") { $flags.startsSession = $true; $flags.clearsNotification = $true }
if ($data.hook_event_name -eq "SessionEnd" -or $data.hook_event_name -eq "Stop") { $flags.stopsSession = $true }

$envelope = @{
    agentClass = "claude-code"
    sessionId = if ($data.session_id) { $data.session_id } else { "unknown" }
    agentId = if ($data.agent_id) { $data.agent_id } else { "root" }
    hookName = $data.hook_event_name
    toolName = if ($data.tool_name) { $data.tool_name } else { "" }
    payload = $data
    cwd = if ($data.cwd) { $data.cwd } else { (Get-Location).Path }
    timestamp = [int64]((Get-Date).ToUniversalTime() - (Get-Date "1970-01-01")).TotalMilliseconds
    flags = $flags
} | ConvertTo-Json -Depth 5 -Compress

# Fire-and-forget POST
try {
    Invoke-RestMethod -Uri $ingestUrl -Method POST -Body $envelope -ContentType "application/json" -TimeoutSec 5 | Out-Null
} catch {}
exit 0
