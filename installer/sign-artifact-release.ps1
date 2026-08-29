[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Endpoint,

    [Parameter(Mandatory = $true)]
    [string]$AccountName,

    [Parameter(Mandatory = $true)]
    [string]$CertificateProfileName,

    [string]$CorrelationId = 'PulseClick-local-release',

    [string]$TimestampUrl = 'http://timestamp.acs.microsoft.com',

    [switch]$CopyToDownloads
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$appPath = Join-Path $projectRoot 'target\release\pulseclick.exe'
$clientRootCandidates = @(
    (Join-Path ${env:ProgramFiles(x86)} 'Microsoft\ArtifactSigningClientTools')
    (Join-Path $env:ProgramFiles 'Microsoft\ArtifactSigningClientTools')
)

if (-not (Test-Path -LiteralPath $appPath)) {
    throw "Release executable not found: $appPath"
}

$signtoolCommand = Get-Command signtool.exe -ErrorAction SilentlyContinue
if ($signtoolCommand) {
    $signtoolPath = $signtoolCommand.Source
} else {
    $signtoolPath = $null
    foreach ($clientRoot in $clientRootCandidates) {
        if (Test-Path -LiteralPath $clientRoot) {
            $candidate = Get-ChildItem -LiteralPath $clientRoot -Filter 'signtool.exe' -File -Recurse -ErrorAction SilentlyContinue |
                Select-Object -First 1 -ExpandProperty FullName
            if ($candidate) {
                $signtoolPath = $candidate
                break
            }
        }
    }
    if (-not $signtoolPath) {
        $sdkCandidates = @(
            (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin\*\x64\signtool.exe')
            (Join-Path $env:ProgramFiles 'Windows Kits\10\bin\*\x64\signtool.exe')
        )
        $signtoolPath = Get-ChildItem -Path $sdkCandidates -File -ErrorAction SilentlyContinue |
            Sort-Object FullName -Descending |
            Select-Object -First 1 -ExpandProperty FullName
    }
}
if (-not $signtoolPath) {
    throw 'signtool.exe was not found. Install Microsoft Artifact Signing Client Tools as an administrator.'
}

$dlibPath = $null
foreach ($clientRoot in $clientRootCandidates) {
    if (Test-Path -LiteralPath $clientRoot) {
        $candidate = Get-ChildItem -LiteralPath $clientRoot -Filter 'Azure.CodeSigning.Dlib.dll' -File -Recurse -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName
        if ($candidate) {
            $dlibPath = $candidate
            break
        }
    }
}
if (-not $dlibPath) {
    throw 'Azure.CodeSigning.Dlib.dll was not found. Install Microsoft Artifact Signing Client Tools as an administrator.'
}

$hasServicePrincipalCredentials =
    -not [string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_ID) -and
    -not [string]::IsNullOrWhiteSpace($env:AZURE_TENANT_ID) -and
    -not [string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_SECRET)
$azureCli = Get-Command az.exe -ErrorAction SilentlyContinue
if (-not $hasServicePrincipalCredentials -and -not $azureCli) {
    throw 'Authenticate with Azure CLI (az login) or set AZURE_CLIENT_ID, AZURE_TENANT_ID, and AZURE_CLIENT_SECRET before signing.'
}

$metadataPath = Join-Path ([IO.Path]::GetTempPath()) ('PulseClick-artifact-signing-' + [Guid]::NewGuid().ToString('N') + '.json')
$metadata = [ordered]@{
    Endpoint = $Endpoint
    CodeSigningAccountName = $AccountName
    CertificateProfileName = $CertificateProfileName
    CorrelationId = $CorrelationId
}
$metadata | ConvertTo-Json | Set-Content -LiteralPath $metadataPath -Encoding utf8

function Invoke-ArtifactSigning {
    param([Parameter(Mandatory = $true)][string]$Path)

    Write-Host "Signing $Path with Artifact Signing Public Trust..."
    $signArguments = @(
        'sign'
        '/v'
        '/fd'
        'SHA256'
        '/tr'
        $TimestampUrl
        '/td'
        'SHA256'
        '/dlib'
        $dlibPath
        '/dmdf'
        $metadataPath
    )
    & $signtoolPath @signArguments $Path
    if ($LASTEXITCODE -ne 0) {
        throw "Artifact Signing failed for $Path"
    }

    & $signtoolPath verify /pa /all $Path
    if ($LASTEXITCODE -ne 0) {
        throw "Signature verification failed for $Path"
    }
}

try {
    # The executable must be signed before the MSI is built so the installed file is signed too.
    Invoke-ArtifactSigning -Path $appPath

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

    Invoke-ArtifactSigning -Path $msiPath.FullName

    if ($CopyToDownloads) {
        $downloadsDirectory = Join-Path ([Environment]::GetFolderPath('UserProfile')) 'Downloads'
        if (-not (Test-Path -LiteralPath $downloadsDirectory)) {
            throw "Downloads folder not found: $downloadsDirectory"
        }
        $downloadsPath = Join-Path $downloadsDirectory $msiPath.Name
        Copy-Item -LiteralPath $msiPath.FullName -Destination $downloadsPath -Force
        Write-Host "Copied signed MSI to: $downloadsPath"
    }
} finally {
    Remove-Item -LiteralPath $metadataPath -Force -ErrorAction SilentlyContinue
}

Write-Host 'The executable and MSI are signed and verified with Artifact Signing Public Trust.'
