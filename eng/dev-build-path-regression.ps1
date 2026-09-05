[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$VoxelRoot,
    [string]$NativeCoreRoot = ''
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$voxelRoot = (Resolve-Path $VoxelRoot).Path
$nativeCoreRoot = if ([string]::IsNullOrWhiteSpace($NativeCoreRoot)) {
    (Resolve-Path (Join-Path $root '..\LumioNativeCore')).Path
} else {
    (Resolve-Path $NativeCoreRoot).Path
}
$contractPath = Join-Path $voxelRoot 'crates\lumio-voxel-contracts\wire\voxel-world-v1.json'
$contractBefore = (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash
$targetDir = Join-Path $root '.build\native-target'
if (Test-Path -LiteralPath $targetDir) {
    Remove-Item -LiteralPath $targetDir -Recurse -Force
}

$previousErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$output = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'eng\dev-build.ps1') `
    -NativeCoreRoot $nativeCoreRoot -VoxelRoot $voxelRoot 2>&1
$ErrorActionPreference = $previousErrorActionPreference
if ($LASTEXITCODE -ne 0) {
    throw "dev-build.ps1 failed with exit code $LASTEXITCODE"
}

$candidateCompilerLine = ($output | ForEach-Object { $_.ToString() }) |
    Where-Object { $_ -match 'Compiling lumio-voxel-(contracts|domain|ops|world) .*\(' -and $_ -like "*$voxelRoot*" } |
    Select-Object -First 1
if (-not $candidateCompilerLine) {
    throw "Native build did not compile a voxel crate from the supplied VoxelRoot '$voxelRoot'."
}

$contractAfter = (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash
if ($contractBefore -ne $contractAfter) {
    throw 'dev-build mutated the supplied voxel contract.'
}

Write-Output "VOXEL_BUILD_PATH_REGRESSION=$candidateCompilerLine"
