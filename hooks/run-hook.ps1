param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("pre-tool-use", "post-tool-use")]
    [string]$Event
)

$ErrorActionPreference = "Stop"
$current = (Get-Location).Path

while ($true) {
    $memory = Join-Path $current "guru-maker"
    $marker = Join-Path $current ".guru-maker/.gitignore"
    if (
        (Test-Path -LiteralPath $memory -PathType Container) -and
        (Test-Path -LiteralPath $marker -PathType Leaf)
    ) {
        $env:GURUMAKER_HOOK_WORKSPACE = $current
        $pluginRoot = if ($env:CLAUDE_PLUGIN_ROOT) {
            $env:CLAUDE_PLUGIN_ROOT
        } else {
            $env:PLUGIN_ROOT
        }
        if (-not $pluginRoot) {
            throw "Gibo hook error: plugin root is unavailable"
        }
        $wrapper = Join-Path $pluginRoot "shared/gurumaker/scripts/gurumaker.ps1"
        & $wrapper hook $Event
        exit $LASTEXITCODE
    }

    $parent = Split-Path -Parent $current
    if (-not $parent -or $parent -eq $current) {
        exit 0
    }
    $current = $parent
}
