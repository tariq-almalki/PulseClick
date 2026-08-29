[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$CertificateThumbprint,

    [string]$TimestampUrl = 'http://timestamp.digicert.com'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$appPath = Join-Path $projectRoot 'target\release\pulseclick.exe'

if (-not (Test-Path -LiteralPath $appPath)) {
    throw "Release executable not found: $appPath"
}

$normalizedThumbprint = $CertificateThumbprint -replace '\s', ''
$certificate = Get-ChildItem Cert:\CurrentUser\My | Where-Object {
    ($_.Thumbprint -replace '\s', '') -eq $normalizedThumbprint
} | Select-Object -First 1
if (-not $certificate) {
    throw 'The requested certificate was not found in the current user certificate store.'
}
if (-not $certificate.HasPrivateKey) {
    throw 'The requested certificate does not have an accessible private key.'
}
if ($certificate.NotAfter -lt (Get-Date)) {
    throw 'The requested code-signing certificate is expired.'
}

$signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if ($signtool) {
    $signtoolPath = $signtool.Source
} else {
    $sdkCandidates = @(
        (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin\*\x64\signtool.exe')
        (Join-Path $env:ProgramFiles 'Windows Kits\10\bin\*\x64\signtool.exe')
    )
    $signtoolPath = Get-ChildItem -Path $sdkCandidates -File -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $signtoolPath) {
    throw 'signtool.exe was not found. Install the Windows SDK or use Microsoft Artifact Signing.'
}

$signArguments = @(
    'sign'
    '/sha1'
    $certificate.Thumbprint
    '/fd'
    'SHA256'
    '/tr'
    $TimestampUrl
    '/td'
    'SHA256'
    '/d'
    'PulseClick'
    '/du'
    'https://github.com/tariq-almalki/PulseClick'
)

Write-Host 'Signing the application before rebuilding the MSI...'
& $signtoolPath @signArguments $appPath
if ($LASTEXITCODE -ne 0) {
    throw "Signing failed for $appPath"
}
& $signtoolPath verify /pa /all $appPath
if ($LASTEXITCODE -ne 0) {
    throw "Signature verification failed for $appPath"
}

& (Join-Path $PSScriptRoot 'build-msi.ps1')
if ($LASTEXITCODE -ne 0) {
    throw 'The MSI rebuild failed after signing the application.'
}

$msiPath = Get-ChildItem -LiteralPath (Join-Path $projectRoot 'dist') -Filter 'PulseClick-Setup-*.msi' -File |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $msiPath) {
    throw 'The signed application rebuild did not produce an MSI.'
}

Write-Host "Signing the MSI..."
& $signtoolPath @signArguments $msiPath.FullName
if ($LASTEXITCODE -ne 0) {
    throw "Signing failed for $($msiPath.FullName)"
}
& $signtoolPath verify /pa /all $msiPath.FullName
if ($LASTEXITCODE -ne 0) {
    throw "Signature verification failed for $($msiPath.FullName)"
}

Write-Host 'The executable and MSI are signed and timestamped.'
