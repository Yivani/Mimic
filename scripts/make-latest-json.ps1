# Assembles release\latest.json from the signed build artifacts so you can
# upload it (plus the installer) to your update host.
#
# Run AFTER a signed build:
#   $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\mimic_updater.key -Raw
#   npm run tauri:build
#   .\scripts\make-latest-json.ps1 -HostUrl "https://updates.yourdomain.com"
#
# Then upload release\latest.json and release\Mimic_*_x64-setup.exe to:
#   <HostUrl>/mimic/

param(
  [string]$HostUrl = "https://REPLACE-WITH-YOUR-HETZNER-HOST",
  [string]$Notes = "Bug fixes and improvements."
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

$conf = Get-Content "$root\src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
$version = $conf.version

$nsis = Get-ChildItem "$root\src-tauri\target\release\bundle\nsis\*_x64-setup.exe" |
  Select-Object -First 1
if (-not $nsis) { throw "No NSIS installer found. Run a signed 'npm run tauri:build' first." }
$sig = (Get-Content "$($nsis.FullName).sig" -Raw).Trim()

$manifest = [ordered]@{
  version   = $version
  notes     = $Notes
  pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = $sig
      url       = "$HostUrl/mimic/$($nsis.Name)"
    }
  }
}

New-Item -ItemType Directory -Force "$root\release" | Out-Null
$manifest | ConvertTo-Json -Depth 6 | Set-Content "$root\release\latest.json" -Encoding utf8
Copy-Item $nsis.FullName "$root\release\" -Force

Write-Host "Wrote release\latest.json (v$version) and copied $($nsis.Name)."
Write-Host "Upload both files to $HostUrl/mimic/"
