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
$workspaceDir = Join-Path $root '.build\native-workspace'
$env:LUMIO_BUILD_ID = $buildId
$env:LUMIO_ABI_HASH = $abiSha256

# sdk-native's checked-in manifest is intentionally repository-relative because it
# is consumed by the architecture checkout. Build in an ephemeral workspace so
# caller-supplied dependency roots are authoritative without rewriting either
# repository's source contracts.
if (Test-Path -LiteralPath $workspaceDir) {
    Remove-Item -LiteralPath $workspaceDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $workspaceDir | Out-Null
$nativeSourceRoot = Join-Path $root 'engine\native'
$excludedWorkspacePath = '\\(target|\.git|\.build|\.run)(\\|$)'
Get-ChildItem -LiteralPath $nativeSourceRoot -Recurse -File |
    Where-Object { $_.FullName -notmatch $excludedWorkspacePath } |
    ForEach-Object {
        $relative = $_.FullName.Substring($nativeSourceRoot.Length + 1)
        $destination = Join-Path $workspaceDir $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
        Copy-Item -LiteralPath $_.FullName -Destination $destination -Force
    }

$sdkManifest = Join-Path $workspaceDir 'modules\sdk-native\Cargo.toml'
$utf8 = [System.Text.UTF8Encoding]::new($false)
$sdkManifestText = [System.IO.File]::ReadAllText($sdkManifest, $utf8)
$nativeCoreCargoRoot = $nativeCoreRoot.Replace('\', '/')
$voxelCargoRoot = $voxelRoot.Replace('\', '/')
$sdkManifestText = $sdkManifestText -replace 'lumio-kernel = \{ path = "[^"]+" \}', ('lumio-kernel = { path = "' + $nativeCoreCargoRoot + '/crates/lumio-kernel" }')
$sdkManifestText = $sdkManifestText -replace 'lumio-voxel-world = \{ path = "[^"]+" \}', ('lumio-voxel-world = { path = "' + $voxelCargoRoot + '/crates/lumio-voxel-world" }')
$sdkManifestText = $sdkManifestText -replace 'lumio-voxel-domain = \{ path = "[^"]+" \}', ('lumio-voxel-domain = { path = "' + $voxelCargoRoot + '/crates/lumio-voxel-domain" }')
$sdkManifestText = $sdkManifestText -replace 'lumio-voxel-ops = \{ path = "[^"]+" \}', ('lumio-voxel-ops = { path = "' + $voxelCargoRoot + '/crates/lumio-voxel-ops" }')
$sdkManifestText = $sdkManifestText -replace 'lumio-voxel-contracts = \{ path = "[^"]+" \}', ('lumio-voxel-contracts = { path = "' + $voxelCargoRoot + '/crates/lumio-voxel-contracts" }')
$sdkManifestText = $sdkManifestText -replace 'lumio-timer = \{ path = "[^"]+" \}', ('lumio-timer = { path = "' + $nativeCoreCargoRoot + '/crates/lumio-timer" }')
[System.IO.File]::WriteAllText($sdkManifest, $sdkManifestText, $utf8)

$cargoArgs = @('--manifest-path', (Join-Path $workspaceDir 'Cargo.toml'), '-p', 'lumio-engine-native', '--target-dir', $targetDir)
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
# BOM-less UTF-8: Set-Content -Encoding utf8 on Windows PowerShell 5.1 emits a BOM,
# which strict JSON parsers (Node, serde_json) reject per RFC 8259. Keep this file
# ASCII-only: PS 5.1 reads BOM-less UTF-8 scripts as ANSI and non-ASCII comments
# can silently corrupt adjacent statements.
$infoJson = $info | ConvertTo-Json
[System.IO.File]::WriteAllText((Join-Path $runRoot 'build-info.json'), $infoJson, [System.Text.UTF8Encoding]::new($false))

Write-Output "BUILD_ID=$buildId"
Write-Output "ABI_HASH=$abiSha256"
Write-Output "NATIVE_PATH=$stagedPath"
Write-Output "BINARY_SHA256=$binarySha256"
