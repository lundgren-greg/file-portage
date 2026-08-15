<#
.SYNOPSIS
    Push docs/wiki to the GitHub Wiki after the Wiki tab has been initialized.

.DESCRIPTION
    GitHub does not create <repo>.wiki.git until someone saves the first page
    in the UI. After that one click, this script copies docs/wiki/*.md into
    C:\Repos\portage-app.wiki (or a temp clone) and pushes master.
#>
[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$WikiDir = "C:\Repos\portage-app.wiki",
    [string]$SourceDir = (Join-Path (Split-Path -Parent $PSScriptRoot) "docs\wiki")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$remote = "https://github.com/lundgren-greg/portage-app.wiki.git"
$probe = git ls-remote $remote 2>&1
if ($LASTEXITCODE -ne 0) {
    throw @"
Wiki git remote is not initialized yet.

1. Open https://github.com/lundgren-greg/portage-app/wiki
2. Click Create the first page and save (title Home is fine).
3. Re-run this script.
"@
}

if (-not (Test-Path (Join-Path $WikiDir ".git"))) {
    New-Item -ItemType Directory -Force -Path $WikiDir | Out-Null
    git clone $remote $WikiDir
}

Get-ChildItem -Path $SourceDir -Filter "*.md" |
    Where-Object { $_.Name -ne "README.md" } |
    ForEach-Object {
        Copy-Item -Path $_.FullName -Destination (Join-Path $WikiDir $_.Name) -Force
    }

Push-Location $WikiDir
try {
    git add -A
    $pending = git status --porcelain
    if (-not $pending) {
        Write-Output "Wiki already up to date."
        return
    }
    if ($PSCmdlet.ShouldProcess($remote, "Commit and push wiki")) {
        git commit -m "Sync wiki from docs/wiki."
        git push origin HEAD:master
    }
}
finally {
    Pop-Location
}
