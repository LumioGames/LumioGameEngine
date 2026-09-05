[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$VoxelRoot,
    [string]$NativeCoreRoot = '',
    [string]$ServerRoot = '',
    [string]$RuntimeRoot = ''
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$voxelRoot = (Resolve-Path $VoxelRoot).Path
if ([string]::IsNullOrWhiteSpace($ServerRoot)) {
    $ServerRoot = [Environment]::GetEnvironmentVariable('LumioServerRoot')
}
if ([string]::IsNullOrWhiteSpace($RuntimeRoot)) {
    $RuntimeRoot = [Environment]::GetEnvironmentVariable('LumioRuntimeRoot')
}
if ([string]::IsNullOrWhiteSpace($ServerRoot) -or [string]::IsNullOrWhiteSpace($RuntimeRoot)) {
    throw 'LumioServerRoot and LumioRuntimeRoot are required for the dev-run VoxelRoot regression.'
}

$env:LumioServerRoot = (Resolve-Path $ServerRoot).Path
$env:LumioRuntimeRoot = (Resolve-Path $RuntimeRoot).Path
$previousErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$output = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'eng\dev-run.ps1') `
    -VoxelRoot $voxelRoot 2>&1
$runExitCode = $LASTEXITCODE
$ErrorActionPreference = $previousErrorActionPreference
if ($runExitCode -ne 0) {
    throw "dev-run.ps1 failed with exit code $runExitCode`n$output"
}

$lines = $output | ForEach-Object { $_.ToString() }
$sdkManifest = Join-Path $root '.build\native-workspace\modules\sdk-native\Cargo.toml'
if (-not (Test-Path -LiteralPath $sdkManifest)) {
    throw "dev-run did not leave the native build workspace manifest at '$sdkManifest'."
}
$manifestText = Get-Content -LiteralPath $sdkManifest -Raw
if ($manifestText -notmatch [regex]::Escape($voxelRoot.Replace('\', '/'))) {
    throw "dev-run native workspace manifest does not select VoxelRoot '$voxelRoot'."
}
$serverLine = $lines | Where-Object { $_ -match '^SERVER SERVER_READY ' } | Select-Object -First 1
$clientLine = $lines | Where-Object { $_ -match '^CLIENT .*ENGINE_NATIVE ' } | Select-Object -First 1
if (-not $serverLine -or -not $clientLine) {
    throw "dev-run did not emit SERVER_READY and ENGINE_NATIVE proofs.`n$output"
}
if ($clientLine -notmatch 'buildId=') {
    throw "ENGINE_NATIVE proof did not include build identity: $clientLine"
}

Write-Output "DEV_RUN_VOXEL_ROOT_REGRESSION=manifest:$sdkManifest voxelRoot:$voxelRoot"
Write-Output $serverLine
Write-Output $clientLine
Write-Output 'DEV_RUN_PROCESSES_CLEANED=true'
