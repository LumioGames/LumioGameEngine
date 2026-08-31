[CmdletBinding()]
param(
    [string]$NativeCoreRoot = '',
    [string]$VoxelRoot = '',
    [ValidateSet('debug', 'release')]
    [string]$Configuration = 'debug'
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$NativeCoreRoot = if ([string]::IsNullOrWhiteSpace($NativeCoreRoot)) { Join-Path $root '..\LumioNativeCore' } else { $NativeCoreRoot }
$VoxelRoot = if ([string]::IsNullOrWhiteSpace($VoxelRoot)) { Join-Path $root '..\LumioVoxelEngine' } else { $VoxelRoot }
$nativeCoreRoot = (Resolve-Path $NativeCoreRoot).Path
$voxelRoot = (Resolve-Path $VoxelRoot).Path
$sourceRoots = @($root, $nativeCoreRoot, $voxelRoot)
$excluded = '\\(\.git|target|bin|obj|\.build|\.run)(\\|$)'

function Get-SourceDigest([string[]]$roots) {
    $entries = [System.Collections.Generic.List[string]]::new()
    foreach ($sourceRoot in $roots) {
        $resolved = (Resolve-Path $sourceRoot).Path.TrimEnd('\')
        $files = Get-ChildItem -LiteralPath $resolved -Recurse -File |
            Where-Object { $_.FullName -notmatch $excluded } |
            Sort-Object FullName
        foreach ($file in $files) {
            $relative = $file.FullName.Substring($resolved.Length + 1).Replace('\', '/')
            $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            $entries.Add("$relative`0$hash")
        }
    }

    $payload = [Text.Encoding]::UTF8.GetBytes(($entries -join "`n"))
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        return ([BitConverter]::ToString($sha.ComputeHash($payload)) -replace '-', '').ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
}

$sourceSha256 = Get-SourceDigest $sourceRoots
$buildId = $sourceSha256.Substring(0, 32)
$abiPath = Join-Path $root 'engine\abi\native-abi.json'
$abiSha256 = (Get-FileHash -LiteralPath $abiPath -Algorithm SHA256).Hash.ToLowerInvariant()
$targetDir = Join-Path $root '.build\native-target'
$env:LUMIO_BUILD_ID = $buildId
$env:LUMIO_ABI_HASH = $abiSha256

$cargoArgs = @('--manifest-path', (Join-Path $root 'engine\native\Cargo.toml'), '-p', 'lumio-engine-native', '--target-dir', $targetDir)
if ($Configuration -eq 'release') {
    $cargoArgs += '--release'
}
& cargo build @cargoArgs
if ($LASTEXITCODE -ne 0) {
    throw "Native SDK build failed with exit code $LASTEXITCODE"
}

$nativeName = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'lumio_engine_native.dll' } else { 'liblumio_engine_native.so' }
$builtPath = Join-Path $targetDir (Join-Path $Configuration $nativeName)
if (-not (Test-Path -LiteralPath $builtPath)) {
    throw "Native SDK output not found: $builtPath"
}

$platform = if ($IsWindows -or $env:OS -eq 'Windows_NT') { 'win-x64' } else { 'linux-x64' }
$runRoot = Join-Path $root (Join-Path '.run' (Join-Path $buildId $platform))
New-Item -ItemType Directory -Force -Path $runRoot | Out-Null
$stagedPath = Join-Path $runRoot $nativeName
Copy-Item -LiteralPath $builtPath -Destination $stagedPath -Force
$binarySha256 = (Get-FileHash -LiteralPath $stagedPath -Algorithm SHA256).Hash.ToLowerInvariant()
$info = [ordered]@{
    buildId = $buildId
    sourceSha256 = $sourceSha256
    abiHash = $abiSha256
    binarySha256 = $binarySha256
    platform = $platform
    nativePath = $stagedPath
    configuration = $Configuration
}
$info | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $runRoot 'build-info.json') -Encoding utf8

Write-Output "BUILD_ID=$buildId"
Write-Output "ABI_HASH=$abiSha256"
Write-Output "NATIVE_PATH=$stagedPath"
Write-Output "BINARY_SHA256=$binarySha256"
