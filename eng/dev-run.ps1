[CmdletBinding()]
param(
    [switch]$KeepRunning
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$buildOutput = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'dev-build.ps1')
if ($LASTEXITCODE -ne 0) {
    throw "dev-build.ps1 failed with exit code $LASTEXITCODE"
}
$buildLine = $buildOutput | Where-Object { $_ -like 'NATIVE_PATH=*' } | Select-Object -Last 1
$nativePath = $buildLine.Substring('NATIVE_PATH='.Length)

$secret = Join-Path ([IO.Path]::GetTempPath()) "lumio-dev-secret-$PID.txt"
Set-Content -LiteralPath $secret -Value 'lumio-dev-secret' -NoNewline
$serverArgs = @(
    'run', '--project', (Join-Path $root '..\LumioServer\mvp-host\src\Lumio.Server.MvpHost.App\Lumio.Server.MvpHost.App.csproj'), '--no-build', '--',
    '--allow-insecure-loopback', '--shared-secret-file', $secret, '--engine-native', $nativePath
)
$clientArgs = @(
    'run', '--project', (Join-Path $root '..\LumioClient\modules\bot\host\Lumio.Client.Bot.Host.csproj'), '--no-build', '--',
    'foundation', '--engine-native', $nativePath
)
$serverLog = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-$PID.log"
$clientLog = Join-Path ([IO.Path]::GetTempPath()) "lumio-client-$PID.log"
$serverErr = Join-Path ([IO.Path]::GetTempPath()) "lumio-server-$PID.err"
$clientErr = Join-Path ([IO.Path]::GetTempPath()) "lumio-client-$PID.err"
$server = Start-Process dotnet -ArgumentList $serverArgs -RedirectStandardOutput $serverLog -RedirectStandardError $serverErr -PassThru -WindowStyle Hidden
$client = Start-Process dotnet -ArgumentList $clientArgs -RedirectStandardOutput $clientLog -RedirectStandardError $clientErr -PassThru -WindowStyle Hidden
try {
    $expectedBuildId = ($buildOutput | Where-Object { $_ -like 'BUILD_ID=*' } | Select-Object -Last 1).Substring(9)
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
        $serverText = if (Test-Path $serverLog) { Get-Content -Raw $serverLog } else { '' }
        $clientText = if (Test-Path $clientLog) { Get-Content -Raw $clientLog } else { '' }
        if ($serverText -match 'ENGINE_NATIVE ' -and $clientText -match 'ENGINE_NATIVE ') {
            $serverLine = ($serverText -split "`r?`n" | Where-Object { $_ -match 'ENGINE_NATIVE ' } | Select-Object -First 1)
            $clientLine = ($clientText -split "`r?`n" | Where-Object { $_ -match 'ENGINE_NATIVE ' } | Select-Object -First 1)
            Write-Output "SERVER $serverLine"
            Write-Output "CLIENT $clientLine"
            if ($serverLine -notmatch [regex]::Escape("buildId=$expectedBuildId")) {
                throw 'Server BuildId proof does not match dev-build output.'
            }
            if ($clientLine -notmatch [regex]::Escape("buildId=$expectedBuildId")) {
                throw 'Client BuildId proof does not match dev-build output.'
            }
            break
        }
        if ($server.HasExited -and $server.ExitCode -ne 0) { throw "Server exited with $($server.ExitCode): $serverText" }
        if ($client.HasExited -and $client.ExitCode -ne 0) { throw "Client exited with $($client.ExitCode): $clientText" }
    } while ([DateTime]::UtcNow -lt $deadline)

    if ($serverText -notmatch 'ENGINE_NATIVE ' -or $clientText -notmatch 'ENGINE_NATIVE ') {
        throw "Timed out waiting for both Host proofs. Server: $serverText Client: $clientText"
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
    Remove-Item -LiteralPath $secret, $serverLog, $clientLog, $serverErr, $clientErr -Force -ErrorAction SilentlyContinue
}
