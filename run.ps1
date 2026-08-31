$nodeDir = Join-Path $env:LOCALAPPDATA "Programs\node-portable\node-v22.20.0-win-x64"
if (Test-Path (Join-Path $nodeDir "node.exe")) {
  $env:PATH = "$nodeDir;$env:PATH"
}
Set-Location $PSScriptRoot
npm run tauri dev
