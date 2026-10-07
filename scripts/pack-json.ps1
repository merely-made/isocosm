# No hand-written JSON in a pack (mere's data formats brief F1; wing design
# record, ruling 624). Authored pack files are TOML; JSON under a `packs/`
# directory is allowed only below `fixtures/`, which holds recorded runs.
# Packs written elsewhere may still be JSON: this polices this repository.
# With -Staged it checks only what the commit being made adds or changes.
param([switch]$Staged)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent

Push-Location -LiteralPath $repoRoot
try {
    $paths = if ($Staged) { git diff --cached --name-only --diff-filter=ACMR } else { git ls-files }
    $offenders = foreach ($path in $paths) {
        $segments = $path -split '/'
        if ([IO.Path]::GetExtension($path) -ne '.json') { continue }
        if ($segments -notcontains 'packs') { continue }
        if ($segments -contains 'fixtures') { continue }
        $path
    }
} finally {
    Pop-Location
}
if ($offenders) {
    Write-Output 'Pack files people write are TOML (data formats brief F1, ruling 624); JSON in a pack belongs under fixtures/:'
    $offenders | Sort-Object | Write-Output
    exit 1
}
exit 0
