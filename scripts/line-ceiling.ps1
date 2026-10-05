# The 600-line ceiling (ruling 592): no tracked source file over 600 lines
# without a technical reason. A file may exceed it only when
# `line-ceiling-exceptions.txt` lists its path with the reason, one
# `path<TAB>reason` per line. Exits 1 and names every offender otherwise.
# With -Staged it checks only what the commit being made adds or changes,
# as staged, which is what the pre-commit hook runs.
param([switch]$Staged)

$ErrorActionPreference = 'Stop'
$Ceiling = 600
$Source = @('.rs', '.ps1', '.py', '.js', '.mjs', '.ts', '.wgsl', '.wesl', '.lua', '.glsl', '.sh', '.html', '.css')
$repoRoot = Split-Path $PSScriptRoot -Parent

$excepted = @{}
$exceptions = Join-Path $PSScriptRoot 'line-ceiling-exceptions.txt'
if (Test-Path -LiteralPath $exceptions) {
    foreach ($line in Get-Content -LiteralPath $exceptions) {
        if ($line -match '^\s*(#|$)') { continue }
        $path, $reason = $line -split "`t", 2
        if (-not $reason) { throw "line-ceiling exception without a reason: $path" }
        $excepted[$path] = $reason
    }
}

Push-Location -LiteralPath $repoRoot
try {
    $paths = if ($Staged) { git diff --cached --name-only --diff-filter=ACMR } else { git ls-files }
    $offenders = foreach ($path in $paths) {
        if ($Source -notcontains [IO.Path]::GetExtension($path)) { continue }
        $lines = if ($Staged) {
            @(git show ":$path").Count
        } elseif (Test-Path -LiteralPath $path) {
            [IO.File]::ReadAllLines((Join-Path $repoRoot $path)).Count
        } else { 0 }
        if ($lines -gt $Ceiling -and -not $excepted.ContainsKey($path)) {
            '{0,6} {1}' -f $lines, $path
        }
    }
} finally {
    Pop-Location
}
if ($offenders) {
    Write-Output "Over the $Ceiling-line ceiling without a listed reason (ruling 592):"
    $offenders | Sort-Object -Descending | Write-Output
    exit 1
}
exit 0
