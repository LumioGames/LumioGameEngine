[CmdletBinding()]
param(
    [switch]$KeepRunning
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$isWindows = $IsWindows -or $env:OS -eq 'Windows_NT'

function Get-RequiredEnvPath([string]$name, [string]$why) {
    $value = [Environment]::GetEnvironmentVariable($name)
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "BLOCKED: $name is not set. $why Do not hard-code a machine path."
    }
    if (-not (Test-Path -LiteralPath $value)) {
        throw "BLOCKED: $name path does not exist: $value"
    }
    return (Resolve-Path -LiteralPath $value).Path
}

function Find-HostFxr {
    if (-not [string]::IsNullOrWhiteSpace($env:LUMIO_HOSTFXR) -and (Test-Path -LiteralPath $env:LUMIO_HOSTFXR)) {
        return (Resolve-Path -LiteralPath $env:LUMIO_HOSTFXR).Path
    }
    $name = if ($isWindows) { 'hostfxr.dll' } else { 'libhostfxr.so' }
    $roots = @()
    if (-not [string]::IsNullOrWhiteSpace($env:DOTNET_ROOT)) { $roots += $env:DOTNET_ROOT }
    $dotnet = Get-Command dotnet -ErrorAction SilentlyContinue
    if ($dotnet -and $dotnet.Source) { $roots += (Split-Path -Parent $dotnet.Source) }
    if (-not [string]::IsNullOrWhiteSpace($env:USERPROFILE)) { $roots += (Join-Path $env:USERPROFILE '.dotnet') }
    if (-not [string]::IsNullOrWhiteSpace($env:HOME)) { $roots += (Join-Path $env:HOME '.dotnet') }
    foreach ($candidateRoot in $roots) {
        $fxr = Join-Path $candidateRoot (Join-Path 'host' 'fxr')
        if (-not (Test-Path -LiteralPath $fxr)) { continue }
        $versions = Get-ChildItem -LiteralPath $fxr -Directory -ErrorAction SilentlyContinue | Sort-Object Name
        foreach ($version in $versions) {
            $dll = Join-Path $version.FullName $name
            if (Test-Path -LiteralPath $dll) { return (Resolve-Path -LiteralPath $dll).Path }
        }
    }
    throw 'BLOCKED: hostfxr not found. Set LUMIO_HOSTFXR or DOTNET_ROOT.'
}

function Find-RustHostExe([string]$serverRoot, [string]$binName) {
    $exeName = if ($isWindows) { "$binName.exe" } else { $binName }
    foreach ($profile in @('debug', 'release')) {
        $candidate = Join-Path $serverRoot (Join-Path 'target' (Join-Path $profile $exeName))
        if (Test-Path -LiteralPath $candidate) { return (Resolve-Path -LiteralPath $candidate).Path }
    }
    return $null
}

function Find-HelloEntry([string]$archRoot) {
    $runtimeRoot = $env:LumioRuntimeRoot
    if ([string]::IsNullOrWhiteSpace($runtimeRoot)) {
        $runtimeRoot = Join-Path $archRoot (Join-Path '..' 'LumioGameRuntime')
    }
    if (-not (Test-Path -LiteralPath $runtimeRoot)) {
        throw "BLOCKED: LumioGameRuntime not found at $runtimeRoot (set LumioRuntimeRoot). Needed to start the rust host."
    }
    $runtimeRoot = (Resolve-Path -LiteralPath $runtimeRoot).Path
    foreach ($cfg in @('Debug', 'Release')) {
        $dir = Join-Path $runtimeRoot "modules\hello\entry\bin\$cfg\net10.0"
        $dll = Join-Path $dir 'Lumio.GameRuntime.HelloEntry.dll'
        $rc = Join-Path $dir 'Lumio.GameRuntime.HelloEntry.runtimeconfig.json'
        if ((Test-Path -LiteralPath $dll) -and (Test-Path -LiteralPath $rc)) {
            return @{ Assembly = (Resolve-Path $dll).Path; RuntimeConfig = (Resolve-Path $rc).Path }
        }
    }
    throw 'BLOCKED: Lumio.GameRuntime.HelloEntry.dll / runtimeconfig missing. Build Runtime modules/hello/entry or set LumioRuntimeRoot.'
}

$serverRoot = Get-RequiredEnvPath 'LumioServerRoot' 'Point it at the LumioServer repo root so this script can start the rust host (lumio-entity-chat-replay or lumio-server).'
$processToml = Join-Path $serverRoot (Join-Path 'modules' (Join-Path 'process' 'Cargo.toml'))
if (-not (Test-Path -LiteralPath $processToml)) {
    throw "BLOCKED: LumioServerRoot is not a LumioServer repo (missing modules/process/Cargo.toml): $serverRoot"
}
$tomlText = Get-Content -LiteralPath $processToml -Raw
if ($tomlText -notmatch 'lumio-entity-chat-replay') {
    throw 'BLOCKED: LumioServerRoot does not declare bin lumio-entity-chat-replay.'
}

$buildOutput = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'dev-build.ps1')
if ($LASTEXITCODE -ne 0) {
    throw "dev-build.ps1 failed with exit code $LASTEXITCODE"
}
$buildLine = $buildOutput | Where-Object { $_ -like 'NATIVE_PATH=*' } | Select-Object -Last 1
$nativePath = $buildLine.Substring('NATIVE_PATH='.Length)
$expectedBuildId = ($buildOutput | Where-Object { $_ -like 'BUILD_ID=*' } | Select-Object -Last 1).Substring(9)
$expectedAbiHash = ($buildOutput | Where-Object { $_ -like 'ABI_HASH=*' } | Select-Object -Last 1)
if ($expectedAbiHash) { $expectedAbiHash = $expectedAbiHash.Substring(9) }

$serverExe = Find-RustHostExe $serverRoot 'lumio-server'
if (-not $serverExe) {
    $cargoToml = Join-Path $serverRoot 'Cargo.toml'
    & cargo build --manifest-path $cargoToml --bin lumio-server --bin lumio-entity-chat-replay
    if ($LASTEXITCODE -ne 0) {
        throw "BLOCKED: cargo build of lumio-server / lumio-entity-chat-replay failed with exit code $LASTEXITCODE"
    }
    $serverExe = Find-RustHostExe $serverRoot 'lumio-server'
    if (-not $serverExe) {
        throw 'BLOCKED: lumio-server binary missing after cargo build.'
    }
}
$replayExe = Find-RustHostExe $serverRoot 'lumio-entity-chat-replay'
$hello = Find-HelloEntry $root
$hostfxr = Find-HostFxr
$wireContract = Join-Path $root (Join-Path 'engine' (Join-Path 'wire' 'hello-wire-v1.json'))
if (-not (Test-Path -LiteralPath $wireContract)) {
    throw "BLOCKED: hello-wire-v1.json missing at $wireContract"
}

$secret = Join-Path ([IO.Path]::GetTempPath()) "lumio-dev-secret-$PID.txt"
Set-Content -LiteralPath $secret -Value 'lumio-dev-secret' -NoNewline
$serverLog = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-$PID.log"
$clientLog = Join-Path ([IO.Path]::GetTempPath()) "lumio-client-$PID.log"
$serverErr = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-$PID.err"
$clientErr = Join-Path ([IO.Path]::GetTempPath()) "lumio-client-$PID.err"
$auditFile = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-audit-$PID.ndjson"
$readyFile = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-ready-$PID.json"

function Quote-WinArg([string]$value) {
    if ($null -eq $value) { return '""' }
    if ($value -notmatch '[\s,"]') { return $value }
    return '"' + ($value -replace '"', '\"') + '"'
}
# Start-Process -ArgumentList splits on commas; pass one quoted command line instead.
$serverArgs = @(
    '--engine-native', (Quote-WinArg $nativePath),
    '--hostfxr', (Quote-WinArg $hostfxr),
    '--runtime-config', (Quote-WinArg $hello.RuntimeConfig),
    '--assembly', (Quote-WinArg $hello.Assembly),
    '--entry-type', (Quote-WinArg 'Lumio.GameRuntime.HelloEntry.HelloEntry, Lumio.GameRuntime.HelloEntry'),
    '--entry-method', 'LumioHelloEntry',
    '--wire-contract', (Quote-WinArg $wireContract),
    '--audit-file', (Quote-WinArg $auditFile),
    '--ready-file', (Quote-WinArg $readyFile)
) -join ' '
$clientProject = Join-Path $root '..\LumioClient\modules\bot\host\Lumio.Client.Bot.Host.csproj'
$clientArgs = @(
    'run', '--project', $clientProject, '--no-build', '--',
    'foundation', '--engine-native', $nativePath
)

Write-Output "RUST_HOST exe=$serverExe replay=$replayExe (lumio-server is the long-running entry of lumio-server-process; lumio-entity-chat-replay is the batch suite runner in the same crate)"
$server = Start-Process -FilePath $serverExe -ArgumentList $serverArgs -RedirectStandardOutput $serverLog -RedirectStandardError $serverErr -PassThru -WindowStyle Hidden
$client = Start-Process dotnet -ArgumentList $clientArgs -RedirectStandardOutput $clientLog -RedirectStandardError $clientErr -PassThru -WindowStyle Hidden
try {
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    $serverText = ''
    $clientText = ''
    do {
        Start-Sleep -Milliseconds 250
        $serverText = if (Test-Path $serverLog) { Get-Content -Raw $serverLog } else { '' }
        $serverText += if (Test-Path $serverErr) { Get-Content -Raw $serverErr } else { '' }
        $clientText = if (Test-Path $clientLog) { Get-Content -Raw $clientLog } else { '' }
        if ($serverText -match 'SERVER_READY ' -and $clientText -match 'ENGINE_NATIVE ') {
            $serverLine = ($serverText -split "`r?`n" | Where-Object { $_ -match 'SERVER_READY ' } | Select-Object -First 1)
            $clientLine = ($clientText -split "`r?`n" | Where-Object { $_ -match 'ENGINE_NATIVE ' } | Select-Object -First 1)
            Write-Output "SERVER $serverLine"
            Write-Output "CLIENT $clientLine"
            if ($clientLine -notmatch [regex]::Escape("buildId=$expectedBuildId")) {
                throw 'Client BuildId proof does not match dev-build output.'
            }
            break
        }
        if ($server.HasExited -and $server.ExitCode -ne 0) { throw "BLOCKED: rust host exited with $($server.ExitCode): $serverText" }
        if ($client.HasExited -and $client.ExitCode -ne 0) { throw "Client exited with $($client.ExitCode): $clientText" }
    } while ([DateTime]::UtcNow -lt $deadline)

    if ($serverText -notmatch 'SERVER_READY ' -or $clientText -notmatch 'ENGINE_NATIVE ') {
        throw "Timed out waiting for rust host SERVER_READY and client ENGINE_NATIVE proofs. Server: $serverText Client: $clientText"
    }

    if ($KeepRunning) {
        Write-Output "HOSTS_RUNNING native=$nativePath serverPid=$($server.Id) clientPid=$($client.Id)"
        Wait-Process -Id $server.Id, $client.Id
    }
}
finally {
    if (-not $KeepRunning) {
        foreach ($process in @($server, $client)) {
            if ($process -and -not $process.HasExited) {
                Stop-Process -Id $process.Id -Force
            }
        }
    }
    Remove-Item -LiteralPath $secret, $serverLog, $clientLog, $serverErr, $clientErr, $auditFile, $readyFile -Force -ErrorAction SilentlyContinue
}
