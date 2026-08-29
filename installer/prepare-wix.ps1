[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$projectRoot = Split-Path -Parent $PSScriptRoot
$toolsRoot = Join-Path $projectRoot '.tools'
$wixVersion = '4.0.6'
$uiVersion = '4.0.6'

$packages = @(
    @{
        Name = 'wix'
        Version = $wixVersion
        Url = "https://api.nuget.org/v3-flatcontainer/wix/$wixVersion/wix.$wixVersion.nupkg"
        Archive = Join-Path $toolsRoot "wix.$wixVersion.nupkg"
        Destination = Join-Path $toolsRoot 'wix4'
        ExpectedHash = 'A94DD42AE1FB56B32DA180E2173CEDA4F0D10B4C8871C5EE59ECB502131A1EB6'
        RequiredFile = Join-Path $toolsRoot 'wix4\tools\net6.0\any\wix.exe'
    }
    @{
        Name = 'WixToolset.UI.wixext'
        Version = $uiVersion
        Url = "https://api.nuget.org/v3-flatcontainer/wixtoolset.ui.wixext/$uiVersion/wixtoolset.ui.wixext.$uiVersion.nupkg"
        Archive = Join-Path $toolsRoot "WixToolset.UI.wixext.$uiVersion.nupkg"
        Destination = Join-Path $toolsRoot 'WixToolset.UI.wixext4'
        ExpectedHash = 'E92C4DDAC5D17F5360291AB856C676E19FD92AF14DAFFB5D5D4B8E1E9C716B47'
        RequiredFile = Join-Path $toolsRoot 'WixToolset.UI.wixext4\wixext4\WixToolset.UI.wixext.dll'
    }
)

New-Item -ItemType Directory -Path $toolsRoot -Force | Out-Null

foreach ($package in $packages) {
    if (-not (Test-Path -LiteralPath $package.RequiredFile)) {
        Write-Host "Downloading $($package.Name) $($package.Version) from NuGet..."
        Invoke-WebRequest -Uri $package.Url -OutFile $package.Archive -UseBasicParsing
    }

    $actualHash = (Get-FileHash -LiteralPath $package.Archive -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($actualHash -ne $package.ExpectedHash) {
        throw "$($package.Name) package hash does not match the pinned SHA-256 value. Delete the incomplete archive and try again."
    }

    if (-not (Test-Path -LiteralPath $package.RequiredFile)) {
        Write-Host "Extracting $($package.Name)..."
        Expand-Archive -LiteralPath $package.Archive -DestinationPath $package.Destination -Force
    }
}

Write-Host 'WiX 4.0.6 is ready for the PulseClick MSI build.'
