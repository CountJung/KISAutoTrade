param(
    [string]$CodexHome = $(if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $HOME ".codex" })
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
$sourceRoot = Join-Path $projectRoot ".codex\skills"
$targetRoot = Join-Path $CodexHome "skills"

if (-not (Test-Path $sourceRoot)) {
    throw "Project Codex skills directory not found: $sourceRoot"
}

New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null

Get-ChildItem -Path $sourceRoot -Directory -Filter "kisautotrade-*" | ForEach-Object {
    $target = Join-Path $targetRoot $_.Name
    if (Test-Path $target) {
        Remove-Item -LiteralPath $target -Recurse -Force
    }
    Copy-Item -LiteralPath $_.FullName -Destination $target -Recurse -Force
    Write-Host "Synced $($_.Name) -> $target"
}

Write-Host "Codex skill sync complete. Restart Codex if the skill list was already loaded."
