# PulseClick

PulseClick is a fast, keyboard-first Windows auto-clicker built in Rust. It is designed for precise repetitive input while keeping the interface simple, responsive, and easy to verify at a glance.

## Highlights

- Single, double, triple, quadruple, and custom 5–1,000-click bursts.
- Batched turbo mode when the burst gap is set to 0 ms.
- Configurable start/stop hotkey, with F6 as the default.
- F8 emergency stop and F9 target capture.
- Current-cursor or fixed-position targeting.
- Continuous or fixed-cycle operation.
- Black and Light themes.
- Desktop click indicator with an in-app preview.
- Portable Windows release executable with an embedded black icon.
- Native x64 MSI installer with a Start Menu shortcut and clean uninstall support.

## Requirements

- Windows 10 or later.
- Rust stable toolchain with the MSVC target.
- Node.js 18+ only if you want to run the documentation site locally.

## Build and run

```powershell
cargo run --release
```

Create the portable executable without launching it:

```powershell
cargo build --release
```

The executable is written to `target/release/pulseclick.exe`.

Create the Windows MSI installer locally:

```powershell
.\installer\build-msi.ps1
```

The first installer build downloads the pinned WiX 4.0.6 packages from NuGet, verifies their SHA-256 hashes, and stores them in the ignored `.tools/` directory. The installer is written to `dist/` as `PulseClick-Setup-<version>-x64.msi`. Add `-CopyToDownloads` to copy it to your Windows Downloads folder.

For a public release, sign both `target/release/pulseclick.exe` and the MSI with a trusted code-signing certificate before uploading them. The helper is:

```powershell
.\installer\sign-release.ps1 -CertificateThumbprint "YOUR_CERTIFICATE_THUMBPRINT"
```

An unsigned or self-signed build can still trigger Microsoft Defender SmartScreen and show **Unknown publisher**. Packaging as MSI improves installation and removal, but trusted code signing is what identifies the publisher.

The recommended Microsoft Public Trust workflow is also prepared:

```powershell
.\installer\sign-artifact-release.ps1 `
  -Endpoint "https://<region>.codesigning.azure.net/" `
  -AccountName "<artifact-signing-account-name>" `
  -CertificateProfileName "<public-trust-certificate-profile-name>" `
  -CopyToDownloads
```

It signs the executable before rebuilding the MSI, signs the MSI, verifies both signatures, and optionally copies the signed installer to Downloads. It requires an Artifact Signing account with identity validation, a **Public Trust** certificate profile, the Certificate Profile Signer role, Azure authentication, and Microsoft Artifact Signing Client Tools installed as administrator.

### Certificate purpose matters

Let’s Encrypt certificates are Domain Validation TLS certificates for HTTPS websites. They prove control of a domain but are not issued for Windows Authenticode code signing, so they cannot remove the SmartScreen publisher warning for PulseClick. Use Microsoft Artifact Signing with a **Public Trust** profile, or another publicly trusted code-signing certificate, for the EXE and MSI.

Run the tests with:

```powershell
cargo test --all-targets
```

## Local documentation

The documentation site is in `docs-site/` and uses VitePress:

```powershell
cd docs-site
npm install
npm run docs:dev
```

Then open the local address printed by VitePress. The documentation covers getting started, configuration, architecture, verification, and future deployment notes.

## Project layout

```text
src/main.rs              Rust application and Windows input/indicator code
assets/                  Application icon assets
build.rs                 Windows resource embedding
docs-site/docs/          Local VitePress documentation
```

## Responsible use

PulseClick sends real mouse input to the active desktop or selected target. Test it in a safe window first, keep the emergency stop available, and use it only where automation is allowed.

## License

MIT
