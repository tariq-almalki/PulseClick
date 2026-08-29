# Release packaging

## Recommended Windows distribution

PulseClick should be distributed with a signed x64 MSI installer as the primary Windows download. The MSI provides a familiar setup flow, a Start Menu shortcut, a Program Files installation, version-aware upgrades, and a clean uninstall entry. Keep a signed portable EXE as an optional download for advanced users.

Build it from the project directory:

```powershell
.\installer\build-msi.ps1
```

The build produces `dist/PulseClick-Setup-<version>-x64.msi`. Use `-CopyToDownloads` to place a copy in the local Downloads folder.

## Windows trust

The installer format does not by itself remove SmartScreen warnings. Public distribution needs trusted Authenticode signing. Sign the application executable and the MSI, use SHA-256, and add a trusted timestamp. A self-signed certificate is suitable only for private testing and will still appear untrusted on other machines.

For the recommended public release, use Microsoft Artifact Signing with a **Public Trust** certificate profile. The local workflow is ready:

```powershell
.\installer\sign-artifact-release.ps1 `
  -Endpoint "https://<region>.codesigning.azure.net/" `
  -AccountName "<artifact-signing-account-name>" `
  -CertificateProfileName "<public-trust-certificate-profile-name>" `
  -CopyToDownloads
```

It signs the application first, rebuilds the MSI with the signed application inside it, signs the MSI, and verifies both signatures. Artifact Signing requires an account, identity validation, a Public Trust certificate profile, the Certificate Profile Signer role, Azure authentication, and Microsoft Artifact Signing Client Tools installed as administrator.

The local certificate-based signing helper remains available when you already have an OV certificate with an accessible private key in the current user's certificate store:

```powershell
.\installer\sign-release.ps1 -CertificateThumbprint "YOUR_CERTIFICATE_THUMBPRINT"
```

Microsoft Artifact Signing or another publicly trusted code-signing certificate can be used for customer releases. A new signed binary may still need time to establish SmartScreen reputation.

## Microsoft Store later

The Microsoft Store/MSIX route is the strongest Windows-native trust and update experience. It can be added later after the direct MSI workflow is stable. The current MSI is intentionally suitable for local testing and direct GitHub or website distribution.
