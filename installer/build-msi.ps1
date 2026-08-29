[CmdletBinding()]
param(
    [switch]$CopyToDownloads
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$publishDirectory = Join-Path $projectRoot 'target\release'
$executablePath = Join-Path $publishDirectory 'pulseclick.exe'
$wixPath = Join-Path $projectRoot '.tools\wix4\tools\net6.0\any\wix.exe'
$uiExtensionPath = Join-Path $projectRoot '.tools\WixToolset.UI.wixext4\wixext4\WixToolset.UI.wixext.dll'
$wixSourcePath = Join-Path $projectRoot 'installer\PulseClick.wxs'
$outputDirectory = Join-Path $projectRoot 'dist'

if (-not (Test-Path -LiteralPath $executablePath)) {
    Write-Host 'Release executable not found. Building PulseClick...'
    & cargo build --release --manifest-path (Join-Path $projectRoot 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) {
        throw 'The Rust release build failed.'
    }
}

if (-not (Test-Path -LiteralPath $wixPath) -or -not (Test-Path -LiteralPath $uiExtensionPath)) {
    & (Join-Path $PSScriptRoot 'prepare-wix.ps1')
    if ($LASTEXITCODE -ne 0) {
        throw 'WiX setup failed.'
    }
}

$versionLine = Select-String -LiteralPath (Join-Path $projectRoot 'Cargo.toml') -Pattern '^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"' | Select-Object -First 1
if (-not $versionLine) {
    throw 'Could not read the three-part application version from Cargo.toml.'
}
$appVersion = $versionLine.Matches[0].Groups[1].Value

New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$msiPath = Join-Path $outputDirectory "PulseClick-Setup-$appVersion-x64.msi"

$wixArguments = @(
    'build'
    $wixSourcePath
    '-arch'
    'x64'
    '-ext'
    $uiExtensionPath
    '-d'
    "ProjectDir=$projectRoot"
    '-d'
    "PublishDir=$publishDirectory"
    '-d'
    "AppVersion=$appVersion"
    '-pdbtype'
    'none'
    '-o'
    $msiPath
)

Write-Host "Building $msiPath..."
& $wixPath @wixArguments
if ($LASTEXITCODE -ne 0) {
    throw 'The WiX MSI build failed.'
}

$hash = (Get-FileHash -LiteralPath $msiPath -Algorithm SHA256).Hash
Write-Host "MSI created: $msiPath"
Write-Host "SHA-256: $hash"

if ($CopyToDownloads) {
    $downloadsDirectory = Join-Path ([Environment]::GetFolderPath('UserProfile')) 'Downloads'
    if (-not (Test-Path -LiteralPath $downloadsDirectory)) {
        throw "Downloads folder not found: $downloadsDirectory"
    }
    $downloadsPath = Join-Path $downloadsDirectory (Split-Path -Leaf $msiPath)
    Copy-Item -LiteralPath $msiPath -Destination $downloadsPath -Force
    Write-Host "Copied to: $downloadsPath"
}
